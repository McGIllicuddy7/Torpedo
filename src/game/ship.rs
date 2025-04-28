use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::level::Entity;

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthComp{
    pub health:usize
}
crate::gen_comp_functions!(HealthComp, health_comps, add_health_comp,remove_health_comp, get_health_comp, get_health_mut);
#[derive(Serialize, Deserialize, Clone)]
pub struct FuelComp{
    pub amount_liters:usize
}
crate::gen_comp_functions!(FuelComp, fuel_comps, add_fuel_comp, remove_fuel_comp,get_fuel_comp, get_fuel_mut );
#[derive(Serialize, Deserialize, Clone)]
pub struct InventoryComp{
    pub items:HashMap<String, usize>
}
crate::gen_comp_functions!(InventoryComp, inventory_comps, add_inventory_comp, remove_inventory_comp, get_inventory_comp, get_inventory_mut);
#[derive(Serialize, Deserialize, Clone)]
pub struct ShipComp{
    weapons:Vec<Entity>, 
    fuel_tanks:Vec<Entity>, 
    crew_compartments:Vec<Entity>, 
    cargo_compartments:Vec<Entity>, 
    engines:Vec<Entity>,
}
crate::gen_comp_functions!(ShipComp, ship_comps, add_ship_comp, remove_ship_comp, get_ship_comp, get_ship_mut);
pub enum DamageType{
    Explosion, Bullet, Laser
}



pub fn apply_damage(ent_id:Entity, amount:usize, _damage_type:DamageType){
    if let Some(mut hc)=  get_health_mut(ent_id){
        hc.health -= amount;
    }
}
