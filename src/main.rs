pub mod system;
use crate::system::{
    ComponentRef, Entity, EntityComponent, EntityRef, EntityStruct, EntityVTableEntry,
    EntityWrapper, GameEngineVTableEntry, create_debug_cube, create_entity, game_loop,
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
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[repr(u32)]
pub enum EntityComponentKind {
    #[default]
    Body = 0,
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
        let tmp = Ship::new(Vector3::zero(), 10);
        let d = 5;
        for i in -d..=d {
            for j in -d..=d {
                for k in -d..=d {
                    create_debug_cube(
                        Vector3::new(i as f32, j as f32, k as f32) * 5.,
                        if i != 0 || j != 0 || k != 0 {
                            -Vector3::new(i as f32, j as f32, k as f32).normalized()
                        } else {
                            Vector3::zero()
                        },
                        0.5,
                        Color::color_from_hsv(
                            (j as f32 + 2.) / 5. * 360.,
                            1.0,
                            (k as f32 + 3.) / 6.0,
                        ),
                    );
                }
            }
        }
    });
}

make_trait_wrapper!(Ship, EntityKind::Object,make_ship, (constructor_args),(data component names: ("data":EntityComponentKind::Body)), ((counter, i32, "data", i32_data, 0,get_counter, get_counter_mut)), (("ship_comp", EntityComponentKind::Body, ShipComp, ShipCompMut,get_ship_comp, get_ship_comp_mut)));

make_component_wrapper!(
    ShipComp,
    ShipCompMut,
    Body,
    ((
        remaining_fuel,
        f32,
        f32_data,
        0,
        get_remaining_fuel,
        get_remaining_fuel_mut
    ))
);

pub fn make_ship(s: &mut Ship) {
    *s.get_ship_comp_mut().get_remaining_fuel_mut() = 1000.0;
}
