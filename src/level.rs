use std::{ops::{Deref, DerefMut}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};
use serde::{Deserialize, Serialize};

use crate::physics::PhysicsComponent;
#[derive(Clone,Copy, Serialize)]
pub struct Entity{
    pub idx:u32, 
    pub generation:u32, 
}
pub struct CompRef<T:'static>{
    lock:RwLockReadGuard<'static, Box<[Option<T>]>>,
    idx:usize,
}
impl <T:'static>Deref for CompRef<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock[self.idx].as_ref().unwrap()
    }
}

pub struct CompMut<T:'static>{
    lock:RwLockWriteGuard<'static, Box<[Option<T>]>>,
    idx:usize,
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
    #[serde(with = "RwLock")]
    pub component_indexes:RwLock<Box<[u32]>>, 
    pub physics_comps:ComponentList<PhysicsComponent>,
}
impl Level{
    pub fn check_entity_ref(&self, ent:Entity)->bool{
        let p = self.component_indexes.read().unwrap();
        return p[ent.idx as usize] == ent.generation
    }
}

