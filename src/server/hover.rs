use serde::{Deserialize, Serialize};
use vfs::FileSystem;

use crate::{
    rpc::{RequestMessage, ResponseMessage, Rpc},
    server::{
        Server,
        document::Document,
        mod_api::{GrugArgument, ModApi},
        text_sync::{Position, Range, TextDocumentPositionParams},
        utils::get_spot_info,
    },
};

#[derive(Serialize, Deserialize, Debug)]
enum MarkupKind {
    PlainText,
    Markup,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkupContent {
    kind: MarkupKind,
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HoverResult {
    contents: MarkupContent,
    range: Range,
}

impl Server {
    fn send_null(id: serde_json::Value, rpc: &mut Rpc) {
        let res: ResponseMessage<serde_json::Value> =
            ResponseMessage::new(id, serde_json::Value::Null);
        let json = serde_json::to_string_pretty(&res).unwrap();

        rpc.send(json);
    }
    fn get_hover(
        mod_api: &ModApi,
        document: &Document,
        node: &tree_sitter::Node<'_>,
    ) -> Option<String> {
        let range = node.byte_range();
        if node.kind() == "identifier" {
            let name = &document.content[range];
            let spot_info = get_spot_info(document, node);
            for var in spot_info.variables.into_iter() {
                if var.name.as_bytes() == name {
                    return Some(format!("{}: {}", var.name, var.r#type.as_str()));
                }
            }

            let name = String::from_utf8(name.to_vec()).ok()?;

            if let Some(func) = mod_api.game_functions.get(&name) {
                let mut text = func.format(&name);
                text.push_str("\n\n");

                text.push_str(&func.description);

                return Some(text);
            }
        } else if node.kind() == "helper_identifier" {
            let name = &document.content[range];
            
            if let Some(helper) = document.helpers.iter().find(|helper| helper.name.as_bytes() == name) {
                return Some(helper.format());
            }
        } else if node.kind() == "on_identifier" {
            let name = String::from_utf8(document.content[range].to_vec()).ok()?;
            if let Some(entity) = mod_api.entities.get(&document.entity_type) {
                if let Some(on_func) = entity.on_functions.get(&name) {
                    let mut text = name;
                    text.push_str("\n\n");

                    text.push_str(&on_func.description);
                    return Some(text);
                }
            }
        }

        None
    }
    pub fn handle_hover(&self, req: RequestMessage<TextDocumentPositionParams>, rpc: &mut Rpc) {
        // We probably wont need to use this server on TCP
        assert!(req.params.text_document.uri.starts_with("file://"));

        let path = &req.params.text_document.uri["file.//".len()..];

        if !self.file_system.exists(path).unwrap_or(false) {
            Server::send_null(req.id, rpc);
            return;
        }

        let document = &self.document_map.get(path).unwrap();

        let ast = &document.tree;

        let point = tree_sitter::Point {
            column: req.params.position.character,
            row: req.params.position.line,
        };

        let node = ast
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let range = node.range();

        let content = Self::get_hover(&self.mod_api, document, &node);
        if content.is_none() {
            Server::send_null(req.id, rpc);
            return;
        }
        let mut content = content.unwrap().as_bytes().to_vec();

        let var = document
            .global_vars
            .iter()
            .find(|var| var.name.as_bytes() == &content);
        if let Some(var) = var {
            content = format!("{}: {}", var.name, var.r#type.as_str())
                .as_bytes()
                .to_vec();
        }

        let res: ResponseMessage<HoverResult> = ResponseMessage::new(
            req.id,
            HoverResult {
                contents: MarkupContent {
                    kind: MarkupKind::PlainText,
                    value: String::from_utf8(content).unwrap(),
                },
                range: Range {
                    start: Position {
                        line: range.start_point.row,
                        character: range.start_point.column,
                    },
                    end: Position {
                        line: range.end_point.row,
                        character: range.end_point.column,
                    },
                },
            },
        );
        let json = serde_json::to_string_pretty(&res).unwrap();

        rpc.send(json);
    }
}
