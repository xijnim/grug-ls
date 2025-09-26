use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tree_sitter::Parser;

use crate::server::document::Type;

pub mod parse;

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct GrugOnFunction {
    #[serde(default = "default_description")]
    pub description: String,

    #[serde(skip)]
    #[serde(default = "default_range")]
    pub range: tree_sitter::Range,
}

impl PartialEq for GrugOnFunction {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
    }
}

#[derive(Serialize, Deserialize, Eq, Debug)]
pub struct GrugEntity {
    #[serde(default = "default_description")]
    pub description: String,

    #[serde(default)]
    pub on_functions: HashMap<String, GrugOnFunction>,

    #[serde(default = "default_range")]
    #[serde(skip)]
    pub range: tree_sitter::Range,
}

impl PartialEq for GrugEntity {
    fn eq(&self, other: &Self) -> bool {
        self.on_functions == other.on_functions && self.description == other.description
    }
}

fn default_range() -> tree_sitter::Range {
    tree_sitter::Range {
        start_byte: 0,
        end_byte: 0,
        start_point: tree_sitter::Point { row: 0, column: 0 },
        end_point: tree_sitter::Point { row: 0, column: 0 },
    }
}
fn default_description() -> String {
    "<NO DESCRIPTION>".to_string()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum GrugDetailedType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "id")]
    ID,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "resource")]
    Resource { resource_extension: String },

    #[serde(untagged)]
    Entity(String),
}

impl GrugDetailedType {
    pub fn as_type(&self) -> Type {
        match self {
            GrugDetailedType::ID => Type::ID,
            GrugDetailedType::F32 => Type::F32,
            GrugDetailedType::Bool => Type::Bool,
            GrugDetailedType::Entity(entity_type) => Type::Entity(entity_type.to_string()),
            GrugDetailedType::I32 => Type::I32,
            GrugDetailedType::String => Type::String,
            GrugDetailedType::Resource { .. } => Type::Resource,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[derive(Debug, PartialEq, Eq)]
pub enum GrugArgument {
    #[serde(rename = "string")]
    String { name: String },
    #[serde(rename = "i32")]
    I32 { name: String },
    #[serde(rename = "f32")]
    F32 { name: String },
    #[serde(rename = "id")]
    ID { name: String },

    #[serde(rename = "bool")]
    Bool { name: String },

    #[serde(rename = "resource")]
    Resource {
        name: String,
        resource_extension: String,
    },
    #[serde(rename = "entity")]
    Entity { name: String, entity_type: String },

    #[serde(untagged)]
    Unknown { name: String, r#type: String },
}

impl GrugArgument {
    pub fn get_name(&self) -> &str {
        match self {
            GrugArgument::String { name }
            | GrugArgument::I32 { name }
            | GrugArgument::F32 { name }
            | GrugArgument::ID { name }
            | GrugArgument::Bool { name }
            | GrugArgument::Resource { name, .. }
            | GrugArgument::Entity { name, .. }
            | GrugArgument::Unknown { name, .. } => name,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            GrugArgument::String { .. } => Type::String,
            GrugArgument::I32 { .. } => Type::I32,
            GrugArgument::F32 { .. } => Type::F32,
            GrugArgument::ID { .. } => Type::ID,
            GrugArgument::Bool { .. } => Type::Bool,
            GrugArgument::Resource { .. } => Type::String,
            GrugArgument::Entity { .. } => Type::String,
            GrugArgument::Unknown { r#type, .. } => Type::Entity(r#type.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct GrugGameFunction {
    #[serde(default = "default_description")]
    pub description: String,

    #[serde(default)]
    pub arguments: Vec<GrugArgument>,

    pub return_type: Option<GrugDetailedType>,

    #[serde(skip)]
    #[serde(default = "default_range")]
    pub range: tree_sitter::Range,
}

impl PartialEq for GrugGameFunction {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
            && self.arguments == other.arguments
            && self.return_type == other.return_type
    }
}

impl GrugGameFunction {
    pub fn format(&self, name: &str) -> String {
        let mut text = format!("{}(", name);
        for (idx, arg) in self.arguments.iter().enumerate() {
            text.push_str(arg.get_name());

            text.push_str(": ");

            text.push_str(arg.get_type().as_str());

            if idx < self.arguments.len() - 1 {
                text.push_str(", ");
            }
        }

        text.push(')');

        if let Some(ret_type) = &self.return_type {
            text.push(' ');
            text.push_str(ret_type.as_type().as_str());
        }

        text
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ModApi {
    pub entities: HashMap<String, GrugEntity>,

    pub game_functions: HashMap<String, GrugGameFunction>,
}

lazy_static! {
    pub static ref JSON_PARSER: Mutex<Parser> = Mutex::new({
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_json::LANGUAGE.into())
            .unwrap();

        parser
    });
}

#[test]
fn mod_api_test() {
    let source = r#" {
    "entities": {
        "gun": {
            "description": "The gun in the center of the screen that the player fires by holding the left mouse button.",
            "on_functions": {
                "on_spawn": {
                    "description": "Called when the entity is spawned."
                },
                "on_despawn": {
                    "description": "Called when the entity is despawned."
                },
                "on_fire": {
                    "description": "Called when the player's gun fires, which happens when the left mouse button is pressed or held."
                }
            }
        },
        "bullet": {
            "description": "The bullet fired by the player's gun.",
            "on_functions": {
                "on_spawn": {
                    "description": "Called when the entity is spawned."
                },
                "on_despawn": {
                    "description": "Called when the entity is despawned."
                },
                "on_tick": {
                    "description": "Called every tick."
                }
            }
        },
        "box": {
            "description": "A static or dynamic box.",
            "on_functions": {
                "on_spawn": {
                    "description": "Called when the entity is spawned."
                },
                "on_despawn": {
                    "description": "Called when the entity is despawned."
                }
            }
        },
        "counter": {
            "description": "A counter that prints information to the console every tick.",
            "on_functions": {
                "on_spawn": {
                    "description": "Called when the entity is spawned."
                },
                "on_despawn": {
                    "description": "Called when the entity is despawned."
                },
                "on_tick": {
                    "description": "Called every tick."
                }
            }
        }
    },
    "game_functions": {
        "set_gun_name": {
            "description": "Sets the name of the spawned gun.",
            "arguments": [
                {
                    "name": "name",
                    "type": "string"
                }
            ]
        },
        "set_gun_sprite_path": {
            "description": "Sets the sprite path of the spawned gun.",
            "arguments": [
                {
                    "name": "sprite_path",
                    "type": "resource",
                    "resource_extension": ".png"
                }
            ]
        },
        "set_gun_rounds_per_minute": {
            "description": "Sets the rounds per minute of the spawned gun.",
            "arguments": [
                {
                    "name": "rounds_per_minute",
                    "type": "i32"
                }
            ]
        },
        "set_gun_companion": {
            "description": "Sets the companion of the spawned gun. This is a box that gets spawned alongside the gun, to show off being able to spawn other entitities during on_spawn().",
            "arguments": [
                {
                    "name": "companion",
                    "type": "entity",
                    "entity_type": "box"
                }
            ]
        },
        "set_bullet_name": {
            "description": "Sets the name of the spawned bullet.",
            "arguments": [
                {
                    "name": "name",
                    "type": "string"
                }
            ]
        },
        "set_bullet_sprite_path": {
            "description": "Sets the sprite path of the spawned bullet.",
            "arguments": [
                {
                    "name": "sprite_path",
                    "type": "resource",
                    "resource_extension": ".png"
                }
            ]
        },
        "set_bullet_density": {
            "description": "Sets the density of the spawned bullet.",
            "arguments": [
                {
                    "name": "density",
                    "type": "f32"
                }
            ]
        },
        "set_box_name": {
            "description": "Sets the name of the spawned box.",
            "arguments": [
                {
                    "name": "name",
                    "type": "string"
                }
            ]
        },
        "set_box_sprite_path": {
            "description": "Sets the sprite path of the spawned box.",
            "arguments": [
                {
                    "name": "sprite_path",
                    "type": "resource",
                    "resource_extension": ".png"
                }
            ]
        },
        "set_counter_name": {
            "description": "Sets the name of the spawned counter.",
            "arguments": [
                {
                    "name": "name",
                    "type": "string"
                }
            ]
        },
        "spawn_bullet": {
            "description": "Spawns a bullet.",
            "arguments": [
                {
                    "name": "name",
                    "type": "entity",
                    "entity_type": "bullet"
                },
                {
                    "name": "x",
                    "type": "f32"
                },
                {
                    "name": "y",
                    "type": "f32"
                },
                {
                    "name": "angle_in_degrees",
                    "type": "f32"
                },
                {
                    "name": "velocity_in_meters_per_second",
                    "type": "f32"
                }
            ]
        },
        "spawn_counter": {
            "description": "Spawns a counter, and returns its ID.",
            "return_type": "id",
            "arguments": [
                {
                    "name": "path",
                    "type": "entity",
                    "entity_type": "counter"
                }
            ]
        },
        "despawn_entity": {
            "description": "Despawns an entity, given its ID.",
            "arguments": [
                {
                    "name": "entity_id",
                    "type": "id"
                }
            ]
        },
        "rand": {
            "description": "Gets a random f32 between min and max.",
            "return_type": "f32",
            "arguments": [
                {
                    "name": "min",
                    "type": "f32"
                },
                {
                    "name": "max",
                    "type": "f32"
                }
            ]
        },
        "print_i32": {
            "description": "Prints an i32.",
            "arguments": [
                {
                    "name": "i",
                    "type": "i32"
                }
            ]
        },
        "print_f32": {
            "description": "Prints an f32.",
            "arguments": [
                {
                    "name": "f",
                    "type": "f32"
                }
            ]
        },
        "print_string": {
            "description": "Prints a string.",
            "arguments": [
                {
                    "name": "s",
                    "type": "string"
                }
            ]
        },
        "print_bool": {
            "description": "Prints a bool.",
            "arguments": [
                {
                    "name": "b",
                    "type": "bool"
                }
            ]
        },
        "play_sound": {
            "description": "Plays a sound.",
            "arguments": [
                {
                    "name": "path",
                    "type": "resource",
                    "resource_extension": ".wav"
                }
            ]
        },
        "map_has_i32": {
            "description": "Returns whether an entity's i32 map contains a key.",
            "return_type": "bool",
            "arguments": [
                {
                    "name": "entity_id",
                    "type": "id"
                },
                {
                    "name": "key",
                    "type": "string"
                }
            ]
        },
        "map_get_i32": {
            "description": "Returns the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!",
            "return_type": "i32",
            "arguments": [
                {
                    "name": "entity_id",
                    "type": "id"
                },
                {
                    "name": "key",
                    "type": "string"
                }
            ]
        },
        "map_set_i32": {
            "description": "Sets the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!",
            "arguments": [
                {
                    "name": "entity_id",
                    "type": "id"
                },
                {
                    "name": "key",
                    "type": "string"
                },
                {
                    "name": "value",
                    "type": "i32"
                }
            ]
        }
    }
}"#;

    let expected = ModApi {
        entities: HashMap::from([
            ("gun".to_string(), GrugEntity {
                description: "The gun in the center of the screen that the player fires by holding the left mouse button.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is spawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_fire".to_string(), GrugOnFunction {
                        description: "Called when the player's gun fires, which happens when the left mouse button is pressed or held.".to_string(),
                        range: default_range(),
                    })
                ]),
                range: default_range(),
            }),
            ("bullet".to_string(), GrugEntity {
                description: "The bullet fired by the player's gun.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is spawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_tick".to_string(), GrugOnFunction {
                        description: "Called every tick.".to_string(),
                        range: default_range(),
                    })
                ]),
                range: default_range(),
            }),
            ("box".to_string(), GrugEntity {
                description: "A static or dynamic box.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction{
                        description: "Called when the entity is spawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string(),
                        range: default_range(),
                    })
                ]),
                range: default_range(),
            }),
            ("counter".to_string(), GrugEntity {
                description: "A counter that prints information to the console every tick.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction{
                        description: "Called when the entity is spawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string(),
                        range: default_range(),
                    }),
                    ("on_tick".to_string(), GrugOnFunction {
                        description: "Called every tick.".to_string(),
                        range: default_range(),
                    })
                ]),
                range: default_range(),
            })
        ]),
        game_functions: HashMap::from([
            ("set_gun_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() }
                ],
                range: default_range(),
            }),
            ("set_gun_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ],
                range: default_range(),
            }),
            ("set_gun_rounds_per_minute".to_string(), GrugGameFunction {
                description: "Sets the rounds per minute of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::I32 {name: "rounds_per_minute".to_string()},
                ],
                range: default_range(),
            }),
            ("set_gun_companion".to_string(), GrugGameFunction {
                description: "Sets the companion of the spawned gun. This is a box that gets spawned alongside the gun, to show off being able to spawn other entitities during on_spawn().".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Entity { name: "companion".to_string(), entity_type: "box".to_string() },
                ],
                range: default_range(),
            }),
            ("set_bullet_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() },
                ],
                range: default_range(),
            }),
            ("set_bullet_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ],
                range: default_range(),
            }),
            ("set_bullet_density".to_string(), GrugGameFunction {
                description: "Sets the density of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::F32 {name: "density".to_string()}
                ],
                range: default_range(),
            }),
            ("set_box_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned box.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String{name: "name".to_string()}
                ],
                range: default_range(),
            }),
            ("set_box_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned box.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ],
                range: default_range(),
            }),
            ("set_counter_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned counter.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() }
                ],
                range: default_range(),
            }),
            ("spawn_bullet".to_string(), GrugGameFunction {
                description: "Spawns a bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Entity { name: "name".to_string(), entity_type: "bullet".to_string() },
                    GrugArgument::F32 {name: "x".to_string()},
                    GrugArgument::F32 {name: "y".to_string()},
                    GrugArgument::F32 {name: "angle_in_degrees".to_string()},
                    GrugArgument::F32 {name: "velocity_in_meters_per_second".to_string()},
                ],
                range: default_range(),
            }),
            ("spawn_counter".to_string(), GrugGameFunction {
                description: "Spawns a counter, and returns its ID.".to_string(),
                return_type: Some(GrugDetailedType::ID),
                arguments: vec![
                    GrugArgument::Entity { name: "path".to_string(), entity_type: "counter".to_string() }
                ],
                range: default_range(),
            }),
            ("despawn_entity".to_string(), GrugGameFunction {
                description: "Despawns an entity, given its ID.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()}
                ],
                range: default_range(),
            }),
            ("rand".to_string(), GrugGameFunction {
                description: "Gets a random f32 between min and max.".to_string(),
                return_type: Some(GrugDetailedType::F32),
                arguments: vec![
                    GrugArgument::F32{name: "min".to_string()},
                    GrugArgument::F32{name: "max".to_string()},
                ],
                range: default_range(),
            }),
            ("print_i32".to_string(), GrugGameFunction {
                description: "Prints an i32.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::I32{name: "i".to_string()}
                ],
                range: default_range(),
            }),
            ("print_f32".to_string(), GrugGameFunction {
                description: "Prints an f32.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::F32 {name: "f".to_string()}
                ],
                range: default_range(),
            }),
            ("print_string".to_string(), GrugGameFunction {
                description: "Prints a string.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "s".to_string() }
                ],
                range: default_range(),
            }),
            ("print_bool".to_string(), GrugGameFunction {
                description: "Prints a bool.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Bool {name: "b".to_string()}
                ],
                range: default_range(),
            }),
            ("play_sound".to_string(), GrugGameFunction {
                description: "Plays a sound.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "path".to_string(), resource_extension: ".wav".to_string() }
                ],
                range: default_range(),
            }),
            ("map_has_i32".to_string(), GrugGameFunction {
                description: "Returns whether an entity's i32 map contains a key.".to_string(),
                return_type: Some(GrugDetailedType::Bool),
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String {name: "key".to_string()}
                ],
                range: default_range(),
            }),
            ("map_get_i32".to_string(), GrugGameFunction {
                description: "Returns the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!".to_string(),
                return_type: Some(GrugDetailedType::I32),
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String {name: "key".to_string()},
                ],
                range: default_range(),
            }),
            ("map_set_i32".to_string(), GrugGameFunction {
                description: "Sets the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String{name: "key".to_string()},
                    GrugArgument::I32{name: "value".to_string()},
                ],
                range: default_range(),
            })
        ]),

        ..Default::default()
    };

    let result: ModApi = ModApi::from_json(source).unwrap();

    for (name, entity) in expected.entities {
        let Some(other) = result.entities.get(&name) else {
            panic!("No {}", name);
        };

        assert_eq!(entity, *other);
    }
}
