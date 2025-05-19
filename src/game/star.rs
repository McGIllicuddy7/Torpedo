use libc::rand;
use raylib::math::Vector3;

use crate::{level::{add_transform_comp, create_entity, TransformComp}, renderer::{add_model_comp, ModelComp}};

pub fn create_star(location:crate::math::Vector3){
    let et = create_entity().unwrap();
    let md = ModelComp::new("sphere", raylib::color::Color::WHITE);
    add_model_comp(et, md);
    let mut trans = TransformComp::new();
    trans.trans.translation = location;
    add_transform_comp(et, trans);
}
pub fn create_stars(radius:f64, count:i64){
    for _ in 0..count{
        let theta = ((rand::random::<u32>()%10000) as i32-5000) as f64/5000.0*2.0*3.14;
        let phi = ((rand::random::<u32>()%10000) as i32-5000) as f64/5000.0 *2.0 * 3.14;
        let x = theta.cos()*phi.sin();
        let y = theta.sin()*phi.sin();
        let z = phi.cos();
        let v = crate::math::Vector3::new(x, y, z)*radius;
        create_star(v);
    }
}