use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{DocumentFormattingParams, TextEdit};
use tree_sitter::Node;

use crate::server::{Server, utils::treesitter_range_to_lsp};

impl Server {
    fn format_node(content: &[u8], node: &Node) -> Vec<String> {
        let lines: Vec<String> = match node.kind() {
            "variable_declaration" => {
                let name_node = node.child_by_field_name("name").unwrap();
                let name = Self::format_node(content, &name_node);
                assert_eq!(name.len(), 1);
                let name = &name[0];

                let type_node = node.child_by_field_name("type").unwrap();
                let type_name = Self::format_node(content, &type_node);
                assert_eq!(type_name.len(), 1);
                let type_name = &type_name[0];

                let mut text = format!("{}: {}", name, type_name);

                if let Some(value_text) = node
                    .child_by_field_name("value")
                    .map(|node| Self::format_node(content, &node))
                {
                    assert_eq!(value_text.len(), 1);
                    text.push_str(&format!(" = {}", value_text[0]));
                }

                vec![text]
            }
            "function_call" => {
                let function_name =
                    Self::format_node(content, &node.child_by_field_name("name").unwrap());
                assert_eq!(function_name.len(), 1);

                let mut text = format!("{}(", function_name[0]);

                let mut cursor = node.walk();
                let params = node.children_by_field_name("argument", &mut cursor);
                let param_amt: usize = params.count();

                let mut cursor = node.walk();
                let params = node.children_by_field_name("argument", &mut cursor);
                for (idx, param) in params.enumerate() {
                    let param = Self::format_node(content, &param);
                    assert_eq!(param.len(), 1);
                    let param = &param[0];

                    text.push_str(param);
                    if idx < param_amt - 1 {
                        text.push(',');
                        text.push(' ');
                    }
                }

                text.push(')');
                vec![text]
            }
            "identifier" | "number" | "type" | "on_identifier" | "helper_identifier"
            | "comment" | "me" | "+" | "-" | "*" | "/" | "string" | "not" | "empty_return"
            | "<" | ">" | "<=" | ">=" | "==" | "!=" | "and" | "or" | "boolean" => {
                let text_bytes = &content[node.byte_range()];

                let text: String = text_bytes
                    .into_iter()
                    .filter_map(|c| match c {
                        b'\n' => None,
                        c => Some(*c as char),
                    })
                    .collect();

                vec![text]
            }
            "argument" => Self::format_node(content, &node.child(0).unwrap()),
            "binary_expression" => {
                let left =
                    &Self::format_node(content, &node.child_by_field_name("left").unwrap())[0];
                let right =
                    &Self::format_node(content, &node.child_by_field_name("right").unwrap())[0];
                let operator =
                    &Self::format_node(content, &node.child_by_field_name("operator").unwrap())[0];

                let text = format!("{} {} {}", left, operator, right);

                vec![text]
            }
            "unary_expression" => {
                let operand =
                    &Self::format_node(content, &node.child_by_field_name("operand").unwrap())[0];
                let operator =
                    &Self::format_node(content, &node.child_by_field_name("operator").unwrap())[0];

                let operator = if operator == "not" { "not " } else { operator };

                vec![format!("{}{}", operator, operand)]
            }
            "contained_expression" => {
                let expr =
                    &Self::format_node(content, &node.child(1).unwrap())[0];

                vec![format!("({})", expr)]
            }
            "assignment" => {
                let name =
                    &Self::format_node(content, &node.child_by_field_name("name").unwrap())[0];

                let value =
                    &Self::format_node(content, &node.child_by_field_name("value").unwrap())[0];

                vec![format!("{} = {}", name, value)]
            }
            "return_statement" => {
                let value =
                    &Self::format_node(content, &node.child_by_field_name("value").unwrap())[0];

                vec![format!("return {}", value)]
            }
            "if_statement" | "while_statement" => {
                let keyword = if node.kind() == "if_statement" {
                    "if"
                } else {
                    "while"
                };
                let condition =
                    Self::format_node(content, &node.child_by_field_name("condition").unwrap());

                assert_eq!(condition.len(), 1);
                let condition = &condition[0];

                let text = format!("{} {}", keyword, condition);

                let mut lines: Vec<String> = Vec::new();
                let mut body =
                    Self::format_node(content, &node.child_by_field_name("body").unwrap());
                body[0] = format!("{} {}", text, body[0]);
                lines.append(&mut body);

                if let Some(else_node) = node.child_by_field_name("else") {
                    let mut else_text = Self::format_node(content, &else_node);

                    let last_line = lines.last_mut().unwrap();
                    *last_line = format!("{} else {}", *last_line, else_text.remove(0));

                    lines.append(&mut else_text);
                }

                lines
            }
            "body" => {
                let mut lines: Vec<String> = Vec::new();

                lines.push("{".to_string());

                let mut cursor = node.walk();

                let children: Vec<Node> = node
                    .children(&mut cursor)
                    .filter(|node| !matches!(node.kind(), "{" | "}"))
                    .collect();

                let mut stmt_lines: Vec<String> = Vec::new();
                let content_str = String::from_utf8(content.to_vec()).unwrap();
                let mut current_child: usize = 0;

                let content_lines: Vec<&str> = content_str.lines().into_iter().collect();
                let line_amt = content_lines.len();
                let mut line_idx: usize = 0;
                let mut can_push_line = false;
                while line_idx < line_amt && current_child < children.len() {
                    let mut child = &children[current_child];
                    if line_idx >= child.start_position().row {
                        let new_line = Self::format_node(content, child);
                        let mut new_line: Vec<String> = new_line
                            .into_iter()
                            .map(|line| format!("    {}", line))
                            .collect();
                        stmt_lines.append(&mut new_line);

                        current_child += 1;
                        can_push_line = true;

                        if let Some(next_child) = children.get(current_child) {
                            child = next_child;
                        } else {
                            break;
                        }
                    }

                    if content_lines[line_idx]
                        .chars()
                        .all(|c| matches!(c, ' ' | '\t'))
                        && can_push_line
                    {
                        stmt_lines.push("".to_string());
                        can_push_line = false;
                    }

                    if line_idx < child.start_position().row {
                        line_idx += 1;
                    }
                }

                if children.len() == 0 {
                    stmt_lines.push("".to_string());
                }

                lines.append(&mut stmt_lines);

                lines.push("}".to_string());

                lines
            }
            "source_file" => {
                let mut lines: Vec<String> = Vec::new();

                let mut cursor = node.walk();

                let children: Vec<Node> = node
                    .children(&mut cursor)
                    .filter(|node| !matches!(node.kind(), "{" | "}"))
                    .collect();

                let mut stmt_lines: Vec<String> = Vec::new();
                let content_str = String::from_utf8(content.to_vec()).unwrap();
                let mut current_child: usize = 0;

                let content_lines: Vec<&str> = content_str.lines().into_iter().collect();
                let line_amt = content_lines.len();
                let mut line_idx: usize = 0;
                let mut can_push_line = false;
                while line_idx < line_amt && current_child < children.len() {
                    let mut child = &children[current_child];
                    if line_idx >= child.start_position().row {
                        let mut new_line = Self::format_node(content, child);
                        stmt_lines.append(&mut new_line);

                        current_child += 1;
                        can_push_line = true;

                        if let Some(next_child) = children.get(current_child) {
                            child = next_child;
                        } else {
                            break;
                        }
                    }

                    if content_lines[line_idx]
                        .chars()
                        .all(|c| matches!(c, ' ' | '\t'))
                        && can_push_line
                    {
                        stmt_lines.push("".to_string());
                        can_push_line = false;
                    }

                    if line_idx < child.start_position().row {
                        line_idx += 1;
                    }
                }

                lines.append(&mut stmt_lines);

                lines
            }
            "function_declaration" => {
                let name =
                    &Self::format_node(content, &node.child_by_field_name("name").unwrap())[0];

                let body = Self::format_node(content, &node.child_by_field_name("body").unwrap());

                let mut decl_line = format!("{}(", name);

                let mut cursor = node.walk();
                let params = node.children_by_field_name("param", &mut cursor);
                let param_amt: usize = params.count();

                let mut cursor = node.walk();
                let params = node.children_by_field_name("param", &mut cursor);
                for (idx, param) in params.enumerate() {
                    if param.kind() == "," {
                        continue;
                    }
                    let param = &Self::format_node(content, &param)[0];
                    decl_line.push_str(param);

                    if idx < param_amt - 1 {
                        decl_line.push(',');
                        decl_line.push(' ');
                    }
                }

                decl_line.push(')');

                if let Some(ret_node) = node.child_by_field_name("ret_type") {
                    let ret_type = &Self::format_node(content, &ret_node)[0];
                    decl_line.push_str(&format!(" {}", ret_type));
                }

                let mut lines: Vec<String> = body;
                lines[0] = format!("{} {}", decl_line, lines[0]);

                lines
            }
            "function_parameter" => {
                let name =
                    &Self::format_node(content, &node.child_by_field_name("name").unwrap())[0];
                let param_type =
                    &Self::format_node(content, &node.child_by_field_name("type").unwrap())[0];

                vec![format!("{}: {}", name, param_type)]
            }

            _ => {
                log::error!("Cannot format node: {:?}", node);
                Vec::new()
            }
        };

        lines
    }

    pub fn formatting(
        &self,
        params: DocumentFormattingParams,
        connection: &mut Connection,
        id: RequestId,
    ) {
        let uri = params.text_document.uri;
        let document = self.get_document_by_uri(&uri).unwrap();

        let range = document.tree.root_node().range();
        let range = treesitter_range_to_lsp(&range);

        let new_lines: Vec<String> =
            Self::format_node(&document.content, &document.tree.root_node());

        let string = new_lines.join("\n");
        let edit = TextEdit::new(range, string);

        let message = Message::Response(Response::new_ok(id, vec![edit]));
        connection.sender.send(message).unwrap();
    }
}
