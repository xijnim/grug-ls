use serde::{Deserialize, Serialize};
use tree_sitter::Parser;
use vfs::MemoryFS;

use crate::{
    rpc::{Notification, RequestMessage, ResponseMessage, Rpc},
    server::{
        completion::CompletionParams, document::Document, helper::{spawn_worker, ServerUpdate}, init::{InitResponse, InitResult, ServerCapabilities, ServerInfo}, mod_api::ModApi, text_sync::{
            DidChangeNotificationParams, DidOpenNotificationParams, TextDocumentPositionParams,
        }
    },
};
use std::{collections::HashMap, default, path::PathBuf, process, sync::mpsc::Receiver};

mod document;
mod helper;
mod hover;
mod init;
mod mod_api;
mod text_sync;
mod utils;
mod completion;

use log::error;
use log::info;

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

    pub fn handle_message(&mut self, json: String, rpc: &mut Rpc, parser: &mut Parser) {
        let message: Result<serde_json::Value, _> = serde_json::from_str(&json);
        if let Err(ref error) = message {
            error!("Error parsing json:\nSRC: {}ERROR: {:?}", json, error);
            panic!();
        }

        let message = message.unwrap();

        if message.get("method").is_none() {
            error!("Received message from the client with not method");
            panic!();
        }

        let method = message.get("method").unwrap();
        if !method.is_string() {
            error!("The method property of the client message wasnt a string");
            panic!();
        }
        let method = method.as_str().unwrap();

        let mut server = match self {
            ServerWrapper::Inactive | ServerWrapper::Shutdown => None,
            ServerWrapper::Active(server) => Some(server),
        };

        if let Some(ref mut server) = server {
            server.handle_worker_messages();
        }

        match method {
            "initialize" => {
                if let ServerWrapper::Active(_) = self {
                    error!("Client tried to initialize twice");
                    panic!();
                };
                info!("Connected with the client");

                let id = message.get("id").unwrap().clone();
                let server = Server::from_request(message);

                info!("Started new server: {:?}", server);

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
                            completion_provider: std::collections::HashMap::new(),
                        },
                    },
                );
                let res_json = serde_json::to_string_pretty(&response);
                if let Err(ref error) = res_json {
                    error!("Error encoding response json: {:?}", error);
                    panic!();
                }

                let res_json = res_json.unwrap();

                rpc.send(&*res_json);
                info!("Sent this init response: {}", res_json);

                return;
            }
            "initialized" => {
                info!("Client connection was established!");
            }
            "textDocument/didOpen" => {
                let did_open_notification: Notification<DidOpenNotificationParams> =
                    serde_json::from_value(message).unwrap();

                server
                    .unwrap()
                    .handle_did_open(did_open_notification, parser);
            }
            "textDocument/didChange" => {
                let did_change_notification: Notification<DidChangeNotificationParams> =
                    serde_json::from_value(message).unwrap();

                server
                    .unwrap()
                    .handle_did_change(did_change_notification, parser);
            }
            "textDocument/didSave" => {
                info!("Saved file");
            }
            "shutdown" => {
                *self = ServerWrapper::Shutdown;

                let res: ResponseMessage<serde_json::Value> =
                    ResponseMessage::new(serde_json::Value::Null, serde_json::Value::Null);

                rpc.send(serde_json::to_string_pretty(&res).unwrap());

                info!("Shutting down");
                std::process::exit(0);
            }
            "textDocument/hover" => {
                let req: RequestMessage<TextDocumentPositionParams> =
                    serde_json::from_value(message).unwrap();

                server.unwrap().handle_hover(req, rpc);
            }
            "textDocument/completion" => {
                let req: RequestMessage<CompletionParams> =
                    serde_json::from_value(message).unwrap();

                server.unwrap().handle_completion(req, rpc);
            }
            "exit" => {
                if let ServerWrapper::Shutdown = self {
                    info!("Exiting with 0");
                    std::process::exit(0);
                } else {
                    error!("Exiting with 0");
                    std::process::exit(1);
                }
            }
            _ => error!("Unknown message method: {}", method),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientCapabilities {
    #[serde(rename = "textDocument")]
    #[serde(default)]
    text_document: TextDocumentClientCapabilities,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct TextDocumentClientCapabilities {
    #[serde(default)]
    synchronization: TextDocumentSyncClientCapabilities,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct TextDocumentSyncClientCapabilities {
    #[serde(rename = "dynamicRegistration")]
    #[serde(default)]
    dynamic_registration: bool,
}

#[derive(Debug)]
pub struct Server {
    root_path: PathBuf,
    client_capabilities: ClientCapabilities,
    mod_api: ModApi,
    file_system: MemoryFS,
    document_map: HashMap<String, Document>,
    messages_chan: Receiver<ServerUpdate>,
}
