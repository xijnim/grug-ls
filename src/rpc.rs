use std::{
    borrow::Borrow, fmt::Debug, io::{Read, Write}
};

use serde::{Deserialize, Serialize};

use crate::Logger;

#[derive(Serialize, Deserialize)]
pub struct ResponseMessage<T> {
    #[serde(default = "default_jsonrpc")]
    jsonrpc: String,

    pub id: serde_json::Value,

    pub result: T,
}

#[derive(Serialize, Deserialize)]
pub struct RequestMessage<T> {
    jsonrpc: String,

    pub id: serde_json::Value,
    
    pub params: T,
}

impl<T> RequestMessage<T> {
    pub fn new(id: serde_json::Value, params: T) -> RequestMessage<T> {
        RequestMessage {
            jsonrpc: "2.0".to_string(),
            id,
            params,
        }
    }
}

impl<T> ResponseMessage<T> {
    pub fn new(id: serde_json::Value, result: T) -> ResponseMessage<T> {
        ResponseMessage {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Notification<T: Debug> {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,

    pub params: T,
}

pub struct Rpc {
    reader: Box<dyn Read>,
    writer: Box<dyn Write>,
    buffer: Vec<u8>,
}

impl<'a> Rpc {
    pub fn new<R: Read + 'static, W: Write + 'static>(reader: R, writer: W) -> Rpc {
        Rpc {
            reader: Box::new(reader),
            writer: Box::new(writer),
            buffer: [0; 65536].to_vec(),
        }
    }

    pub fn recv(&'a mut self, logger: &mut Logger) -> &'a [u8] {
        let mut bytes_read = 0;
        loop {
            let amt = self.reader.read(&mut self.buffer[bytes_read..]).unwrap();
            bytes_read += amt;

            if bytes_read == self.buffer.len() {
                logger.log_str("Increasing recv buffer size");
                self.buffer.resize(self.buffer.len(), 0);
            }

            let mut header_length: Option<usize> = None;

            for idx in 0..bytes_read {
                if self.buffer[idx..].starts_with("\r\n\r\n".as_bytes()) {
                    header_length = Some(idx);
                }
            }

            if header_length.is_none() {
                continue;
            }
            let header_length = header_length.unwrap();
            let content_length_bytes = &self.buffer[0..header_length];

            const CONTENT_LENGTH_HEADER: &[u8] = "Content-Length: ".as_bytes();
            if !content_length_bytes.starts_with(CONTENT_LENGTH_HEADER) {
                let mut error_msg = "Content Length Error".to_string();

                if let Ok(text) = String::from_utf8(content_length_bytes.to_vec()) {
                    error_msg.push_str(&format!(": {}", text));
                }

                logger.log_str(error_msg);
                panic!();
            }

            let length: usize =
                String::from_utf8(content_length_bytes[CONTENT_LENGTH_HEADER.len()..].to_vec())
                    .unwrap()
                    .parse()
                    .unwrap();

            if bytes_read < length + CONTENT_LENGTH_HEADER.len() + 4 {
                continue;
            }

            logger.log_str(format!("Got message of length: {}\n", length));

            let content = &self.buffer
                [content_length_bytes.len() + 4..content_length_bytes.len() + 4 + length];

            return content;
        }
    }

    pub fn send<S: Borrow<str>>(&mut self, json: S) {
        let json = json.borrow();
        let mut content = json.as_bytes().to_vec();
        let mut header = format!("Content-Length: {}\r\n\r\n", content.len())
            .as_bytes()
            .to_vec();
        let mut message: Vec<u8> = Vec::with_capacity(content.len() + header.len());
        message.append(&mut header);
        message.append(&mut content);

        self.writer.write(&mut message).unwrap();
        self.writer.flush().unwrap();
    }
}
