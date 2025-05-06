use std::collections::HashMap;

use raylib::color::Color;
use serde::{Deserialize, Serialize};

use crate::{
    level::{
        Entity, TransformComp, add_child_object, add_tag, add_transform_comp, create_entity,
        destroy_entity, get_transform_comp, get_transform_mut,
    },
    math::{BoundingBox, Quaternion, Transform, Vector3},
    physics::{Collision, PhysicsComp, add_physics_comp, get_physics_mut},
    renderer::{ModelComp, add_model_comp, get_model_mut},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthComp {
    pub health: usize,
}
crate::gen_comp_functions!(
    HealthComp,
    health_comps,
    add_health_comp,
    remove_health_comp,
    get_health_comp,
    get_health_mut
);
#[derive(Serialize, Deserialize, Clone)]
pub struct FuelComp {
    pub amount_liters: usize,
}
crate::gen_comp_functions!(
    FuelComp,
    fuel_comps,
    add_fuel_comp,
    remove_fuel_comp,
    get_fuel_comp,
    get_fuel_mut
);
#[derive(Serialize, Deserialize, Clone)]
pub struct InventoryComp {
    pub items: HashMap<String, usize>,
}
crate::gen_comp_functions!(
    InventoryComp,
    inventory_comps,
    add_inventory_comp,
    remove_inventory_comp,
    get_inventory_comp,
    get_inventory_mut
);
#[derive(Serialize, Deserialize, Clone)]
pub struct ShipComp {
    weapons: Vec<Entity>,
    fuel_tanks: Vec<Entity>,
    crew_compartments: Vec<Entity>,
    cargo_compartments: Vec<Entity>,
    engines: Vec<Entity>,
}
crate::gen_comp_functions!(
    ShipComp,
    ship_comps,
    add_ship_comp,
    remove_ship_comp,
    get_ship_comp,
    get_ship_mut
);

pub struct ExplodeOnDestroyedComp {
    pub damage: usize,
    pub damage_type: DamageType,
    pub exponent: f64,
    pub range: f64,
}
pub enum DamageType {
    Explosion,
    Bullet,
    Laser,
}

pub fn apply_damage(ent_id: Entity, amount: usize, _damage_type: DamageType) {
    if let Some(mut hc) = get_health_mut(ent_id) {
        if amount >= hc.health {
            hc.health = 0;
            destroy_entity(ent_id);
        } else {
            hc.health -= amount;
        }
    }
}

pub struct ShipBuilder {
    pub ref_entity: Entity,
}
impl Default for ShipComp {
    fn default() -> Self {
        Self::new()
    }
}

impl ShipComp {
    pub const fn new() -> Self {
        Self {
            weapons: Vec::new(),
            fuel_tanks: Vec::new(),
            crew_compartments: Vec::new(),
            cargo_compartments: Vec::new(),
            engines: Vec::new(),
        }
    }
}
impl Default for ShipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ShipBuilder {
    pub fn new() -> Self {
        let out = create_entity().unwrap();
        add_ship_comp(out, ShipComp::new());
        add_transform_comp(out, TransformComp::new());
        add_physics_comp(out, PhysicsComp::new());
        add_tag(out, "ship");
        Self { ref_entity: out }
    }
    pub fn location(self, location: Vector3) -> Self {
        get_transform_mut(self.ref_entity)
            .unwrap()
            .trans
            .translation = location;
        self
    }
    pub fn rotation(self, rotation: Quaternion) -> Self {
        get_transform_mut(self.ref_entity).unwrap().trans.rotation = rotation;
        self
    }
    pub fn body(self, path: &str, tint: Color, extents: Vector3) -> Self {
        if let Some(mut md) = get_model_mut(self.ref_entity) {
            *md = ModelComp::new(path, tint)
        } else {
            add_model_comp(self.ref_entity, ModelComp::new(path, tint));
        }

        get_physics_mut(self.ref_entity)
            .unwrap()
            .collisions
            .push(Collision {
                col: BoundingBox {
                    min: -extents / 2.,
                    max: extents / 2.,
                },
                offset: Transform::default(),
                entity_ref: None,
                mass: 1.,
            });
        self
    }
    pub fn build(self) -> Entity {
        self.ref_entity
    }
    pub fn add_child(self, offset: Transform, entity: Entity) -> Self {
        {
            if get_transform_comp(entity).is_none() {
                add_transform_comp(entity, TransformComp::new());
            }
            get_transform_mut(entity).unwrap().trans = offset;
        }
        add_child_object(self.ref_entity, entity);
        self
    }
}

pub fn create_basic_ship(location: Vector3) -> Entity {
    ShipBuilder::new()
        .location(location)
        .body("cylinder", Color::BLUE, Vector3::new(0.5, 0.25, 0.25))
        .build()
}
