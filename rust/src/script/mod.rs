use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::dynamo::{DynSer, Obj};

pub trait Script:DynSer+Send+Sync{
    fn on_update(&mut self, dt:f64){
        let _ = dt;
    }
    fn on_damage(&mut self, damage:f64){
        let _ = damage;
    }
}
#[derive(Serialize, Deserialize)]
pub struct ScriptComp{
    pub obj:Obj<dyn Script>, 
}

impl Deref for ScriptComp{
    type Target = dyn Script;

    fn deref(&self) -> &Self::Target {
        self.obj.deref()
    }
}
impl DerefMut for ScriptComp{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.deref_mut()
    }
}
crate::gen_comp_functions!(
    ScriptComp, 
    script_comps,
    add_script_comp,
    remove_script_comp, 
    get_script_comp, 
    get_script_mut
);