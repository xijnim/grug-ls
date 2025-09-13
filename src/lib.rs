use std::{borrow::Borrow, fmt::Debug, io::Write};

pub mod rpc;
pub mod server;

pub struct Logger {
    file: std::fs::File,
}

impl Logger {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Logger {
        let file = std::fs::File::create(path).unwrap();
        Logger { file }
    }

    pub fn log_str<S: Borrow<str>>(&mut self, text: S) {
        let text = text.borrow();
        self.file.write(text.as_bytes()).unwrap();
        if !text.ends_with('\n') {
            self.file.write(b"\n").unwrap();
        }
    }
}

trait LoggableResult<T> {
    fn unwrap_or_log<S: Borrow<str>>(self, logger: &mut Logger, message: S) -> T;
}

impl<T, E: Debug> LoggableResult<T> for Result<T, E> {
    fn unwrap_or_log<S: Borrow<str>>(self, logger: &mut Logger, message: S) -> T {
        let message = message.borrow();
        match self {
            Ok(value) => value,
            Err(err) => {
                logger.log_str(message);
                logger.log_str(format!("{:?}", err));
                panic!()
            }
        }
    }
}

impl<T> LoggableResult<T> for Option<T> {
    fn unwrap_or_log<S: Borrow<str>>(self, logger: &mut Logger, message: S) -> T {
        let message = message.borrow();
        match self {
            Some(value) => value,
            None => {
                logger.log_str(message);
                eprintln!("{}", std::backtrace::Backtrace::capture().to_string());
                
                panic!()
            }
        }
    }
}
