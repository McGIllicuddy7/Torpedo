use raylib::{RaylibHandle, RaylibThread, camera::Camera3D, ffi::CameraMode};

use crate::{
    level::get_transform_comp,
    level::{default_setup, get_player_entity},
    math::Vector3,
    renderer::ModelList,
    ui::UI,
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
    let et = get_player_entity();
    let trans = get_transform_comp(et).unwrap().trans;
   // handle.update_camera(&mut player_data.camera, CameraMode::CAMERA_ORBITAL);
    player_data.camera.position = trans.translation.as_rl_vec();
    let mat = trans.rotation.to_matrix();
    player_data.camera.target = raylib::prelude::Vector3::forward().transform_with( mat);
    player_data.camera.up = raylib::prelude::Vector3::up().transform_with(mat);
}
pub fn game_create_level(
    thread: &raylib::RaylibThread,
    handle: &mut raylib::RaylibHandle,
) -> ModelList {
    let out = default_setup(thread, handle, 16384);
    let player = ship::create_basic_ship(Vector3::new(5.0, 0.0, 0.0));
    ship::create_basic_ship(Vector3::new(-5.0, 0.0, 0.0));
    #[allow(static_mut_refs)]
    unsafe {
        crate::level::LEVEL.as_mut().unwrap().player_entity = player;
    }
    out
}
pub fn run_game_systems(
    player_data: &mut PlayerData,
    thread: &RaylibThread,
    handle: &mut RaylibHandle,
    _dt: f64,
    _ui: &mut UI,
) {
    handle_player(player_data, thread, handle);
}

pub fn run_ai() {}
pub fn run_ships(){

}
