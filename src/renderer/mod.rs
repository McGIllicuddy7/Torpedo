

use std::collections::HashMap;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};
pub mod particles;
use crate::level::get_level;
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelData{
    pub model:String,
    pub diffuse:String,
    pub normal:String,
    pub tint:Color,
    pub offset:Transform,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelComp{
    pub models:Vec<ModelData>
}
impl ModelComp{
    pub fn new(model:&str, tint:Color)->Self{
        Self { models:vec![ModelData{model: model.to_string(), diffuse: "".to_string(), normal: "".to_string(), tint: tint, offset:Transform::default()}]}
    }
}
pub struct ModelList{
    pub list:HashMap<String, Model>
}
crate::gen_comp_functions!(ModelComp, model_comps, add_model_comp,remove_model_comp, get_model_comp, get_model_mut);
pub fn render(_thread:&RaylibThread, handle:&mut RaylibDrawHandle, models:&mut ModelList, cam:&mut Camera){
    let l = get_level().model_comps.list.read().unwrap();
    let transforms = get_level().transform_comps.list.read().unwrap();
    handle.update_camera(cam, CameraMode::CAMERA_FREE);
    let mut rend = handle.begin_mode3D(*cam);

    for i in 0..l.len(){
        if let Some(v) = &l[i]{
            let trans = transforms[i].as_ref().unwrap();
            for model in &v.models{
                models.list.get_mut(&model.model).unwrap().transform = trans.trans.rotation.to_matrix().into();
                rend.draw_model(&models.list[&model.model], trans.trans.translation.as_rl_vec(), 1.0, model.tint);
            }
        } 
    }
    drop(rend);
    handle.draw_fps(1400, 200);
}