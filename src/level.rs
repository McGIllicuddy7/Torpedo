use std::{ops::{Deref, DerefMut}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};
use raylib::math::Transform;
use serde::{Deserialize, Serialize};
pub static mut LEVEL:Option<Level> = None;
pub unsafe fn level_check_entity(ent:Entity)->bool{
    get_level().check_entity_ref(ent)
}
use crate::{physics::PhysicsComp, renderer::ModelComp};
#[derive(Serialize, Deserialize, Clone)]
pub struct TransformComp{
    pub trans:Transform,
}
crate::gen_comp_functions!(TransformComp, transform_comps, add_transform_comp,remove_transform_comp, get_transform_comp, get_transform_mut);
#[derive(Clone,Copy, Serialize)]
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
    pub loaded_models:Vec<String>,
    pub existing_entities:RwLock<Box<[bool]>>,
    #[serde(with = "RwLock")]
    pub component_indexes:RwLock<Box<[u32]>>, 
    pub physics_comps:ComponentList<PhysicsComp>,
    pub transform_comps:ComponentList<TransformComp>,
    pub model_comps:ComponentList<ModelComp>,
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
        Self { loaded_models:Vec::new(),existing_entities:RwLock::new(existing_entities.into()),component_indexes: RwLock::new(comp_idexs.into()), physics_comps: ComponentList::init(ent_size), transform_comps:  ComponentList::init(ent_size), model_comps:  ComponentList::init(ent_size) }
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