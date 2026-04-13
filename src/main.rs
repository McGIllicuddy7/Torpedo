pub mod entity;
use crate::entity::EntityUpdater;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
lazy_static::lazy_static!(
    pub static ref UPDATE_TABLE:HashMap<EntityKind, EntityUpdater> = HashMap::new();
);
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[repr(u32)]
pub enum EntityKind {
    #[default]
    Object = 0,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[repr(u32)]
pub enum EntityComponentKind {
    #[default]
    Body = 0,
}

fn main() {}
