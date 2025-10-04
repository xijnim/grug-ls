use lsp_server::{Connection, Message};
use lsp_types::{
    ClientCapabilities, CompletionParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, HoverParams,
};
use serde::{Deserialize, Serialize};
use tree_sitter::Parser;
use vfs::MemoryFS;

use crate::server::{document::Document, helper::ServerUpdate, mod_api::ModApi};
use std::{collections::HashMap, path::PathBuf, sync::mpsc::Receiver};

mod completion;
mod document;
mod goto_definition;
mod helper;
mod hover;
pub mod init;
mod mod_api;
mod text_sync;
mod utils;
mod rename;

use log::error;
use log::info;

pub enum ServerFileElement {
    File(String),
    Directory(String, Vec<ServerFileElement>),
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
    pub should_exit: bool,
    root_path: PathBuf,
    client_capabilities: ClientCapabilities,
    mod_api: ModApi,
    file_system: MemoryFS,
    document_map: HashMap<String, Document>,
    messages_chan: Receiver<ServerUpdate>,
}

impl Server {
    pub fn handle_message(
        &mut self,
        message: Message,
        connection: &mut Connection,
        parser: &mut Parser,
    ) {
        self.handle_worker_messages();

        let (id, method, params) = match message {
            Message::Request(req) => (Some(req.id), req.method, req.params),
            Message::Notification(notif) => (None, notif.method, notif.params),
            _ => {
                return;
            }
        };

        match method.as_str() {
            "textDocument/didOpen" => {
                let did_open_notification: DidOpenTextDocumentParams =
                    serde_json::from_value(params).unwrap();

                self.handle_did_open(did_open_notification, parser);
            }
            "textDocument/didChange" => {
                let did_change_notification: DidChangeTextDocumentParams =
                    serde_json::from_value(params).unwrap();

                self.handle_did_change(did_change_notification, parser);
            }
            "textDocument/didSave" => {
                info!("Saved file");
            }
            "shutdown" => {
                info!("Shutting down");
                self.should_exit = true;
            }
            "textDocument/hover" => {
                let req: HoverParams = serde_json::from_value(params).unwrap();

                self.handle_hover(req, connection, id.unwrap());
            }
            "textDocument/completion" => {
                let req: CompletionParams = serde_json::from_value(params).unwrap();

                self.handle_completion(req, connection, id.unwrap());
            }
            "textDocument/definition" => {
                let params: GotoDefinitionParams = serde_json::from_value(params).unwrap();

                self.handle_goto_definition(params, connection, id.unwrap());
            }
            "textDocument/rename" => {
                let params: lsp_types::RenameParams = serde_json::from_value(params).unwrap();

                self.rename(params, connection, id.unwrap());

            }
            "exit" => {
                self.should_exit = true;
            }
            _ => error!("Unknown message method: {}", method),
        }
    }
}
