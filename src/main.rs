pub mod system;
use crate::system::{EntityUpdater, create_debug_cube, game_loop};
use raylib::{color::Color, math::Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
lazy_static::lazy_static!(
    pub static ref UPDATE_TABLE:HashMap<EntityKind, EntityUpdater> = HashMap::new();
);
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum EntityKind {
    #[default]
    Object = 0,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default)]
#[repr(u32)]
pub enum EntityComponentKind {
    #[default]
    Body = 0,
}

fn main() {
    game_loop(|| {
        let d = 10;
        for i in -d..=d {
            for j in -d..=d {
                for k in -d..=d {
                    create_debug_cube(
                        Vector3::new(i as f32, j as f32, k as f32) * 5.,
                        if i != 0 || j != 0 || k != 0 {
                            -Vector3::new(i as f32, j as f32, k as f32).normalized()
                        } else {
                            Vector3::zero()
                        },
                        0.5,
                        Color::color_from_hsv(
                            (j as f32 + 2.) / 5. * 360.,
                            1.0,
                            (k as f32 + 3.) / 6.0,
                        ),
                    );
                }
            }
        }
    });
}
