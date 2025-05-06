use libc::exit;
use pprof::flamegraph::color;
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
        ui.new_frame_v(400, 20);
        ui.new_text_rounded("testing 123".to_string(), 20, Color::BLACK, Color::WHITE);
        ui.new_botton_text(50, 1, Color::WHITE, "hello world".to_string(), Color::BLACK);
        ui.new_botton_text(50, 2, Color::WHITE, "hello".to_owned(), Color::BLACK); 
        ui.end_frame();
        ui.new_frame_h(300, 20);
        if ui.new_botton_text(500, 3, Color::WHITE, "Hi Toast :3".to_string(), Color::WHEAT){
            unsafe{exit(0)};
        }
        ui.end_frame();
        ui.end_drawing();
}