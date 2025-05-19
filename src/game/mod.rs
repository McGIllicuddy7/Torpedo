use raylib::{camera::Camera3D, color, input::key_from_i32, math::Matrix, RaylibHandle, RaylibThread};
use ship::{get_input_comp, get_input_mut};
use star::create_stars;

use crate::{
    draw_call::draw_text, level::{default_setup, entities_with_tag, get_player_entity, get_transform_comp, get_transform_mut}, math::{Quaternion, Vector3, Vector4}, physics::{get_physics_comp, get_physics_mut}, renderer::ModelList, ui::UI
};

pub mod ship;
pub mod star;
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
    player_data.camera.target = raylib::prelude::Vector3::forward().transform_with( mat)+player_data.camera.position;
    player_data.camera.up = raylib::prelude::Vector3::up().transform_with(mat)+player_data.camera.position;
   // let dr = handle.get_mouse_delta().normalized();
    get_input_mut(et).unwrap().rotational_input.x = 0.0;
    get_input_mut(et).unwrap().rotational_input.y=  0.0;
    get_input_mut(et).unwrap().rotational_input.z = 0.0;
    get_input_mut(et).unwrap().linear_input = Vector3::new(0.0, 0.0, 0.0);
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_LEFT_SHIFT){
           get_input_mut(et).unwrap().linear_input.z = 1.0;
    } else if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_C){
           get_input_mut(et).unwrap().linear_input.z = -1.0;
    }else{
            get_input_mut(et).unwrap().linear_input= -get_physics_comp(et).unwrap().velocity/100.0; 
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_W){
           get_input_mut(et).unwrap().rotational_input.y -= 0.1; 
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_S){
            get_input_mut(et).unwrap().rotational_input.y += 0.1;  
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_A){
           get_input_mut(et).unwrap().rotational_input.x -= 0.1; 
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_D){
            get_input_mut(et).unwrap().rotational_input.x += 0.1;  
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_Q){
        get_input_mut(et).unwrap().rotational_input.z-=0.1;
    }
    if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_E){
        get_input_mut(et).unwrap().rotational_input.z+=0.1;
    }
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
    create_stars(100.0, 1000);
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
    run_ai();
    run_ships();
}

pub fn run_ai() {}
pub fn run_ships(){
    let ships = entities_with_tag("ship");
    let mut idx =0;
    for i in ships{
        let imp = get_input_comp(i).unwrap();
        let mut trans = get_transform_mut(i).unwrap();  
        let a =imp.rotational_input.as_rl_vec()*0.1;
        let p = a+ trans.trans.rotation.as_rl_vec().to_euler();
        let quat = raylib::math::Quaternion::from_euler(p.x,p.y, a.z);
        trans.trans.rotation= Quaternion::from_rl_vec(quat);
        draw_text(1200, 100+idx*120, 8, format!("{:#?}, {:#?}", trans.trans.rotation, a),color::Color::WHITE);
        if let Some(mut phys) = get_physics_mut (i){
            phys.velocity += Vector3::from_rl_vec(imp.linear_input.as_rl_vec().rotate_by(trans.trans.rotation.as_rl_vec()));
        }
        idx += 1;
    }
}