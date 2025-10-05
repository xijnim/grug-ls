use lsp_server::{Connection, Message};
use lsp_types::Uri;
use lsp_types::{
    ClientCapabilities, CompletionParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, HoverParams,
};
use tree_sitter::Parser;
use vfs::{FileSystem, MemoryFS};

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
mod formatting;

use log::error;
use log::info;

pub enum ServerFileElement {
    File(String),
    Directory(String, Vec<ServerFileElement>),
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
    pub fn get_document_by_uri<'a>(&'a self, uri: &Uri) -> Option<&'a Document> {
        let uri = uri.as_str();
        // We probably wont need to use this server on TCP
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];

        if !self.file_system.exists(path).unwrap_or(false) {
            return None;
        }

        self.document_map.get(path)

    }

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
            "textDocument/formatting" => {
                let params: lsp_types::DocumentFormattingParams = serde_json::from_value(params).unwrap();

                self.formatting(params, connection, id.unwrap());
            }
            "exit" => {
                self.should_exit = true;
            }
            _ => error!("Unknown message method: {}", method),
        }
    }
}
