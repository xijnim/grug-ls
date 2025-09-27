use std::{borrow::Borrow, collections::HashMap};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PRIMITIVE_TYPES: HashMap<&'static str, &'static str> = HashMap::from([
        ("resource", "A resource, such as an image or audio file"),
        ("f32", "A 32 bit floating point number"),
        ("i32", "A 32 bit integer"),
        (
            "id",
            "An opaque type. It can represent anything external to the language."
        ),
        (
            "entity",
            "Holds names of types of entities (e.g. modname:entityname)"
        ),
        ("string", "Represents text"),
    ]);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    I32,
    F32,
    ID,
    Bool,
    String,
    Resource,
    Entity(String),
}

impl Type {
    fn from_str<S: Borrow<str>>(s: S) -> Type {
        let s = s.borrow();

        match s {
            "f32" => Type::F32,
            "i32" => Type::I32,
            "string" => Type::String,
            "bool" => Type::Bool,
            "id" => Type::ID,
            "resource" => Type::Resource,
            _ => Type::Entity(s.to_string()),
        }
    }

    pub fn as_str<'a>(&'a self) -> &'a str {
        match self {
            Type::ID => "id",
            Type::Bool => "bool",
            Type::String => "string",
            Type::I32 => "i32",
            Type::F32 => "f32",
            Type::Resource => "resource",
            Type::Entity(s) => s.as_str(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub r#type: Type,

    pub range: tree_sitter::Range,
}

impl Variable {
    pub fn format(&self) -> String {
        format!("{}: {}", self.name, self.r#type.as_str())
    }
}

type Parameter = Variable;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ret_type: Option<Type>,
    pub range: tree_sitter::Range,
}

impl Function {
    pub fn format(&self) -> String {
        let mut out = format!("{}(", self.name);

        for (idx, param) in self.params.iter().enumerate() {
            out.push_str(&param.name);
            out.push_str(": ");
            out.push_str(&param.r#type.as_str());

            if idx < self.params.len() - 1 {
                out.push_str(", ");
            }
        }

        out.push(')');

        if let Some(ret_type) = &self.ret_type {
            out.push(' ');
            out.push_str(ret_type.as_str());
        }

        out
    }
}

#[derive(Debug)]
pub struct Document {
    pub entity_type: String,
    pub content: Vec<u8>,
    pub tree: tree_sitter::Tree,
    pub global_vars: Vec<Variable>,
    pub helpers: Vec<Function>,
    pub on_functions: Vec<Function>,
}

pub mod parser_utils {
    use std::string::FromUtf8Error;

    use crate::server::document::{Type, Variable, parser_utils};

    pub fn node_get_content<'a>(content: &'a [u8], node: &tree_sitter::Node) -> &'a [u8] {
        let range = node.range();
        &content[range.start_byte..range.end_byte]
    }

    #[derive(Debug)]
    pub enum ParseVariableDeclarationErr {
        #[allow(dead_code)]
        StringParseError(FromUtf8Error),
        #[allow(dead_code)]
        UnsupportedNodeType(&'static str),
    }
    pub fn parse_variable_declaration(
        content: &[u8],
        node: &tree_sitter::Node,
    ) -> Result<Variable, ParseVariableDeclarationErr> {
        if node.kind() != "variable_declaration" && node.kind() != "function_parameter" {
            return Err(ParseVariableDeclarationErr::UnsupportedNodeType(
                node.kind(),
            ));
        }

        let name = node.child_by_field_name("name").unwrap();
        let kind = node.child_by_field_name("type").unwrap();

        let name = parser_utils::node_get_content(&content, &name);
        let kind = parser_utils::node_get_content(&content, &kind);

        let name = String::from_utf8(name.to_vec())
            .map_err(|err| ParseVariableDeclarationErr::StringParseError(err))?;
        let kind = String::from_utf8(kind.to_vec())
            .map_err(|err| ParseVariableDeclarationErr::StringParseError(err))?;

        let kind = Type::from_str(kind);

        return Ok(Variable {
            name,
            r#type: kind,
            range: node.range(),
        });
    }
}

impl Document {
    pub fn new(parser: &mut tree_sitter::Parser, content: Vec<u8>, name: String) -> Document {
        let tree = parser.parse(&content, None).unwrap();

        let mut cursor = tree.root_node().walk();
        let global_vars: Vec<Variable> = tree
            .root_node()
            .children(&mut cursor)
            .filter(|child| child.kind() == "variable_declaration")
            .map(|child| parser_utils::parse_variable_declaration(&content, &child))
            .filter(|var| var.is_ok())
            .map(|var| var.unwrap())
            .collect();

        cursor.reset(tree.root_node());

        macro_rules! parse_functions {
            ($tree:ident, $node_type:expr) => {
                $tree
                    .root_node()
                    .children(&mut cursor)
                    .filter(|child| {
                        if child.kind() == "function_declaration" {
                            let name = child.child_by_field_name("name").unwrap();

                            name.kind() == $node_type
                        } else {
                            false
                        }
                    })
                    .filter_map(|decl| {
                        let name = decl.child_by_field_name("name").unwrap();
                        let ret_type = decl
                            .child_by_field_name("ret_type")
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
                            })
                            .flatten();

                        let name = parser_utils::node_get_content(&content, &name);
                        let name = String::from_utf8(name.to_vec()).ok()?;

                        let mut cursor = decl.walk();
                        let params: Vec<Parameter> = decl
                            .children_by_field_name("param", &mut cursor)
                            .find_map(|param| {
                                let name = param.child_by_field_name("name").unwrap();
                                let kind = param.child_by_field_name("type").unwrap();

                                let name = parser_utils::node_get_content(&content, &name);
                                let kind = parser_utils::node_get_content(&content, &kind);

                                let name = String::from_utf8(name.to_vec()).ok()?;
                                let kind = String::from_utf8(kind.to_vec()).ok()?;
                                let kind = Type::from_str(kind);

                                Some(Parameter {
                                    name,
                                    r#type: kind,
                                    range: param.range(),
                                })
                            })
                            .into_iter()
                            .collect();

                        Some(Function {
                            name,
                            params,
                            ret_type,
                            range: decl.range(),
                        })
                    })
                    .collect()
            };
        }

        let entity_type = name
            .split('-')
            .last()
            .unwrap()
            .strip_suffix(".grug")
            .unwrap();

        let helpers: Vec<Function> = parse_functions!(tree, "helper_identifier");
        let on_functions: Vec<Function> = parse_functions!(tree, "on_identifier");

        drop(cursor);
        Document {
            content,
            tree,
            global_vars,
            helpers,
            on_functions,
            entity_type: entity_type.to_string(),
        }
    }
}
