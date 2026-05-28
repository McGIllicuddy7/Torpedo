use raylib::ffi::Vector3;

use crate::engine::{GameObject, GameObjectData};

pub struct Ship {
    pub data: GameObjectData,
    pub is_ai: bool,
}
impl GameObject for Ship {
    fn on_update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        self.update(handle, thread);
    }
    fn on_event(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
        ev: crate::engine::Event,
    ) {
    }
    fn get_data(&self) -> &GameObjectData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut GameObjectData {
        &mut self.data
    }
}

impl Ship {
    pub fn update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        let mut input = if self.is_ai {
        } else {
        };
    }
}

pub struct Input {
    pub should_fire_cannon: bool,
    pub should_fire_missile: bool,
    pub rotational_acc: Vector3,
    pub lin_acc: Vector3,
}
