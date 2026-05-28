use rand::{random, random_bool};
use raylib::{RaylibHandle, RaylibThread, math::Vector3};

use crate::engine::{generate_cube, generate_ufo, random_vector};
pub mod engine;
pub mod mesh;
pub mod ship;
fn main() {
    engine::run(setup_ufo);
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
    for _ in 0..100 {
        let t = generate_ufo(random_vector() * 20., 1.0);
        let pos = t.get().get_data().location;
        let mut data = t.get_mut();
        let data = data.get_data_mut();
        data.velocity = -pos.normalized();
        data.angular_velocity = Vector3::new(
            0.0,
            -0.5 + (random::<u64>() % 100) as f32 / 100.,
            -2. + (random::<u64>() % 100) as f32 / 25.,
        );
    }
}
