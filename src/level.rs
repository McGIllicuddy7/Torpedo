use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use raylib::{camera::Camera3D, color, ffi::TraceLogLevel, models::RaylibMesh, prelude::RaylibDraw, RaylibHandle, RaylibThread};
use crate::{game::{handle_player, PlayerData}, math::{Transform, Vector3}, physics, renderer};
static LEVEL_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
static GAME_SHOULD_CONTINUE:Mutex<bool> = Mutex::new(true);
use serde::{Deserialize, Serialize};
pub static mut LEVEL:Option<Level> = None;
pub unsafe fn level_check_entity(ent:Entity)->bool{
    get_level().check_entity_ref(ent)
}
use crate::{physics::PhysicsComp, renderer::{ModelComp, ModelList}};
#[derive(Serialize, Deserialize, Clone)]
pub struct TransformComp{
    pub trans:Transform,
}
crate::gen_comp_functions!(TransformComp, transform_comps, add_transform_comp,remove_transform_comp, get_transform_comp, get_transform_mut);
#[derive(Clone,Copy, Serialize,Deserialize)]
pub struct Entity{
    pub idx:u32, 
    pub generation:u32, 
}
pub struct CompRef<T:'static>{
    pub lock:RwLockReadGuard<'static, Box<[Option<T>]>>,
    pub idx:usize,
}
impl <T:'static>Deref for CompRef<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock[self.idx].as_ref().unwrap()
    }
}

pub struct CompMut<T:'static>{
    pub lock:RwLockWriteGuard<'static, Box<[Option<T>]>>,
    pub idx:usize,
}
impl <T:'static>Deref for CompMut<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock[self.idx].as_ref().unwrap()
    }
}
impl <T:'static>DerefMut for CompMut<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock[self.idx].as_mut().unwrap()
    }
}
#[derive(Serialize, Deserialize)]
pub struct ComponentList<T:'static+Serialize+Send+Sync+for<'a> Deserialize<'a>+Clone>{    
    #[serde(with = "RwLock")]
    pub list:RwLock<Box<[Option<T>]>>,
}

impl <T:'static+Serialize+Send+Sync+for<'a> Deserialize<'a>+Clone> ComponentList<T>{
    pub fn init(size:usize)->Self{
        let mut list = Vec::new();
        list.reserve_exact(size);
        for _ in 0..size{
            list.push(None)
        }
        Self { list: RwLock::new(list.into()) }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level{
    pub frame_time:f64,
    pub loaded_models:Vec<String>,
    pub existing_entities:RwLock<Box<[bool]>>,
    #[serde(with = "RwLock")]
    pub component_indexes:RwLock<Box<[u32]>>, 
    pub physics_comps:ComponentList<PhysicsComp>,
    pub transform_comps:ComponentList<TransformComp>,
    pub model_comps:ComponentList<ModelComp>,
    pub player_entity:Entity,
}
impl Level{
    pub fn check_entity_ref(&self, ent:Entity)->bool{
        let p = self.component_indexes.read().unwrap();
        return p[ent.idx as usize] == ent.generation
    }
    pub fn new(ent_size:usize)->Self{
        let mut comp_idexs = Vec::new();
        let mut existing_entities = Vec::new();
        existing_entities.reserve_exact(ent_size);
        comp_idexs.reserve_exact(ent_size);
        for _ in 0..ent_size{
            comp_idexs.push(0);
            existing_entities.push(false);
        }
        Self { loaded_models:Vec::new(),existing_entities:RwLock::new(existing_entities.into()),component_indexes: RwLock::new(comp_idexs.into()), physics_comps: ComponentList::init(ent_size), transform_comps:  ComponentList::init(ent_size), model_comps:  ComponentList::init(ent_size) ,frame_time:1./60. ,player_entity:Entity { idx: ent_size as u32+1, generation: ent_size as u32 +1}}
    }
}
pub fn get_level()->&'static Level{
    unsafe{
        let t = &raw const LEVEL;
        t.as_ref().unwrap().as_ref().unwrap()
    }
}

#[macro_export]
macro_rules! gen_comp_functions {
    ($t:ty, $var_name:ident,$adder:ident, $remover:ident, $getter:ident, $getter_mut:ident) => {
        pub fn $adder(ent:crate::level::Entity, value:$t){
            unsafe{
                assert!(crate::level::level_check_entity(ent));
                let mut lock =  crate::level::get_level().$var_name.list.write().unwrap();
                lock[ent.idx as usize] = Some(value);
            }
        }
        pub fn $remover(ent:crate::level::Entity){
            unsafe{
                assert!(crate::level::level_check_entity(ent));
                let mut lock =  crate::level::get_level().$var_name.list.write().unwrap();
                lock[ent.idx as usize] =None;
            }
        }
        pub fn $getter(ent:crate::level::Entity)->Option<crate::level::CompRef<$t>>{
            unsafe{
                assert!(crate::level::level_check_entity(ent));
                let lock =  crate::level::get_level().$var_name.list.read().unwrap(); 
                if lock[ent.idx as usize].is_some(){
                    Some(crate::level::CompRef{lock, idx:ent.idx as usize})
                } else{
                    None
                }
            }
        }
        pub fn $getter_mut(ent:crate::level::Entity)->Option<crate::level::CompMut<$t>>{
            unsafe{
                assert!(crate::level::level_check_entity(ent));
                let lock =  crate::level::get_level().$var_name.list.write().unwrap(); 
                if lock[ent.idx as usize].is_some(){
                    Some(crate::level::CompMut{lock, idx:ent.idx as usize})
                } else{
                    None
                }
            }

        }
    };
}
pub fn create_entity()->Option<Entity>{
        let lv = get_level();
        let mut existing = lv.existing_entities.write().unwrap();
        let counts = lv.component_indexes.write().unwrap();
        for i in 0..existing.len(){
            if! existing[i]{
                existing[i] = true;
                return Some(Entity { idx: i as u32, generation:counts[i] as u32 });
            }
        }
        None
}
pub fn destroy_entity(ent:Entity){
    unsafe{
        if !level_check_entity(ent){
            return;
        }
        let lv = get_level();
        let mut existing = lv.existing_entities.write().unwrap();
        let mut counts = lv.component_indexes.write().unwrap(); 
        existing[ent.idx as usize] = false;
        counts[ent.idx as usize] += 1;
    }
}
pub fn save_level(file_name:&str){
    let bytes = serde_json::to_string_pretty(get_level()).unwrap();
    std::fs::write(file_name, bytes).unwrap();
}
pub fn load_level(file_name:&str)->Level{
    let level:Level = serde_json::from_slice(&std::fs::read(file_name).unwrap()).unwrap(); 
    level
}
pub fn init_level(entity_count:usize){
    let level = Level::new(entity_count);
    unsafe{
        LEVEL = Some(level);
    }
}
pub fn default_setup(thread:&RaylibThread, handle:&mut RaylibHandle, entity_count:usize)->ModelList{
    let mut model_list = ModelList{list:HashMap::new()};
    let sz = 0.1;
    let ms =raylib::models::Mesh::gen_mesh_cube(thread, sz, sz,sz);
    let box_mesh = handle.load_model_from_mesh(thread, unsafe {
        ms.make_weak()  
    }).unwrap();
    model_list.list.insert("box".into(),box_mesh);
    let msh = raylib::models::Mesh::gen_mesh_sphere(thread, sz/2.,     32, 13);
    let sphere_mesh = handle.load_model_from_mesh(thread, unsafe{msh.make_weak()}).unwrap();
    model_list.list.insert("sphere".into(), sphere_mesh);
    init_level(entity_count);
    let trs = raylib::models::Mesh::gen_mesh_torus(thread, sz/2., sz*2.,32,32);
    let torus_mesh = handle.load_model_from_mesh(thread, unsafe{trs.make_weak()}).unwrap();
    model_list.list.insert("torus".into(), torus_mesh);
    let cyl = raylib::models::Mesh::gen_mesh_cylinder(thread, sz/4., sz, 32);
    let cl_mesh = handle.load_model_from_mesh(thread, unsafe{cyl.make_weak()}).unwrap();
    model_list.list.insert("cylinder".into(), cl_mesh);
    model_list
}
pub fn get_frame_time()->f64{
    get_level().frame_time
}
static LEVEL_TO_LOAD:Mutex<Option<Box<dyn Fn(&raylib::RaylibThread,&mut raylib::RaylibHandle)-> ModelList+Send+Sync>>> = Mutex::new(None);

pub fn level_loop(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle){
    let mut model_list =LEVEL_TO_LOAD.lock().unwrap().as_ref().unwrap()(thread, handle);
    let mut cam =  Camera3D::perspective(crate::math::Vector3::new(-0.4, 0., 0.0).as_rl_vec() ,Vector3::new(1.0,0.,0.).as_rl_vec(), crate::math::Vector3::new(0.0, 0.0, 1.0,).as_rl_vec(),90.0);
    let mut player_data = PlayerData{camera:cam};
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
        handle_player(&mut player_data, thread, handle);
        let j = std::thread::spawn(|| physics::update());
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(color::Color::new(0,0, 20,255));
        renderer::render(thread, &mut draw, &mut model_list,&mut cam);
        j.join().unwrap();
        //save_level("test.json");
    }
    unsafe{
        crate::level::LEVEL = None;
    }
}
pub fn main_loop(level_to_load:Box<dyn Fn(&raylib::RaylibThread, &mut raylib::RaylibHandle)->ModelList+Send+Sync+'static>){
    *LEVEL_TO_LOAD.lock().unwrap() = Some(level_to_load);
    let (mut handle, thread) =raylib::init().title("hello window").size(1600,1000).msaa_4x().log_level
    (TraceLogLevel::LOG_ERROR).
    build();
    handle.set_target_fps(60);
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