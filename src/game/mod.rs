use raylib::{camera::Camera3D, ffi::CameraMode, RaylibHandle, RaylibThread};
pub mod ship;
pub struct PlayerData{
    pub camera:Camera3D,
}
pub fn handle_player(player_data:&mut PlayerData,_thread:&RaylibThread,handle:&mut RaylibHandle){
    handle.update_camera(&mut player_data.camera, CameraMode::CAMERA_ORBITAL);
}
pub fn game_create_level(){
    
}