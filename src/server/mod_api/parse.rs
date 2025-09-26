use std::collections::HashMap;

use serde::{de::DeserializeOwned};
use tree_sitter::Node;

use crate::server::mod_api::{GrugEntity, GrugGameFunction, GrugOnFunction, JSON_PARSER, ModApi};

trait ModApiEntry {
    fn set_range(&mut self, range: tree_sitter::Range);
}

impl ModApiEntry for GrugEntity {
    fn set_range(&mut self, range: tree_sitter::Range) {
        self.range = range;
    }
}

impl ModApiEntry for GrugGameFunction {
    fn set_range(&mut self, range: tree_sitter::Range) {
        self.range = range;
    }
}

impl ModApi {
    fn parse_game_functions(
        out: &mut HashMap<String, GrugGameFunction>,
        entry: &Node,
        json: &[u8],
    ) {
        if entry.kind() != "object" {
            return;
        }

        let mut entity_cursor = entry.walk();
        for func_entry in entry.children(&mut entity_cursor) {
            if func_entry.kind() != "pair" {
                continue;
            }

            let Some(key) = func_entry.child_by_field_name("key") else {
                continue;
            };
            if key.kind() != "string" {
                continue;
            }
            let Some(key) = key.child(1) else {
                continue;
            };
            let Ok(key) = String::from_utf8(json[key.byte_range()].to_vec()) else {
                continue;
            };
            let Some(obj) = func_entry.child_by_field_name("value") else {
                continue;
            };

            let buf = &json[obj.byte_range()];
            let Ok(mut game_func) = serde_json::from_slice::<GrugGameFunction>(&buf) else {
                return;
            };

            game_func.range = func_entry.range();
            out.insert(key, game_func);
        }
    }

    fn parse_entity(node: &Node, json: &[u8]) -> Option<GrugEntity> {
        assert_eq!(node.kind(), "object");
        let mut description = "<NO DESCRIPTION>".to_string();
        let mut on_functions: HashMap<String, GrugOnFunction> = HashMap::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "pair" {
                continue;
            }
            let key = child.child_by_field_name("key").unwrap();
            let key = key.child(1).unwrap();
            let key = &json[key.byte_range()];

            let obj = child.child_by_field_name("value").unwrap();

            match key {
                b"description" => {
                    if obj.kind() != "string" {
                        continue;
                    }
                    let desc = obj.child(1).unwrap();
                    if let Ok(desc) =
                        String::from_utf8(json[desc.byte_range()].to_vec())
                    {
                        description = desc;
                    }
                }
                b"on_functions" => {
                    if obj.kind() != "object" {
                        continue;
                    }

                    let mut func_cursor = obj.walk();
                    for func_entry in obj.children(&mut func_cursor) {
                        let Some(func_name) = func_entry.child_by_field_name("key") else {
                            continue;
                        };
                        if func_name.kind() != "string" {
                            continue;
                        }
                        let func_name = func_name.child(1).unwrap();
                        let Ok(func_name) =
                            String::from_utf8(json[func_name.byte_range()].to_vec())
                        else {
                            continue;
                        };

                        let Some(obj) = func_entry.child_by_field_name("value") else {
                            continue;
                        };

                        if obj.kind() != "object" {
                            continue;
                        }

                        let obj = &json[obj.byte_range()];
                        let Ok(mut on_function) = serde_json::from_slice::<GrugOnFunction>(obj) else {
                            continue;
                        };
                        on_function.range = func_entry.range();

                        on_functions.insert(func_name, on_function);
                    }
                }
                _ => continue,
            }
        }

        Some(GrugEntity {
            description,
            on_functions,
            range: node.range(),
        })
    }
    fn parse_entities(out: &mut HashMap<String, GrugEntity>, entry: &Node, json: &[u8]) {
        if entry.kind() != "object" {
            return;
        }

        let mut entity_cursor = entry.walk();
        for entity_entry in entry.children(&mut entity_cursor) {
            if entity_entry.kind() != "pair" {
                continue;
            }

            let Some(key) = entity_entry.child_by_field_name("key") else {
                continue;
            };
            if key.kind() != "string" {
                continue;
            }
            let Some(key) = key.child(1) else {
                continue;
            };
            let Ok(key) = String::from_utf8(json[key.byte_range()].to_vec()) else {
                continue;
            };
            let Some(obj) = entity_entry.child_by_field_name("value") else {
                continue;
            };

            let Some(entity) = Self::parse_entity(&obj, json) else {
                continue;
            };
            out.insert(key, entity);
        }
    }

    pub fn from_json(json: &str) -> Option<ModApi> {
        let json = json.as_bytes();

        let mut parser = JSON_PARSER.lock().unwrap();
        let tree = parser.parse(json, None)?;

        let mut entities: HashMap<String, GrugEntity> = HashMap::new();
        let mut game_functions: HashMap<String, GrugGameFunction> = HashMap::new();

        let root = tree.root_node();
        let root = root.child(0)?;
        if root.kind() != "object" {
            return None;
        }

        let mut cursor = root.walk();

        for entry in root.children(&mut cursor) {
            if entry.kind() != "pair" {
                continue;
            }

            let key = if let Some(key) = entry.child_by_field_name("key") {
                key
            } else {
                continue;
            };

            let key = &json[key.byte_range()];
            match key {
                b"\"entities\"" => {
                    let Some(value) = entry.child_by_field_name("value") else {
                        continue;
                    };
                    if value.kind() != "object" {
                        continue;
                    }

                    Self::parse_entities(&mut entities, &value, json);
                }
                b"\"game_functions\"" => {
                    let Some(value) = entry.child_by_field_name("value") else {
                        continue;
                    };
                    if value.kind() != "object" {
                        continue;
                    }

                    Self::parse_game_functions(&mut game_functions, &value, json);
                }
                _ => {
                    println!("Unkown key: {:?}", String::from_utf8(key.to_vec()));
                }
            }
        }

        drop(cursor);

        Some(ModApi {
            entities,
            game_functions,
        })
    }
}
