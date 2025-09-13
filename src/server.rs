use serde::{Deserialize, Serialize};
use tree_sitter::Parser;
use vfs::MemoryFS;

use crate::{
    rpc::{Notification, RequestMessage, ResponseMessage, Rpc}, server::{
        document::Document, init::{InitResponse, InitResult, ServerCapabilities, ServerInfo}, text_sync::{DidChangeNotificationParams, DidOpenNotificationParams, TextDocumentPositionParams}
    }, LoggableResult, Logger
};
use std::{collections::HashMap, default, path::PathBuf, process};

mod init;
mod text_sync;
mod hover;
mod document;
mod utils;

pub enum ServerWrapper {
    Inactive,
    Active(Server),
    Shutdown,
}

pub enum ServerFileElement {
    File(String),
    Directory(String, Vec<ServerFileElement>),
}

impl ServerWrapper {
    pub fn new() -> ServerWrapper {
        ServerWrapper::Inactive
    }

    pub fn handle_message(
        &mut self,
        json: String,
        logger: &mut Logger,
        rpc: &mut Rpc,
        parser: &mut Parser,
    ) {
        let message: Result<serde_json::Value, _> = serde_json::from_str(&json);
        if let Err(ref error) = message {
            logger.log_str(format!("Error parsing json: {:?}", error));
            logger.log_str(format!("The received json was: {:?}", json));
            panic!();
        }

        let message = message.unwrap();

        if message.get("method").is_none() {
            logger.log_str("Received message from the client with no method");
            panic!();
        }

        let method = message.get("method").unwrap();
        if !method.is_string() {
            logger.log_str("The method property of the client message wasnt a string");
            panic!();
        }
        let method = method.as_str().unwrap();

        let server = match self {
            ServerWrapper::Inactive | ServerWrapper::Shutdown => None,
            ServerWrapper::Active(server) => Some(server),
        };

        match method {
            "initialize" => {
                if let ServerWrapper::Active(_) = self {
                    logger.log_str("Client tried to initialize twice");
                    panic!();
                };
                logger.log_str("Connected with the client, bitch");

                let id = message.get("id").unwrap().clone();
                let server = Server::from_request(logger, message);
                logger.log_str(format!("{:?}", server));
                *self = ServerWrapper::Active(server);

                let response = ResponseMessage::new(
                    id,
                    InitResult {
                        server_info: Some(ServerInfo {
                            name: "Grug-LS".to_string(),
                            version: Some("1.0.0".to_string()),
                        }),
                        capabilities: ServerCapabilities {
                            position_encoding: "utf-8".to_string(),

                            //TODO: Incremental updates
                            text_document_sync: 1,

                            hover_provider: true,
                        },
                    },
                );
                let res_json = serde_json::to_string_pretty(&response);
                if let Err(ref error) = res_json {
                    logger.log_str(format!("Error encoding response json: {:?}", error));
                }
                let res_json = res_json.unwrap();

                rpc.send(&*res_json);
                logger.log_str(format!("Sent this init response: {}", res_json));

                return;
            }
            "initialized" => {
                logger.log_str("Client connection was established!");
            }
            "textDocument/didOpen" => {
                let did_open_notification: Notification<DidOpenNotificationParams> =
                    serde_json::from_value(message)
                        .unwrap_or_log(logger, "Did Open notification parse error");

                server
                    .unwrap_or_log(logger, "didOpen while inactive server")
                    .handle_did_open(logger, did_open_notification, parser);
            }
            "textDocument/didChange" => {
                let did_change_notification: Notification<DidChangeNotificationParams> =
                    serde_json::from_value(message)
                        .unwrap_or_log(logger, "Did Change notification parse error");

                server
                    .unwrap_or_log(logger, "didChange while inactive server")
                    .handle_did_change(logger, did_change_notification, parser);
            }
            "textDocument/didSave" => {
                logger.log_str("Saved file");
            }
            "shutdown" => {
                *self = ServerWrapper::Shutdown;

                let res: ResponseMessage<serde_json::Value> =
                    ResponseMessage::new(serde_json::Value::Null, serde_json::Value::Null);

                rpc.send(serde_json::to_string_pretty(&res).unwrap());

                logger.log_str("Shutting down");
                std::process::exit(0);
            }
            "textDocument/hover" => {
                let req: RequestMessage<TextDocumentPositionParams> =
                    serde_json::from_value(message)
                    .unwrap_or_log(logger, "textDocument/hover parse error");

                
                server
                    .unwrap_or_log(logger, "didHover while inactive server")
                    .handle_hover(req, rpc);
            }
            "exit" => {
                if let ServerWrapper::Shutdown = self {
                    logger.log_str("Exiting with 0");
                    std::process::exit(0);
                } else {
                    logger.log_str("Exiting with 1");
                    std::process::exit(1);
                }
            }
            _ => logger.log_str(format!("Unknown message method: {}", method)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientCapabilities {
    #[serde(rename = "textDocument")]
    #[serde(default)]
    text_document: TextDocumentClientCapabilities,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Default)]
struct TextDocumentClientCapabilities {
    #[serde(default)]
    synchronization: TextDocumentSyncClientCapabilities,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Default)]
struct TextDocumentSyncClientCapabilities {
    #[serde(rename = "dynamicRegistration")]
    #[serde(default)]
    dynamic_registration: bool,
}

#[derive(Debug)]
pub struct Server {
    root_path: PathBuf,
    client_capabilities: ClientCapabilities,
    file_system: MemoryFS,
    document_map: HashMap<String, Document>,
}
