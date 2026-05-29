use std::f32::consts::PI;

use rand::{random, random_bool};
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    drawing::RaylibDraw,
    math::{Quaternion, Vector3},
};

use crate::{
    engine::{ENGINE, GObject, GameMode, generate_cube, generate_ufo, random_vector, raycast},
    graphics::{ParticleSystem, SpawnData, SpawnVelocityData, create_particle_system},
};
pub mod engine;
pub mod graphics;
pub mod mesh;
pub mod ship;
fn main() {
    engine::run(setup_ufo, &mut TorpedoGameMode {});
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
    let _t = ship::create_player_ufo(Vector3::new(-10., 0., 0.0), Quaternion::identity());
    for _ in 0..20 {
        let _fountain = create_particle_system(ParticleSystem::new(
            random_vector() * 2.,
            Vector3::zero(),
            true,
            SpawnData {
                amount_to_spawn: 5,
                probability_to_spawn_per_second: 10.,
                spawn_radius: 2.,
                velocity_info: SpawnVelocityData::Sphere { radius: 10. },
                max_connection_count: 0,
                max_connection_distance: 1.,
                max_lifetime: 5.,
                min_lifetime: 2.,
                spawning_duration: -1.,
                can_stop_spawning: false,
                color: Color {
                    r: 0,
                    g: 50,
                    b: 255,
                    a: 100,
                },
                ending_color: Color {
                    r: 0,
                    g: 200,
                    b: 240,
                    a: 64,
                },
                glowing: true,
                min_radius: 0.05,
                max_radius: 0.4,
            },
        ));
        _fountain.get().get_mut().force_fields[1] = Some(graphics::ForceField::Point {
            offset: Vector3::new(0., 0., -10.),
            amount: 100.,
        });
        _fountain.get().get_mut().force_fields[2] = Some(graphics::ForceField::Point {
            offset: Vector3::new(0.0, 0.0, 10.),
            amount: 100.,
        });
        _fountain.get().get_mut().force_fields[3] = Some(graphics::ForceField::Drag { coef: 0.1 });
    }
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
        /*if handle.is_key_down(raylib::ffi::KeyboardKey::KEY_F) {
            let cinfo = ENGINE.camera_data.lock().unwrap();
            let pos = cinfo.position;
            let dir = (cinfo.target - cinfo.position).normalized();
            if let Some(rc) = raycast(pos, dir, 100., &[], true) {
                engine::delete_object(GObject::new(), rc.hit_object);
                // println!("destroyed?");
            }
        }*/
    }
}
