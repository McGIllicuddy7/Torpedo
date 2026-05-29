use std::time::Duration;

use rand::{random, random_bool};
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    drawing::RaylibDraw,
    math::{Quaternion, Vector3},
};

use crate::{
    audio::{audio_write_func, init_audio},
    engine::{GameMode, generate_cube, generate_ufo, random_vector},
    graphics::{ParticleSystem, SpawnData, SpawnVelocityData, create_particle_system, draw_flame},
};
pub mod audio;
pub mod engine;
pub mod graphics;
pub mod mesh;
pub mod ship;
fn main() {
    //engine::run(setup_ufo, &mut TorpedoGameMode {});
    let _ad = init_audio();
    audio_write_func(|x| (x * 4400.).sin() / 10. + (x * 2000.).cos() / 10., 5.);
    std::thread::sleep(Duration::from_secs(10));
}

pub fn setup_old(_handle: &mut RaylibHandle, _thread: &RaylibThread) {
    let t = generate_cube(Vector3::new(0., -10., 0.0), 1.0);
    t.get_mut().get_data_mut().velocity = Vector3::new(0.0, 1.0, 0.0);
    t.get_mut().get_data_mut().angular_velocity = Vector3::new(0.0, 1.0, 0.0);
    let t = generate_cube(Vector3::new(0., 10., 0.), 1.0);
    t.get_mut().get_data_mut().velocity = Vector3::new(0.0, -1.0, 0.0);
    t.get_mut().get_data_mut().angular_velocity = Vector3::new(0.0, -1.0, 0.0);
}

pub fn setup(_handle: &mut RaylibHandle, _thread: &RaylibThread) {
    for _ in 0..1000 {
        let t = generate_cube(random_vector() * 50., 1.0);
        let pos = t.get().get_data().location;
        let mut data = t.get_mut();
        let data = data.get_data_mut();
        data.velocity = -pos.normalized();
        data.angular_velocity = random_vector();
        data.is_static = random_bool(0.5);
    }
}

pub fn setup_ufo(_handle: &mut RaylibHandle, _thread: &RaylibThread) {
    for _ in 0..10 {
        let _t = ship::create_ai_ufo(random_vector() * 20., Quaternion::identity());
    }
    let _t = ship::create_player_ufo(Vector3::new(-10., 0., 0.0), Quaternion::identity());
}

pub struct TorpedoGameMode {}
impl GameMode for TorpedoGameMode {
    fn on_render(
        &mut self,
        handle: &mut raylib::prelude::RaylibDrawHandle,
        _thread: &RaylibThread,
    ) {
        let w = handle.get_screen_width();
        let h = handle.get_screen_height();
        handle.draw_rectangle(w / 2 - 1, h / 2 - 10, 2, 20, Color::WHITE);
        handle.draw_rectangle(w / 2 - 10, h / 2 - 1, 20, 2, Color::WHITE);
    }
    fn on_update(&mut self, _handle: &mut RaylibHandle, _thread: &RaylibThread) {
        draw_flame(
            Vector3::zero(),
            Vector3::new(0., 0., 1.),
            0.1,
            1.,
            Color::VIOLET,
        );
    }
}
