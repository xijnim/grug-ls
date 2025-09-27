use lsp_types::Position;
use tree_sitter::Node;

use crate::server::document::{Document, Variable, parser_utils};

#[derive(PartialEq, Eq, Debug)]
pub struct SpotInfo {
    pub variables: Vec<Variable>,
}

pub fn get_nearest_node<'a>(document: &'a Document, position: Position) -> Node<'a> {
    let point = tree_sitter::Point {
        column: position.character as usize,
        row: position.line as usize,
    };
    let maybe_nearest = document
        .tree
        .root_node()
        .descendant_for_point_range(point, point)
        .unwrap();

    let mut ret = maybe_nearest;

    let mut cursor = maybe_nearest.walk();
    for child in ret.children(&mut cursor) {
        let child_point = child.range().start_point;

        if child_point.row == point.row {
            if child_point.column > point.column {
                break;
            }
        } else if child_point.row > point.row {
            break;
        }

        ret = child;
    }

    ret
}

pub fn get_spot_info(document: &Document, node: &tree_sitter::Node) -> SpotInfo {
    let mut variables: Vec<Variable> = Vec::new();
    for global_var in document.global_vars.iter() {
        variables.push(global_var.clone());
    }

    let mut parent = node.clone();

    while let Some(next_parent) = parent.parent() {
        if next_parent.kind() == "source_file" {
            parent = next_parent;
            continue;
        }

        macro_rules! handle {
            ($node:expr) => {
                if $node.kind() == "variable_declaration" {
                    if let Ok(decl) =
                        parser_utils::parse_variable_declaration(&document.content, &$node)
                    {
                        variables.push(decl);
                    }
                }

                if $node.kind() == "function_parameter" {
                    if let Ok(param) =
                        parser_utils::parse_variable_declaration(&document.content, &$node)
                    {
                        variables.push(param);
                    }
                }
            };
        }

        let mut current_node = parent;
        handle!(current_node);
        while let Some(sibling) = current_node.prev_sibling() {
            handle!(sibling);

            current_node = sibling;
        }

        parent = next_parent;
    }

    SpotInfo { variables }
}

#[test]
pub fn test_var_get() {
    let source = r#"a: i32 = 2
b: f32 = 4.

on_spawn(str: string) {
    c: f32 = 6
    if true {
        no: i32 = 3
    }
    print()
    
    d: f32 = 5
}
"#;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_grug::LANGUAGE.into())
        .unwrap();

    let tree = parser.parse(source.as_bytes(), None).unwrap();
    let func_call = tree
        .root_node()
        .named_descendant_for_point_range(
            tree_sitter::Point { row: 8, column: 5 },
            tree_sitter::Point { row: 8, column: 11 },
        )
        .unwrap();

    assert!(func_call.kind() == "function_call");
    assert!(func_call.child_by_field_name("name").unwrap().kind() == "identifier");

    let document = Document::new(
        &mut parser,
        source.as_bytes().to_vec(),
        "tired-box.grug".to_string(),
    );
    assert_eq!(document.entity_type, "box");

    let spot_info = get_spot_info(&document, &func_call);

    use crate::server::document::Type;

    let names = ["a", "b", "c", "str"];
    let types = [Type::I32, Type::F32, Type::F32, Type::String];

    assert!(spot_info.variables.iter().any(|var| {
        names
            .iter()
            .zip(types.iter())
            .any(|(name, kind)| var.name == **name && var.r#type == *kind)
    }));
}

pub fn treesitter_range_to_lsp(range: &tree_sitter::Range) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            character: range.start_point.column as u32,
            line: range.start_point.row as u32,
        },
        end: lsp_types::Position {
            character: range.end_point.column as u32,
            line: range.end_point.row as u32,
        },
    }
}

pub fn is_function_call(node: &Node) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    parent.kind() == "function_call"
}
