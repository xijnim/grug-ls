use crate::server::document::{Document, Variable, parser_utils};

#[derive(PartialEq, Eq, Debug)]
pub struct SpotInfo {
    pub variables: Vec<Variable>,
}

pub fn get_spot_info(document: &Document, node: &tree_sitter::Node) -> SpotInfo {
    let mut variables: Vec<Variable> = Vec::new();
    for global_var in document.global_vars.iter() {
        variables.push(Variable {
            name: global_var.name.clone(),
            r#type: global_var.r#type.clone(),
        });
    }

    let break_node_idx = node.id();
    let mut parent = node.clone();

    while let Some(next_parent) = parent.parent() {
        let mut cursor = next_parent.walk();

        for child in parent.children(&mut cursor) {
            if child.id() == break_node_idx {
                break;
            }

            if child.kind() == "variable_declaration" {
                if let Ok(decl) =
                    parser_utils::parse_variable_declaration(&document.content, &child)
                {
                    variables.push(decl);
                }
            }

            if child.kind() == "function_parameter" {
                if let Ok(param) =
                    parser_utils::parse_variable_declaration(&document.content, &child)
                {
                    variables.push(param);
                }
            }
        }

        parent = next_parent;
    }

    SpotInfo { variables }
}

#[test]
fn test_var_get() {
    let source = r#"a: i32 = 2;
b: f32 = 4.;

on_spawn(str: string) {
    c: f32 = 6;
    if true {
        no: i32 = 3;
    }
    print()
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
    assert_eq!(
        spot_info,
        SpotInfo {
            variables: vec![
                Variable {
                    name: "a".to_string(),
                    r#type: Type::I32,
                },
                Variable {
                    name: "b".to_string(),
                    r#type: Type::F32,
                },
                Variable {
                    name: "c".to_string(),
                    r#type: Type::F32,
                },
                Variable {
                    name: "str".to_string(),
                    r#type: Type::String,
                },
            ],
        }
    );
}
