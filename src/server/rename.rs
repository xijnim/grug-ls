use std::collections::HashMap;

use lsp_server::{Connection, ErrorCode, Message, RequestId, Response};
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use tree_sitter::Node;
use vfs::FileSystem;

use crate::server::{
    Server,
    document::Document,
    utils::{get_spot_info, treesitter_range_to_lsp},
};

use log::info;

#[derive(PartialEq, Eq)]
enum RenameType {
    Variable,
    Function,
}
impl Server {
    fn rename_in_node(
        document: &Document,
        node: &Node,
        old_name: &str,
        new_name: &str,
        rename_type: &RenameType,
    ) -> Vec<TextEdit> {
        let mut edits: Vec<TextEdit> = Vec::new();

        match node.kind() {
            "while_statement" | "if_statement" => {
                let condition_node = node.child_by_field_name("condition").unwrap();
                edits.append(&mut Self::rename_in_node(
                    document,
                    &condition_node,
                    old_name,
                    new_name,
                    rename_type,
                ));

                let body_node = node.child_by_field_name("body").unwrap();
                edits.append(&mut Self::rename_in_node(
                    document,
                    &body_node,
                    old_name,
                    new_name,
                    rename_type,
                ));

                if let Some(else_node) = node.child_by_field_name("else") {
                    edits.append(&mut Self::rename_in_node(
                        document,
                        &else_node,
                        old_name,
                        new_name,
                        rename_type,
                    ))
                }
            }
            "source_file" | "body" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    edits.append(&mut Self::rename_in_node(
                        document,
                        &child,
                        old_name,
                        new_name,
                        rename_type,
                    ))
                }
            }
            "variable_declaration" => {
                let value_node = node.child_by_field_name("value").unwrap();
                edits.append(&mut Self::rename_in_node(
                    document,
                    &value_node,
                    old_name,
                    new_name,
                    rename_type,
                ))
            }
            "function_call" => {
                if *rename_type == RenameType::Function {
                    let name_node = node.child_by_field_name("name").unwrap();
                    let name = &document.content[name_node.byte_range()];

                    if name == old_name.as_bytes() {
                        let range = treesitter_range_to_lsp(&name_node.range());

                        edits.push(TextEdit::new(range, new_name.to_string()));
                    }
                }

                let mut cursor = node.walk();
                for argument in node.children_by_field_name("argument", &mut cursor) {
                    edits.append(&mut Self::rename_in_node(
                        document,
                        &argument,
                        old_name,
                        new_name,
                        rename_type,
                    ))
                }
            }
            "argument" => {
                edits.append(&mut Self::rename_in_node(
                    document,
                    &node.child(0).unwrap(),
                    old_name,
                    new_name,
                    rename_type,
                ));
            }
            "identifier" => {
                if *rename_type == RenameType::Variable {
                    let name = &document.content[node.byte_range()];

                    if name == old_name.as_bytes() {
                        let range = treesitter_range_to_lsp(&node.range());
                        edits.push(TextEdit::new(range, new_name.to_string()));
                    }
                }
            }
            "function_declaration" => {
                if *rename_type == RenameType::Function {
                    let name_node = node.child_by_field_name("name").unwrap();
                    let name = &document.content[name_node.byte_range()];

                    if name == old_name.as_bytes() {
                        let range = treesitter_range_to_lsp(&name_node.range());
                        edits.push(TextEdit::new(range, new_name.to_string()));
                    }
                }

                let body_node = node.child_by_field_name("body").unwrap();
                edits.append(&mut Self::rename_in_node(
                    document,
                    &body_node,
                    old_name,
                    new_name,
                    rename_type,
                ));
            }
            "binary_expression" => {
                let left_node = node.child_by_field_name("left").unwrap();
                let right_node = node.child_by_field_name("right").unwrap();

                edits.append(&mut Self::rename_in_node(
                    document,
                    &left_node,
                    old_name,
                    new_name,
                    rename_type,
                ));
                edits.append(&mut Self::rename_in_node(
                    document,
                    &right_node,
                    old_name,
                    new_name,
                    rename_type,
                ));
            }
            "unary_expression" => {
                let operand_node = node.child_by_field_name("operand").unwrap();

                edits.append(&mut Self::rename_in_node(
                    document,
                    &operand_node,
                    old_name,
                    new_name,
                    rename_type,
                ));
            }
            "return_statement" => {
                let value_node = node.child_by_field_name("value").unwrap();

                edits.append(&mut Self::rename_in_node(
                    document,
                    &value_node,
                    old_name,
                    new_name,
                    rename_type,
                ));
            }

            _ => {
                info!("Can't rename: {:?}", node);
            }
        }

        edits
    }

    fn rename_var(
        document: &Document,
        node: &Node,
        old_name: &str,
        new_name: &str,
    ) -> Vec<lsp_types::TextEdit> {
        if !matches!(node.kind(), "variable_declaration" | "function_parameter") {
            panic!("{}", node.kind());
        }

        let mut edits: Vec<TextEdit> = Vec::new();

        let mut node = node.clone();

        let range = treesitter_range_to_lsp(&node.child_by_field_name("name").unwrap().range());
        edits.push(TextEdit::new(range, new_name.to_string()));

        while let Some(sibling) = node.next_sibling() {
            let mut new_edits = Self::rename_in_node(
                document,
                &sibling,
                old_name,
                new_name,
                &RenameType::Variable,
            );
            edits.append(&mut new_edits);

            node = sibling;
        }

        edits
    }

    fn rename_helper(
        document: &Document,
        node: &Node,
        old_name: &str,
        new_name: &str,
    ) -> Vec<TextEdit> {
        let mut edits: Vec<TextEdit> = Vec::new();

        assert_eq!(node.kind(), "source_file");

        edits.append(&mut Self::rename_in_node(
            document,
            &node,
            old_name,
            new_name,
            &RenameType::Function,
        ));

        edits
    }

    pub fn rename(&self, params: RenameParams, connection: &mut Connection, id: RequestId) {
        let uri = params.text_document_position.text_document.uri.as_str();

        // We probably wont need to use this server on TCP
        assert!(uri.starts_with("file://"));

        let path = &uri["file.//".len()..];

        macro_rules! send_err {
            ($($arg:tt)*) => {
                connection
                    .sender
                    .send(Message::Response(Response::new_err(
                        id,
                        ErrorCode::InvalidRequest as i32,
                        format!($($arg)*),
                    )))
                    .unwrap()
            };
        }

        if !self.file_system.exists(path).unwrap_or(false) {
            send_err!("File doesnt exist: {}", path);
            return;
        }

        let document = self.document_map.get(path).unwrap();

        let point = tree_sitter::Point {
            column: params.text_document_position.position.character as usize,
            row: params.text_document_position.position.line as usize,
        };

        let node = document
            .tree
            .root_node()
            .descendant_for_point_range(point, point)
            .unwrap();
        let name = &document.content[node.byte_range()];
        let node_kind = node.kind();

        let spot_info = get_spot_info(document, &node);

        let edits: Option<WorkspaceEdit> = if node_kind != "identifier"
            && node_kind != "on_identifier"
            && node_kind != "helper_identifier"
        {
            None
        } else {
            if let Some(var) = spot_info
                .variables
                .iter()
                .find(|var| var.name.as_bytes() == name)
            {
                info!("Renaming variable {} to {}", var.name, params.new_name);
                let node = document
                    .tree
                    .root_node()
                    .descendant_for_byte_range(var.range.start_byte, var.range.end_byte)
                    .unwrap();
                let edits = Self::rename_var(document, &node, &var.name, &params.new_name);

                Some(WorkspaceEdit::new(HashMap::from([(
                    document.uri.clone(),
                    edits,
                )])))
            } else if let Some(func) = document
                .helpers
                .iter()
                .find(|func| func.name.as_bytes() == name)
            {
                info!("Renaming helper {} to {}", func.name, params.new_name);
                let node = document.tree.root_node();
                let edits = Self::rename_helper(document, &node, &func.name, &params.new_name);

                Some(WorkspaceEdit::new(HashMap::from([(
                    document.uri.clone(),
                    edits,
                )])))
            } else {
                None
            }
        };
        info!("{:?}", edits);

        connection
            .sender
            .send(Message::Response(Response::new_ok(id, edits)))
            .unwrap()
    }
}
