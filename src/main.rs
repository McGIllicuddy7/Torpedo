
use level::{default_setup, main_loop};

use math::Vector3;
use physics::create_box;
use renderer::ModelList;

pub mod physics;
pub mod renderer;
pub mod game;
pub mod level;
pub mod math;

#[allow(unused)]
pub fn make_test_level(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let out = default_setup(thread, handle, 4096*8);
    for x in 0..20{
        for y in 0..20{
            let t = create_box(Vector3::new(x as f64/5., y as f64/5., 0.0), Vector3::new(-x as f64, -y as f64, 0.)/100.0);
        }
    }

    out
}


fn main() {
    main_loop(Box::new(make_test_level));
}
