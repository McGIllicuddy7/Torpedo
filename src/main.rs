use std::{f64::consts::TAU, sync::Mutex};

use level::{default_setup, main_loop, save_level};
use physics::{create_box, create_box_movable};
use crate::math::*;
use renderer::ModelList;
use raylib::color::Color;

pub mod physics;
pub mod renderer;
pub mod game;
pub mod level;
pub mod math;

#[allow(unused)]
pub fn make_test_level(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let out = default_setup(thread, handle, 4096*8);
    let mut size = Vector3::new(0.1, 0.1, 0.1);
    let count = 500;
    let rad = 20.;
    let colors = [Color::RED, Color::BLUE, Color::GREEN, Color::WHITE, Color::BLACK, Color::PURPLE, Color::PINK, Color::CRIMSON, Color::CYAN, Color::DARKGREEN];
    for i in 0..count{
        let mut deg = i as f64 / count as f64*TAU;
        let x = deg.cos()*rad;
        let y= deg.sin()*rad;
        let location = Vector3::new(x,y, 0.);
        let velocity = Vector3::new(-x, -y, 0.).normalized()/5.0;
        create_box_movable(size, location, velocity, colors[i as usize %colors.len()]);
    }
    out
}
pub fn make_test_level2(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let out =default_setup(thread, handle, 4096*8);
    let size = Vector3::new(0.1, 0.1, 0.1);
    let count:i32 = 10;
    let colors = [Color::RED, Color::BLUE, Color::GREEN, Color::WHITE, Color::BLACK, Color::PURPLE, Color::PINK, Color::CRIMSON, Color::CYAN, Color::DARKGREEN];
    let mut actually = 0;
    for x in -count..count{
        for y in -count..count{
            for z in -count..count{
                let p = Vector3::new(x as f64, y as f64, z as f64)/2.0;
                let v = -p/2.;
                actually += 1;
                create_box_movable(size, p, v, colors[(x+y*count) as usize%colors.len()]);

            }

        }
    }
    println!("{:#?}", actually);
    out
}

fn main() {
    main_loop(Box::new(make_test_level2));
}
