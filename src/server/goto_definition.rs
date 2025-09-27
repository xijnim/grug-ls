use std::str::FromStr;

use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, LocationLink, Uri};
use vfs::FileSystem;

use crate::server::{
    Server,
    document::Document,
    utils::{get_spot_info, is_function_call, treesitter_range_to_lsp},
};

use log::info;

impl Server {
    fn get_definition(
        &self,
        uri: String,
        document: &Document,
        node: &tree_sitter::Node<'_>,
    ) -> Option<GotoDefinitionResponse> {
        let uri = Uri::from_str(&uri).ok()?;
        let spot_info = get_spot_info(document, node);
        let text = String::from_utf8(document.content[node.byte_range()].to_vec()).ok()?;
        info!("Trying to get definition for: {}", node.kind());
        if node.kind() == "identifier" {
            if !is_function_call(&node) {
                if let Some(var) = spot_info.variables.iter().find(|var| var.name == text) {
                    let node = document
                        .tree
                        .root_node()
                        .descendant_for_byte_range(var.range.start_byte, var.range.end_byte)
                        .unwrap();
                    let link = LocationLink {
                        target_uri: uri,
                        target_range: treesitter_range_to_lsp(&node.range()),
                        target_selection_range: treesitter_range_to_lsp(
                            &node.child_by_field_name("name").unwrap().range(),
                        ),
                        origin_selection_range: None,
                    };
                    return Some(GotoDefinitionResponse::Link(vec![link]));
                }
            }

            if let Some(entity) = self.mod_api.entities.get(&text) {
                let link = LocationLink {
                    target_uri: Uri::from_str(&format!(
                        "file://{}",
                        self.root_path.join("mod_api.json").to_str().unwrap()
                    ))
                    .unwrap(),
                    target_range: treesitter_range_to_lsp(&entity.range),
                    // Store the name key for the entity
                    target_selection_range: treesitter_range_to_lsp(&entity.range),
                    origin_selection_range: None,
                };
                return Some(GotoDefinitionResponse::Link(vec![link]));
            }

            if let Some(func) = self.mod_api.game_functions.get(&text) {
                let link = LocationLink {
                    target_uri: Uri::from_str(&format!(
                        "file://{}",
                        self.root_path.join("mod_api.json").to_str().unwrap()
                    ))
                    .unwrap(),
                    target_range: treesitter_range_to_lsp(&func.range),
                    target_selection_range: treesitter_range_to_lsp(&func.range),
                    origin_selection_range: None,
                };
                return Some(GotoDefinitionResponse::Link(vec![link]));
            }
        }

        if node.kind() == "helper_identifier" {
            if let Some(helper) = document.helpers.iter().find(|func| func.name == text) {
                let node = document
                    .tree
                    .root_node()
                    .descendant_for_byte_range(helper.range.start_byte, helper.range.end_byte)
                    .unwrap();
                let link = LocationLink {
                    target_uri: uri,
                    target_range: treesitter_range_to_lsp(&node.range()),
                    target_selection_range: treesitter_range_to_lsp(
                        &node.child_by_field_name("name").unwrap().range(),
                    ),
                    origin_selection_range: None,
                };
                return Some(GotoDefinitionResponse::Link(vec![link]));
            }
        }
        if node.kind() == "on_identifier" {
            if let Some(entity) = self.mod_api.entities.get(&document.entity_type) {
                if let Some(on_func) = entity.on_functions.get(&text) {
                    let link = LocationLink {
                        target_uri: Uri::from_str(&format!(
                            "file://{}",
                            self.root_path.join("mod_api.json").to_str().unwrap()
                        ))
                        .unwrap(),
                        target_range: treesitter_range_to_lsp(&on_func.range),
                        target_selection_range: treesitter_range_to_lsp(&on_func.range),
                        origin_selection_range: None,
                    };
                    return Some(GotoDefinitionResponse::Link(vec![link]));
                }
            }
        }
        None
    }
    pub fn handle_goto_definition(
        &self,
        params: GotoDefinitionParams,
        connection: &mut Connection,
        id: RequestId,
    ) {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .as_str();

        // We probably wont need to use this server on TCP
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];

        if !self.file_system.exists(path).unwrap_or(false) {
            connection
                .sender
                .send(Message::Response(Response::new_err(
                    id,
                    ErrorCode::InvalidRequest as i32,
                    format!("File doesnt exist: {}", path),
                )))
                .unwrap();
            return;
        }

        let document = &self.document_map.get(path).unwrap();

        let point = tree_sitter::Point {
            column: params.text_document_position_params.position.character as usize,
            row: params.text_document_position_params.position.line as usize,
        };

        let node = document
            .tree
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let definition = self.get_definition(uri.to_string(), document, &node);

        if let Some(definition) = definition {
            connection
                .sender
                .send(Message::Response(Response::new_ok(id, definition)))
                .unwrap();
        } else {
            connection
                .sender
                .send(Message::Response(Response::new_ok(
                    id,
                    serde_json::Value::Null,
                )))
                .unwrap();
        }
    }
}
