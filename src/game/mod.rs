use raylib::{camera::Camera3D, color::Color, ffi::CameraMode, RaylibHandle, RaylibThread};
use ship::ShipBuilder;

use crate::{level::default_setup, math::Vector3, physics::{get_physics_comp, get_physics_mut}, renderer::ModelList};
pub mod ship;
pub struct PlayerData{
    pub camera:Camera3D,
}
pub fn handle_player(player_data:&mut PlayerData,_thread:&RaylibThread,handle:&mut RaylibHandle){
    handle.update_camera(&mut player_data.camera, CameraMode::CAMERA_ORBITAL);
}
pub fn game_create_level(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let out = default_setup(thread, handle, 16384);
    let s = ShipBuilder::new().body("cylinder", Color::WHITE, Vector3::new(0.25, 0.25, 0.5)).build();
    get_physics_mut(s).unwrap().velocity = Vector3::new(0. ,1., 0.);
    out
}