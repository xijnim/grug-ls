use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Position, Range};
use vfs::FileSystem;

use crate::server::{
    document::{Document, PRIMITIVE_TYPES}, mod_api::ModApi, utils::get_spot_info, Server
};

struct HoverContent {
    code: String,
    text: String,
}

impl HoverContent {
    pub fn new_code_only(code: String) -> HoverContent {
        HoverContent { code, text: "".to_string() }
    }
}

impl Server {
    fn get_hover(
        mod_api: &ModApi,
        document: &Document,
        node: &tree_sitter::Node<'_>,
    ) -> Option<HoverContent> {
        let range = node.byte_range();
        if node.kind() == "identifier" {
            let name = &document.content[range];
            let spot_info = get_spot_info(document, node);
            for var in spot_info.variables.into_iter() {
                if var.name.as_bytes() == name {
                    let code = format!("{}: {}", var.name, var.r#type.as_str());
                    return Some(HoverContent::new_code_only(code));
                }
            }

            let name = String::from_utf8(name.to_vec()).ok()?;

            if let Some(func) = mod_api.game_functions.get(&name) {
                let declaration = func.format(&name);

                return Some(HoverContent {
                    code: declaration,
                    text: func.description.to_string(),
                });
            }

        } else if node.kind() == "helper_identifier" {
            let name = &document.content[range];
            
            if let Some(helper) = document.helpers.iter().find(|helper| helper.name.as_bytes() == name) {
                return Some(HoverContent::new_code_only(helper.format()));
            }
        } else if node.kind() == "on_identifier" {
            let name = String::from_utf8(document.content[range].to_vec()).ok()?;
            if let Some(entity) = mod_api.entities.get(&document.entity_type) {
                if let Some(on_func) = entity.on_functions.get(&name) {
                    return Some(HoverContent {
                        code: name,
                        text: on_func.description.to_string(),
                    });
                }
            }
        } else if node.kind() == "type" {
            let name = String::from_utf8(document.content[range].to_vec()).ok()?;
            if let Some(desc) = PRIMITIVE_TYPES.get(name.as_str()) {
                return Some(HoverContent {
                    code: name,
                    text: desc.to_string(),
                });
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

        let content = content.unwrap();
        let mut hover_text = String::new();

        if !content.code.is_empty() {
            hover_text = format!("```rust\n{}\n```", content.code);
        }
        if !content.text.is_empty() {
            if !content.code.is_empty() {
                hover_text.push('\n');
            }
            hover_text.push_str(&content.text);
        }

        let res = Response::new_ok(id, Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: String::from_utf8(hover_text.as_bytes().to_vec()).unwrap(),
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
