use raylib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PhysicsComponent{
    pub location:Transform, 
    pub collision:BoundingBox,
}
crate::get_level_comp_list!(PhysicsComponent, physics_comps, add_physics_comp,remove_physics_comp,get_physics_comp, get_physics_mut);