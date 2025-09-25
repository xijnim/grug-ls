use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position, Range};
use vfs::FileSystem;

use crate::server::{
        document::{Document, PRIMITIVE_TYPES}, mod_api::ModApi, utils::get_spot_info, Server
    };

impl Server {
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
        } else if node.kind() == "type" {
            let name = String::from_utf8(document.content[range].to_vec()).ok()?;
            if let Some(desc) = PRIMITIVE_TYPES.get(name.as_str()) {
                return Some(format!("{}\n\n{}", name, desc));
            }
        }

        None
    }
    pub fn handle_hover(&self, params: HoverParams, connection: &mut Connection, id: RequestId) {
        let uri = params.text_document_position_params.text_document.uri.as_str();
        
        // We probably wont need to use this server on TCP
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];

        if !self.file_system.exists(path).unwrap_or(false) {
            connection.sender.send(Message::Response(Response::new_err(id, ErrorCode::InvalidRequest as i32, format!("File doesnt exist: {}", path)))).unwrap();
            return;
        }

        let document = &self.document_map.get(path).unwrap();

        let ast = &document.tree;

        let point = tree_sitter::Point {
            column: params.text_document_position_params.position.character as usize,
            row: params.text_document_position_params.position.line as usize,
        };

        let node = ast
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let range = node.range();

        let content = Self::get_hover(&self.mod_api, document, &node);
        if content.is_none() {
            connection.sender.send(Message::Response(Response::new_ok(id, serde_json::Value::Null))).unwrap();
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

        let res = Response::new_ok(id, Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: String::from_utf8(content).unwrap(),
            }),
            range: Some(Range {
                start: Position {
                    line: range.start_point.row as u32,
                    character: range.start_point.column as u32,
                },
                end: Position {
                    line: range.end_point.row as u32,
                    character: range.end_point.column as u32,
                },
            }),
        });

        connection.sender.send(Message::Response(res)).unwrap();
    }
}
