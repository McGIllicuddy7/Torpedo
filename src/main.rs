use std::{f64::consts::TAU, sync::Mutex};

use level::{default_setup, save_level};
use physics::{create_box, create_box_movable};
use crate::math::*;
use renderer::ModelList;
use raylib::{camera::Camera3D, color::{self, Color}, ffi::TraceLogLevel, prelude::RaylibDraw};

pub mod physics;
pub mod renderer;
pub mod game;
pub mod level;
pub mod math;
static LEVEL_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
static GAME_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
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
    for x in -count..count{
        for y in -count..count{
            for z in -count..count{
                let p = Vector3::new(x as f64, y as f64, z as f64)/2.0;
                let v = -p/2.;
                if rand::random::<u64>()%100<99{
                    create_box(size, p,colors[(x+y*count) as usize%colors.len()]);
                } else{
                    create_box_movable(size, p, v, colors[(x+y*count) as usize%colors.len()]);
                }

            }

        }
    }
    out
}
static LEVEL_TO_LOAD:Mutex<Option<Box<dyn Fn(&raylib::RaylibThread,&mut raylib::RaylibHandle)-> ModelList+Send+Sync>>> = Mutex::new(None);
pub fn level_loop(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle){
    let mut model_list =LEVEL_TO_LOAD.lock().unwrap().as_ref().unwrap()(thread, handle);
    let mut cam =  Camera3D::perspective(Vector3::new(-0.4, 0., 0.0).as_rl_vec() ,Vector3::new(1.0,0.,0.).as_rl_vec(), Vector3::new(0.0, 0.0, 1.0,).as_rl_vec(),90.0);
    loop{
        let should_continue = LEVEL_SHOULD_CONTINUE.lock().unwrap();
        if !*should_continue{
            break;
        }
        drop(should_continue);
        if handle.window_should_close(){
            *GAME_SHOULD_CONTINUE.lock().unwrap() = false;
            break;
        }
        let j = std::thread::spawn(|| physics::update());
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(color::Color::NAVY);
        renderer::render(thread, &mut draw, &mut model_list,&mut cam);
        j.join().unwrap();
    }
    save_level("test.json");
    unsafe{
        crate::level::LEVEL = None;
    }
}
pub fn main_loop(){
    *LEVEL_TO_LOAD.lock().unwrap() = Some(Box::new(make_test_level2));
    let (mut handle, thread) =raylib::init().title("hello window").size(1600,1000).msaa_4x().log_level(TraceLogLevel::LOG_ERROR).
    build();
    handle.disable_cursor();
    loop{
        let should_continue = GAME_SHOULD_CONTINUE.lock().unwrap();
        if !*should_continue{
            break;
        }
        drop(should_continue);
        level_loop(&thread, &mut handle);
    }
}
fn main() {
    main_loop();
}
