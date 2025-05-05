use raylib::{RaylibHandle, RaylibThread, camera::Camera3D, color::Color, ffi::CameraMode};
use ship::ShipBuilder;

use crate::{
    draw_call::draw_rounded_box, level::{add_child_object, default_setup, get_transform_mut}, math::{Transform, Vector3}, physics::{default_mesh, get_physics_comp, get_physics_mut, C}, renderer::ModelList, ui::{show_mouse, UI}
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
    ship::create_basic_ship(Vector3::new(5.0, 0.0, 0.0));
    ship::create_basic_ship(Vector3::new(-5.0, 0.0, 0.0));
    out
}
pub fn run_game_systems(player_data: &mut PlayerData,
    thread: &RaylibThread,
    handle: &mut RaylibHandle,
    _dt:f64,
    ui:&mut UI){
        handle_player(player_data, thread, handle);
     //   show_mouse();
        ui.new_frame_v(300, 20);
        ui.new_botton(50, 1, Color::WHITE);
        ui.new_botton(50, 2, Color::WHITE); 
       // println!("{:#?}", ui);
        ui.end_frame();
        ui.end_frame();
}