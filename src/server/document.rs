use std::borrow::Borrow;

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub enum Type {
    I32,
    F32,
    ID,
    String,
}

impl Type {
    fn from_str<S: Borrow<str>>(s: S) -> Option<Type> {
        let s = s.borrow();

        match s {
            "f32" => Some(Type::F32),
            "i32" => Some(Type::I32),
            "string" => Some(Type::String),
            "id" => Some(Type::ID),
            _ => None
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Type::ID => "id",
            Type::String => "string",
            Type::I32 => "i32",
            Type::F32 => "332",
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub r#type: Type,
}

type Parameter = Variable;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ret_type: Option<Type>,
}

#[derive(Debug)]
pub struct Document {
    pub content: Vec<u8>,
    pub tree: tree_sitter::Tree,
    pub global_vars: Vec<Variable>,
    pub helpers: Vec<Function>,
}

pub mod parser_utils {
    use crate::server::document::{parser_utils, Type, Variable};

    pub fn node_get_content<'a>(content: &'a [u8], node: &tree_sitter::Node) -> &'a [u8] {
        let range = node.range();
        &content[range.start_byte..range.end_byte]
    }
    pub fn parse_variable_declaration(content: &[u8], node: &tree_sitter::Node) -> Option<Variable> {
        if node.kind() != "variable_declaration" && node.kind() != "function_parameter" {
            return None;
        }

        let name = node.child_by_field_name("name").unwrap();
        let kind = node.child_by_field_name("type").unwrap();

        let name = parser_utils::node_get_content(&content, &name);
        let kind = parser_utils::node_get_content(&content, &kind);

        let name = String::from_utf8(name.to_vec()).ok()?;
        let kind = String::from_utf8(kind.to_vec()).ok()?;

        let kind = Type::from_str(kind)?;
        
        return Some(Variable { name, r#type: kind })
    }
}

impl Document {
    pub fn new(parser: &mut tree_sitter::Parser, content: Vec<u8>) -> Document {
        let tree = parser.parse(&content, None).unwrap();

        let mut cursor = tree.root_node().walk();
        let global_vars: Vec<Variable> = tree
            .root_node()
            .children(&mut cursor)
            .filter(|child| child.kind() == "variable_declaration")
            .map(|child| {
                parser_utils::parse_variable_declaration(&content, &child)
            })
            .filter(|var| var.is_some())
            .map(|var| var.unwrap())
            .collect();

        cursor.reset(tree.root_node());
        let helpers: Vec<Function> = tree
            .root_node()
            .children(&mut cursor)
            .filter(|child| {
                if child.kind() == "function_declaration" {
                    let name = child.child_by_field_name("name").unwrap();

                    name.kind() == "helper_identifier"
                } else {
                    false
                }
            })
            .filter_map(|decl| {
                let name = decl.child_by_field_name("name").unwrap();
                let ret_type = decl.child_by_field_name("ret_type")
                    .map(|ret_type| {
                        let ret_type = parser_utils::node_get_content(&content, &ret_type);
                        let ret_type = String::from_utf8(ret_type.to_vec()).ok()?;

                        let ret_type = match ret_type.as_str() {
                            "f32" => Type::F32,
                            "i32" => Type::F32,
                            "id" => Type::ID,
                            "string" => Type::String,
                            _ => {
                                return None;
                            }
                        };

                        Some(ret_type)
                    })?;

                let name = parser_utils::node_get_content(&content, &name);
                let name = String::from_utf8(name.to_vec()).ok()?;

                let mut cursor = decl.walk();
                let params: Vec<Parameter> = decl.children_by_field_name("param", &mut cursor)
                    .find_map(|param| {
                        let name = param.child_by_field_name("name").unwrap();
                        let kind = param.child_by_field_name("type").unwrap();
                        
                        let name = parser_utils::node_get_content(&content, &name);
                        let kind = parser_utils::node_get_content(&content, &kind);
                        
                        let name = String::from_utf8(name.to_vec()).ok()?;
                        let kind = String::from_utf8(kind.to_vec()).ok()?;
                        let kind = Type::from_str(kind)?;
                        
                        Some(Parameter {
                            name,
                            r#type: kind,
                        })
                    })
                    .into_iter().collect();
                
                Some(Function {
                    name,
                    params,
                    ret_type,
                })
            })
            .collect();

        drop(cursor);
        Document { content, tree, global_vars, helpers }
    }
}
