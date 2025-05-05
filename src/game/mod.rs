use raylib::{RaylibHandle, RaylibThread, camera::Camera3D, color::Color, ffi::CameraMode};
use ship::ShipBuilder;

use crate::{
    level::{add_child_object, default_setup, get_transform_mut},
    math::{Transform, Vector3},
    physics::{C, default_mesh, get_physics_comp, get_physics_mut},
    renderer::ModelList,
};
pub mod ship;
pub struct PlayerData {
    pub camera: Camera3D,
}
pub fn handle_player(
    player_data: &mut PlayerData,
    _thread: &RaylibThread,
    handle: &mut RaylibHandle,
) {
    handle.update_camera(&mut player_data.camera, CameraMode::CAMERA_ORBITAL);
}
pub fn game_create_level(
    thread: &raylib::RaylibThread,
    handle: &mut raylib::RaylibHandle,
) -> ModelList {
    let out = default_setup(thread, handle, 16384);
    let builder = ShipBuilder::new().body("cylinder", Color::WHITE, Vector3::new(0.25, 0.5, 0.25));
    let sphere = default_mesh("sphere", Vector3::zero(), Color::RED);
    let s = builder.build();
    add_child_object(s, sphere);
    get_physics_mut(s).unwrap().velocity = Vector3::new(0., -1., 0.);
    get_transform_mut(s).unwrap().trans.translation = Vector3::new(-C, 0., 0.);
    out
}
pub fn ship_system() {}
pub fn player_system() {}
pub fn ai_system() {}

