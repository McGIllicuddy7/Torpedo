pub use anyhow::Result;
pub use raylib::math::{BoundingBox, RayCollision, Vector3};
use serde::{Deserialize, Serialize};
pub use std::any::Any;
pub use std::collections::HashMap;
pub use std::sync::Arc;
pub use std::sync::{Mutex, RwLock};
pub const MAX_ENTITY_COUNT: usize = 65536;
pub type EntityUpdater = fn(&mut EntityStruct, f32) -> Result<()>;
pub use super::{EntityComponentKind, EntityKind, UPDATE_TABLE};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
pub struct Entity {
    index: u32,
    generation: u32,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityStruct {
    pub self_id: Entity,
    pub name: Option<Arc<str>>,
    pub kind: EntityKind,
    pub is_player: bool,
    pub position: Vector3,
    pub velocity: Vector3,
    pub cached_bounds: BoundingBox,
    pub component_table: HashMap<Arc<str>, EntityComponent>,
    pub data_component_table: HashMap<Arc<str>, EntityDataComponent>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityComponent {
    pub parent_id: Entity,
    pub kind: EntityComponentKind,
    pub this_bounds: BoundingBox,
    pub offset: Vector3,
    pub health: u32,
    pub u32_data: [u32; 8],
    pub i32_data: [i32; 8],
    pub f32_data: [f32; 8],
    pub entity_data: [Entity; 4],
    pub string_data: [Option<Arc<str>>; 4],
    pub component_data: [Option<Box<EntityComponent>>; 4],
    pub vector_data: [Vector3; 4],
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityDataComponent {
    pub parent_id: Entity,
    pub kind: EntityComponentKind,
    pub u32_data: [u32; 8],
    pub i32_data: [i32; 8],
    pub f32_data: [f32; 8],
    pub entity_data: [Entity; 4],
    pub string_data: [Option<Arc<str>>; 4],
    pub vector_data: [Vector3; 4],
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityNicheData {
    pub is_valid: bool,
    pub generation: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityNiche {
    pub entity: RwLock<EntityStruct>,
    pub cached: RwLock<EntityStruct>,
    pub data: Mutex<EntityNicheData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub entities: Box<[EntityNiche]>, //always MAX_ENTITY_COUNT in length,
    pub player_entity: Mutex<Option<Entity>>,
}

pub struct Runtime {
    pub level: Level,
    pub update_table: &'static HashMap<EntityKind, EntityUpdater>,
    pub to_load_path: Mutex<Option<Arc<str>>>,
}

lazy_static::lazy_static!(
    pub static ref RUNTIME: Runtime = Runtime {
        level: Level::new(),
        update_table: &UPDATE_TABLE,
        to_load_path:Mutex::new(None),
    };
);

impl Level {
    pub fn new() -> Self {
        let list: Vec<EntityNiche> = (0..MAX_ENTITY_COUNT)
            .map(|_| EntityNiche {
                entity: RwLock::new(EntityStruct::default()),
                cached: RwLock::new(EntityStruct::default()),
                data: Mutex::new(EntityNicheData {
                    is_valid: false,
                    generation: 0,
                }),
            })
            .collect();
        Self {
            entities: list.into(),
            player_entity: Mutex::new(None),
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}
