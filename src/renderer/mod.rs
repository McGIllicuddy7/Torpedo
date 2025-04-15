

use std::collections::HashMap;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::level::get_level;
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelComp{
    pub model:String,
    pub diffuse:String,
    pub normal:String,
    pub tint:Color,
}
impl ModelComp{
    pub fn new(model:&str, tint:Color)->Self{
        Self { model: model.to_string(), diffuse: "".to_string(), normal: "".to_string(), tint: tint}
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
            models.list.get_mut(&v.model).unwrap().transform = trans.trans.rotation.to_matrix().into();
            rend.draw_model(&models.list[&v.model], trans.trans.translation, 1.0, v.tint);
            /*if let Some(x) = &get_level().physics_comps.list.read().unwrap()[i]{
                let mut bb =BoundingBox{max:x.collision.max(), min:x.collision.min()};
                bb.min += trans.trans.translation;
                bb.max += trans.trans.translation;
                rend.draw_bounding_box(bb, Color::GREEN);
            }*/
        } 
    }
    drop(rend);
    handle.draw_fps(1400, 200);
}