use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams};
use tree_sitter::Parser;
use vfs::FileSystem;

use crate::server::{Server, document::Document};

use log::error;
use log::info;

impl Server {
    pub fn handle_did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
        parser: &mut Parser,
    ) {
        let uri = params.text_document.uri.as_str();
        // We probably wont need to use this server on TCP
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];
        let path = PathBuf::from_str(path).unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        info!("Opened the file: {:?}", path.to_str().unwrap());

        let paths: Vec<&Path> = path.ancestors().collect();
        let piece_amt = paths.len();

        for (idx, path) in paths.into_iter().rev().enumerate() {
            let is_file = idx == piece_amt - 1;
            let path = path.to_str().unwrap();

            if is_file {
                if self.file_system.exists(path).unwrap() {
                    error!("Trying to create file that already exists: {}", path);
                    break;
                }

                let mut file = self.file_system.create_file(path).unwrap();
                file.write(path.as_bytes()).unwrap();

                let document = Document::new(
                    parser,
                    params.text_document.text.as_bytes().to_vec(),
                    file_name,
                );
                info!("New document: {:?}", document);
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
        params: DidChangeTextDocumentParams,
        parser: &mut Parser,
    ) {
        let uri = params.text_document.uri.as_str();
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];
        let file_name = path.split("/").last().unwrap().to_string();

        info!("Updated file: {:?}", path);

        let document = self.document_map.get_mut(path).unwrap();
        *document = Document::new(
            parser,
            params.content_changes[0].text.as_bytes().to_vec(),
            file_name
        );
    }
}
