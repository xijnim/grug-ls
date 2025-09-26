use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize};
use tree_sitter::Node;

use crate::server::mod_api::{GrugEntity, GrugGameFunction, JSON_PARSER, ModApi};

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
    fn parse_entry<T: ModApiEntry + DeserializeOwned>(out: &mut HashMap<String, T>, entry: &Node, json: &[u8]) {
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
            dbg!(&key);

            let buf = &json[obj.byte_range()];
            let Ok(mut entry) = serde_json::from_slice::<T>(&buf) else {
                return;
            };

            entry.set_range(entity_entry.range());
            out.insert(key, entry);
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
                    
                    Self::parse_entry(&mut entities, &value, json);
                }
                b"\"game_functions\"" => {
                    let Some(value) = entry.child_by_field_name("value") else {
                        continue;
                    };
                    if value.kind() != "object" {
                        continue;
                    }
                    
                    Self::parse_entry(&mut game_functions, &value, json);
                }
                _ => {
                    println!("Unkown key: {:?}", String::from_utf8(key.to_vec()));
                },
            }
        }

        drop(cursor);

        Some(ModApi {
            entities,
            game_functions,
        })
    }
}
