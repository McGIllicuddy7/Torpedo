pub mod ship;
pub mod system;
use crate::{
    ship::create_solar_system,
    system::{
        Entity, EntityComponent, EntityStruct, EntityVTableEntry, GameEngineVTableEntry,
        create_debug_cube, create_entity, game_loop,
    },
};
use raylib::{color::Color, math::Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub fn make_vtable() -> HashMap<EntityKind, EntityVTableEntry> {
    HashMap::new()
}
pub fn make_system_vtable() -> HashMap<GameMode, GameEngineVTableEntry> {
    HashMap::new()
}
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum EntityKind {
    #[default]
    Object = 0,
    Ship,
    Asteroid,
    Station,
    Missile,
    Bullet,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[repr(u32)]
pub enum EntityComponentKind {
    #[default]
    Body = 0,
    PressurizedHull,
    FuelTank,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum GameMode {
    #[default]
    Menu,
    Running,
}
fn main() {
    game_loop(|| {
        create_solar_system();
    });
}
