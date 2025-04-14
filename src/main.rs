use std::{collections::HashMap, f32::consts::TAU, sync::Mutex};

use level::{add_transform_comp, create_entity, save_level, Level, TransformComp};
use physics::{add_physics_comp, create_box, PhysicsComp};
use raylib::prelude::*;
use renderer::{add_model_comp, ModelComp, ModelList};


pub mod physics;
pub mod renderer;

pub mod level;
static LEVEL_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
static GAME_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
#[allow(unused)]
pub fn make_test_level(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let mut model_list = ModelList{list:HashMap::new()};
    let sz = 0.1;
    let ms =raylib::models::Mesh::gen_mesh_cube(thread, sz, sz,sz);
    let box_mesh = handle.load_model_from_mesh(thread, unsafe {
        ms.make_weak()  
    }).unwrap();
    model_list.list.insert("box".into(),box_mesh);
    let level = Level::new(32);
    unsafe{
        level::LEVEL = Some(level);
    }
    let mut size = Vector3::new(0.2, 0.2, 0.2);
    let count = 10;
    let rad = 1.;
    let colors = [Color::RED, Color::BLUE, Color::GREEN, Color::WHITE, Color::BLACK, Color::PURPLE, Color::PINK, Color::CRIMSON, Color::CYAN, Color::DARKGREEN];
    for i in 0..count{
        let mut deg = i as f32 / count as f32*TAU;
        let x = deg.cos()*rad;
        let y= deg.sin()*rad;
        let location = Vector3::new(x,y, 0.);
        let velocity = Vector3::new(-x, -y, 0.).normalized()/10.0;
        create_box(size, location, velocity, colors[i as usize]);
    }
    model_list
}
static LEVEL_TO_LOAD:Mutex<Option<Box<dyn Fn(&raylib::RaylibThread,&mut raylib::RaylibHandle)-> ModelList+Send+Sync>>> = Mutex::new(None);
pub fn level_loop(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle){
    let mut model_list =LEVEL_TO_LOAD.lock().unwrap().as_ref().unwrap()(thread, handle);
    let mut cam =  Camera3D::perspective(Vector3::new(-0.4, 0., 0.0) ,Vector3::new(1.0,0.,0.), Vector3::new(0.0, 0.0, 1.0,),90.0);
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
        physics::update();
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(color::Color::NAVY);
        renderer::render(thread, &mut draw, &mut model_list,&mut cam);
    }
    save_level("test.json");
    unsafe{
        crate::level::LEVEL = None;
    }
}
pub fn main_loop(){
    *LEVEL_TO_LOAD.lock().unwrap() = Some(Box::new(make_test_level));
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
