

use std::{collections::HashMap, sync::RwLockReadGuard};

use raylib::prelude::*;
use serde::{Deserialize, Serialize};
pub mod particles;
use crate::{draw_call, level::{get_level, get_transform_comp, Entity, Instant, TransformComp}, physics::{self, Octree, PhysicsComp, C}};
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelData{
    pub model:String,
    pub diffuse:String,
    pub normal:String,
    pub tint:Color,
    pub offset:crate::math::Transform,
    pub parent:Option<Entity>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ModelComp{
    pub models:Vec<ModelData>, 
}
impl ModelComp{
    pub fn new(model:&str, tint:Color)->Self{
        Self { models:vec![ModelData{model: model.to_string(), diffuse: "".to_string(), normal: "".to_string(), tint: tint, offset:crate::math::Transform::default(), parent:None}]}
    }
    pub const fn empty()->Self{
        Self { models: Vec::new() }
    }
}
pub struct ModelList{
    pub list:HashMap<String, Model>
}
crate::gen_comp_functions!(ModelComp, model_comps, add_model_comp,remove_model_comp, get_model_comp, get_model_mut);
pub fn render_object<T>(dt:f64,i:usize,transforms:&RwLockReadGuard<'_,Box<[Option<TransformComp>]>>, v:&ModelComp,physics:&RwLockReadGuard<'_,Box<[Option<PhysicsComp>]>>,models:&mut ModelList, rend:&mut RaylibMode3D<T>, cam_lock:crate::math::Vector3){
    let trans = transforms[i].as_ref().unwrap();
    let loc = if let Some(k) = get_transform_comp(get_level().player_entity){
        k.trans.translation
    } else{
        cam_lock
    };
    let current = Instant{trans:trans.trans, is_valid:true};
    let iter = [current];
    let vs = if (trans.trans.translation-loc).length()/C <=dt{
        iter.as_slice()

    } else{
        trans.previous.as_ref()
    };
    if(trans.trans.translation-loc).length()<C*1./60.{
        for model in &v.models{
            if let Some(p) = &physics[i]{
                let d = p.gamma_distort();
                let d_trans =Matrix::scale(d.x as f32, d.y as f32, d.z as f32);
                let mut m_trans = Matrix::identity();
                m_trans *= trans.trans.rotation.to_matrix();
                 m_trans*= d_trans;
                models.list.get_mut(&model.model).unwrap().transform = m_trans.into();
            } else{
                models.list.get_mut(&model.model).unwrap().transform = trans.trans.rotation.to_matrix().into();
            }
            rend.draw_model(&models.list[&model.model], trans.trans.translation.as_rl_vec(), 1.0, model.tint);
        }
    }
    else{
        for i in (0..vs.len()).rev(){
            let trans = &vs[i];
            if !trans.is_valid{
                continue;
            }
            let del = (trans.trans.translation- loc).length()/C;
            if del>i as f64*1./60.0*physics::UPDATE_FREQ as f64 && del>0.00001{
                continue;
            }
            for model in &v.models{
                if let Some(p) = &physics[i]{
                    let d = p.gamma_distort();
                    let d_trans =Matrix::scale(d.x as f32, d.y as f32, d.z as f32);
                    let mut m_trans = Matrix::identity();
                    m_trans *= trans.trans.rotation.to_matrix();
                     m_trans*= d_trans;
                    models.list.get_mut(&model.model).unwrap().transform = m_trans.into();
                } else{
                    models.list.get_mut(&model.model).unwrap().transform = trans.trans.rotation.to_matrix().into();
                }
                rend.draw_model(&models.list[&model.model], trans.trans.translation.as_rl_vec(), 1.0, model.tint);
            }
            break;
        }
    }


}
pub fn render(_thread:&RaylibThread, handle:&mut RaylibDrawHandle, models:&mut ModelList, cam:&mut Camera){
    loop{
        while physics::SAFE_TO_TAKE.try_lock().is_err(){
            
        }
        if *physics::SAFE_TO_TAKE.lock().unwrap(){
            break;
        }
    }
    let l = get_level().model_comps.list.read().unwrap();
    let transforms = get_level().transform_comps.list.read().unwrap();
    let physics = get_level().physics_comps.list.read().unwrap();
    let dt = handle.get_frame_time() as f64;
    handle.update_camera(cam, CameraMode::CAMERA_FREE);
    let mut rend = handle.begin_mode3D(*cam);
    for i in 0..l.len(){
        if let Some(v) = &l[i]{
            render_object(dt, i, &transforms, v,&physics, models,&mut rend, crate::math::Vector3::from_rl_vec(cam.position));
        }
    }
    drop(rend);
    draw_call::run_draw_calls(handle);
    handle.draw_fps(1400, 200);
}