use std::sync::Mutex;

use level::Level;
use raylib::prelude::*;

pub mod physics;
pub mod renderer;
pub mod level;
#[macro_export]
macro_rules! get_level_comp_list {
    (t:#ty, a_var:#ident,getter:#ident, mut_getter:#ident, adder:#ident, remover:#ident)=>{
        pub fn #getter(ent_idx:Entity)->CompRef<t>{
            assert!(crate::Level.check_entity_ref(ent_idx));
            let lock= crate::Level.#a_var.read().unwrap();
            CompRef{lock, ent_idx.idx}
        }
        pub fn #getter_mut(ent_idx:Entity)->CompMut<t>{
            assert!(crate::Level.check_entity_ref(ent_idx));
            let mut lock= crate::Level.#a_var.write().unwrap();
            CompMut{lock, ent_idx.idx} 
        }
        pub fn #adder(ent_idx:Entity, value:#t){
            assert!(crate::Level.check_entity_ref(ent_idx));
            let mut lock = crate::Level.#a_var.write().unwrap;
            *lock[ent_idx.idx as usize] = Some(value); 
        }
        pub fn #remover(ent_idx:Entity){
            assert!(crate::Level.check_entity_ref(ent_idx));
            let mut lock = crate::Level.#a_var.write().unwrap;
            *lock[ent_idx.idx as usize] = None;
        }
    }
}
static LEVEL_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
static GAME_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
#[allow(unused)]
static LEVEL_TO_LOAD:Mutex<String> = Mutex::new(String::new());
static mut LEVEL:Option<Level> = None;
pub fn level_loop(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle){
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
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(color::Color::NAVY);
        renderer::render(thread, &mut draw);
    }
}
pub fn main_loop(){
    let (mut handle, thread) =raylib::init().title("hello window").size(1600,1000).build();
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
