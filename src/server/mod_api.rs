use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GrugOnFunction {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GrugEntity {
    pub description: String,
    pub on_functions: HashMap<String, GrugOnFunction>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq, Eq)]
pub enum GrugType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "i32")]
    I32,
    #[serde(rename = "f32")]
    F32,
    #[serde(rename = "id")]
    ID,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "resource")]
    Resource { resource_extension: String },
    #[serde(rename = "entity")]
    Entity { entity_type: String },
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
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GrugGameFunction {
    pub description: String,
    pub arguments: Vec<GrugArgument>,
    pub return_type: Option<GrugType>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ModApi {
    pub entities: HashMap<String, GrugEntity>,
    pub game_functions: HashMap<String, GrugGameFunction>,
}

#[derive(Debug)]
pub struct GrugModInfo {
    pub name: String,
    pub version: String,
    pub game_version: String,
    pub author: String,
}

#[derive(Debug)]
pub struct GrugMod {
    pub about: GrugModInfo,
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
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string()
                    }),
                    ("on_fire".to_string(), GrugOnFunction {
                        description: "Called when the player's gun fires, which happens when the left mouse button is pressed or held.".to_string()
                    })
                ])
            }),
            ("bullet".to_string(), GrugEntity {
                description: "The bullet fired by the player's gun.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is spawned.".to_string()
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string()
                    }),
                    ("on_tick".to_string(), GrugOnFunction {
                        description: "Called every tick.".to_string()
                    })
                ]),
            }),
            ("box".to_string(), GrugEntity {
                description: "A static or dynamic box.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction{
                        description: "Called when the entity is spawned.".to_string()
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string()
                    })
                ]),
            }),
            ("counter".to_string(), GrugEntity {
                description: "A counter that prints information to the console every tick.".to_string(),
                on_functions: HashMap::from([
                    ("on_spawn".to_string(), GrugOnFunction{
                        description: "Called when the entity is spawned.".to_string()
                    }),
                    ("on_despawn".to_string(), GrugOnFunction {
                        description: "Called when the entity is despawned.".to_string()
                    }),
                    ("on_tick".to_string(), GrugOnFunction {
                        description: "Called every tick.".to_string()
                    })
                ]),
            })
        ]),
        game_functions: HashMap::from([
            ("set_gun_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() }
                ]
            }),
            ("set_gun_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ]
            }),
            ("set_gun_rounds_per_minute".to_string(), GrugGameFunction {
                description: "Sets the rounds per minute of the spawned gun.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::I32 {name: "rounds_per_minute".to_string()},
                ]
            }),
            ("set_gun_companion".to_string(), GrugGameFunction {
                description: "Sets the companion of the spawned gun. This is a box that gets spawned alongside the gun, to show off being able to spawn other entitities during on_spawn().".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Entity { name: "companion".to_string(), entity_type: "box".to_string() },
                ]
            }),
            ("set_bullet_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() },
                ]
            }),
            ("set_bullet_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ]
            }),
            ("set_bullet_density".to_string(), GrugGameFunction {
                description: "Sets the density of the spawned bullet.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::F32 {name: "density".to_string()}
                ]
            }),
            ("set_box_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned box.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String{name: "name".to_string()}
                ]
            }),
            ("set_box_sprite_path".to_string(), GrugGameFunction {
                description: "Sets the sprite path of the spawned box.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "sprite_path".to_string(), resource_extension: ".png".to_string() }
                ]
            }),
            ("set_counter_name".to_string(), GrugGameFunction {
                description: "Sets the name of the spawned counter.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "name".to_string() }
                ]
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
                ]
            }),
            ("spawn_counter".to_string(), GrugGameFunction {
                description: "Spawns a counter, and returns its ID.".to_string(),
                return_type: Some(GrugType::ID),
                arguments: vec![
                    GrugArgument::Entity { name: "path".to_string(), entity_type: "counter".to_string() }
                ]
            }),
            ("despawn_entity".to_string(), GrugGameFunction {
                description: "Despawns an entity, given its ID.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()}
                ]
            }),
            ("rand".to_string(), GrugGameFunction {
                description: "Gets a random f32 between min and max.".to_string(),
                return_type: Some(GrugType::F32),
                arguments: vec![
                    GrugArgument::F32{name: "min".to_string()},
                    GrugArgument::F32{name: "max".to_string()},
                ]
            }),
            ("print_i32".to_string(), GrugGameFunction {
                description: "Prints an i32.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::I32{name: "i".to_string()}
                ]
            }),
            ("print_f32".to_string(), GrugGameFunction {
                description: "Prints an f32.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::F32 {name: "f".to_string()}
                ]
            }),
            ("print_string".to_string(), GrugGameFunction {
                description: "Prints a string.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::String { name: "s".to_string() }
                ]
            }),
            ("print_bool".to_string(), GrugGameFunction {
                description: "Prints a bool.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Bool {name: "b".to_string()}
                ]
            }),
            ("play_sound".to_string(), GrugGameFunction {
                description: "Plays a sound.".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::Resource { name: "path".to_string(), resource_extension: ".wav".to_string() }
                ]
            }),
            ("map_has_i32".to_string(), GrugGameFunction {
                description: "Returns whether an entity's i32 map contains a key.".to_string(),
                return_type: Some(GrugType::Bool),
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String {name: "key".to_string()}
                ]
            }),
            ("map_get_i32".to_string(), GrugGameFunction {
                description: "Returns the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!".to_string(),
                return_type: Some(GrugType::I32),
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String {name: "key".to_string()},
                ]
            }),
            ("map_set_i32".to_string(), GrugGameFunction {
                description: "Sets the value of a key in an entity's i32 map. Note that if the map doesn't contain the key, the game will throw an error, so make sure to call map_has_i32() first!".to_string(),
                return_type: None,
                arguments: vec![
                    GrugArgument::ID {name: "entity_id".to_string()},
                    GrugArgument::String{name: "key".to_string()},
                    GrugArgument::I32{name: "value".to_string()},
                ]
            })
        ]),
    };

    let result: ModApi = serde_json::from_str(source).unwrap();
    assert_eq!(expected, result);
}
