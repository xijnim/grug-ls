use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use tree_sitter::Parser;
use vfs::FileSystem;

use crate::{rpc::Notification, server::{document::Document, Server}, LoggableResult, Logger};

type DocumentURI = String;

#[derive(Serialize, Deserialize, Debug)]
struct TextDocumentItem {
    uri: DocumentURI,

    #[serde(rename = "languageId")]
    language: String,
    version: isize,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextDocumentIdentifier {
    pub uri: DocumentURI,
}

#[derive(Serialize, Deserialize, Debug)]
struct VersionedTextDocumentIdentifier {
    uri: DocumentURI,
    version: isize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidOpenNotificationParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentItem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidChangeNotificationParams {
    #[serde(rename = "textDocument")]
    text_document: VersionedTextDocumentIdentifier,

    #[serde(rename = "contentChanges")]
    content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextDocumentPositionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,

    pub position: Position,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextDocumentContentChangeEvent {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrugOnFunction {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrugEntity {
    pub description: String,
    pub on_functions: Vec<GrugOnFunction>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[derive(Debug)]
pub enum GrugType {
    String,
    I32,
    F32,
    Resource { resource_extension: String },
    Entity { entity_type: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[derive(Debug)]
pub enum GrugArgument {
    #[serde(rename = "string")]
    String { name: String },
    #[serde(rename = "i32")]
    I32 { name: String },
    #[serde(rename = "f32")]
    F32 { name: String },
    #[serde(rename = "resource")]
    Resource {
        name: String,
        resource_extension: String,
    },
    #[serde(rename = "entity")]
    Entity { name: String, entity_type: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrugGameFunction {
    pub description: String,
    pub arguments: Vec<GrugArgument>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModApi {
    pub entities: HashMap<String, GrugEntity>,
    pub game_functions: HashMap<String, GrugGameFunction>,
}

#[derive(Debug)]
pub struct GrugModInfo {
    pub name: String,
    pub version: String,
    pub game_version: String,
    pub author: String,
}

#[derive(Debug)]
pub struct GrugMod {
    pub about: GrugModInfo,
}

impl Server {
    pub fn handle_did_open(
        &mut self,
        logger: &mut Logger,
        req: Notification<DidOpenNotificationParams>,
        parser: &mut Parser,
    ) {
        // We probably wont need to use this server on TCP
        assert!(req.params.text_document.uri.starts_with("file://"));

        let path = &req.params.text_document.uri["file.//".len()..];
        let path = PathBuf::from_str(path).unwrap_or_log(logger, "Unresolved file path");

        logger.log_str(format!("Opened the file: {:?}", path.to_str().unwrap()));

        let paths: Vec<&Path> = path.ancestors().collect();
        let piece_amt = paths.len();

        for (idx, path) in paths.into_iter().rev().enumerate() {
            let is_file = idx == piece_amt - 1;
            let path = path.to_str().unwrap();

            if is_file {
                if self.file_system.exists(path).unwrap() {
                    logger.log_str(format!(
                        "Trying to create file that already exists: {}",
                        path
                    ));
                    break;
                }

                let mut file = self.file_system.create_file(path).unwrap();
                file.write(path.as_bytes())
                    .unwrap();

                let document = Document::new(parser, req.params.text_document.text.as_bytes().to_vec());
                logger.log_str(format!("{:?}", document));
                self.document_map.insert(path.to_string(), document);
                break;
            }

            if !self.file_system.exists(path).unwrap() {
                self.file_system.create_dir(path).unwrap();
            }
        }
    }

    pub fn handle_did_change(
        &mut self,
        logger: &mut Logger,
        req: Notification<DidChangeNotificationParams>,
        parser: &mut Parser,
    ) {
        assert!(req.params.text_document.uri.starts_with("file://"));

        let path = &req.params.text_document.uri["file.//".len()..];

        logger.log_str(format!("Updated file: {:?}", path));

        let document = self.document_map.get_mut(path).unwrap();
        *document = Document::new(parser, req.params.content_changes[0].text.as_bytes().to_vec());
    }
}
