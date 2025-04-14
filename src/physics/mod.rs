
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{level::{add_transform_comp, create_entity, get_level, Entity, TransformComp}, renderer::{add_model_comp, ModelComp}};

pub fn min<T:PartialOrd>(a:T, b:T)->T{
    if a<b{
        a
    } else{
        b
    }
}
pub fn max<T:PartialOrd>(a:T, b:T)->T{
    if a<b{
        b
    } else{
        a
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub enum Collision{
    Box{collision:BoundingBox}, Collection{values:Vec<PhysicsComp>}
}
impl Collision{
    pub fn max(&self)->Vector3{
        match self{
            Collision::Box { collision } => collision.max,
            Collision::Collection { values } => {
                let mut mx =-Vector3::new(10000.0, 10000., 10000.0,);
                for i in values{
                    let p = i.max();
                    if p.x>mx.x{
                        mx.x = p.x;
                    }
                    if p.y>mx.y{
                        mx.y = p.y;
                    }
                    if p.z>mx.z{
                        mx.z = p.z;
                    }
                }
                mx
            },
        }
    }
    pub fn min(&self)->Vector3{
            match self{
                Collision::Box { collision } => collision.min,
                Collision::Collection { values } => {
                    let mut mx =Vector3::new(10000.0, 10000., 10000.0,);
                    for i in values{
                        let p = i.max();
                        if p.x<mx.x{
                            mx.x = p.x;
                        }
                        if p.y<mx.y{
                            mx.y = p.y;
                        }
                        if p.z<mx.z{
                            mx.z = p.z;
                        }
                    }
                    mx
                },
            }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct PhysicsComp{
    pub collision:Collision,
    pub velocity:Vector3,
    pub offset:Transform,
    pub did_collide:bool,
}
impl PhysicsComp{
    pub fn max(&self)->Vector3{
        self.collision.max()
    }
    pub fn min(&self)->Vector3{
        self.collision.min()
    }
}
pub struct Col{
    pub hit_ref:Entity,
    pub norm:Vector3,
}
crate::gen_comp_functions!(PhysicsComp, physics_comps, add_physics_comp,remove_physics_comp, get_physics_comp, get_physics_mut);
fn get_vertices(a:BoundingBox,offset:Transform, a_trans:Transform)->[Vector3;8]{
    let mut verts= [Vector3::new(1.,1., 1.), Vector3::new(1., -1., 1.), Vector3::new(-1., 1., 1.), Vector3::new(-1., -1., 1.0), 
    Vector3::new(1.,1., -1.), Vector3::new(1., -1., -1.), Vector3::new(-1., 1., -1.), Vector3::new(-1., -1., -1.0)
    
    ];
    let dx = a.max.x-a.min.x;
    let dy = a.max.y-a.min.y;
    let dz = a.max.z-a.min.z;
    for i in &mut verts{
        let x = i.x*dx/4.;
        let y = i.y*dy/4.;
        let z = i.z*dz/4.;
        *i = Vector3::new(x,y,z);
    }
    for i in &mut verts{
        let mut tmp = *i;
        tmp +=  a_trans.translation;
        tmp += offset.translation.transform_with((a_trans.rotation*offset.rotation).to_matrix());
        *i = tmp;
    }
    verts
}
fn get_normals(a_trans:Transform, a_off:Transform)->[Vector3;18]{
    let mut normals = [
        Vector3::new(1., 0., 0.), Vector3::new(-1., 0., 0.),
        Vector3::new(0., 1., 0.), Vector3::new(0., -1., 0.),
        Vector3::new(0., 0., 1.), Vector3::new(0., 0., -1.),

        Vector3::new(1., 0., 1.), Vector3::new(1., 0., -1.),
        Vector3::new(-1., 0., 1.),Vector3::new(-1., 0., -1.),

        Vector3::new(1., 1., 0.), Vector3::new(1., -1., 0.),
        Vector3::new(-1., -1., 0.), Vector3::new(1., -1., 0.),

        Vector3::new(0., 1., 1.), Vector3::new(0., 1., -1.),
        Vector3::new(0., -1., 1.), Vector3::new(0., -1., -1.),
        ];
    let rot = (a_trans.rotation*a_off.rotation).to_matrix();
    for i in &mut normals{
        i.normalize();
    }
    for i in &mut normals{
        i.transform(rot);
    }

    normals
}
fn check_collision(a:BoundingBox, a_off:Transform,a_trans:TransformComp, b:BoundingBox,b_off:Transform, b_trans:TransformComp)->Option<Col>{
    let a_verts = get_vertices(a, a_off,a_trans.trans);
    let b_verts = get_vertices(b, b_off,b_trans.trans);
    let a_norms = get_normals(a_trans.trans,a_off);
    let b_norms = get_normals(b_trans.trans, b_off);
    let mut norms = [const{Vector3::new(0., 0., 0.,)};36];
    let mut idx = 0;
    for i in a_norms{
        norms[idx] = i;
        idx +=1;
    }
    for i in b_norms{
        norms[idx] = i;
        idx +=1;
    }
    for i in &norms{
        assert!(i.length()>0.);
    }
    let mut col_norm = Vector3::new(0., 0., 0.);
    let mut col_depth = 100000.0;
    for i in norms{
        let mut a_max = -1000000.0;
        let mut a_min = -a_max;
        let mut b_max = a_max;
        let mut b_min = -b_max;
        for j in a_verts{
            let a_dot = j.dot(i);
            if a_dot >a_max{
                a_max = a_dot;
            }
            if a_dot<a_min{
                a_min = a_dot;
            }
        }
        for j in b_verts{
            let b_dot = j.dot(i);
            if b_dot >b_max{
                b_max = b_dot;
            }
            if b_dot<b_min{
                b_min = b_dot;
            }
        }
        if a_min>b_max+0.1 || b_min>a_max+0.1{
           return None;
        } else{
            let da = (a_min-b_max).abs();
            let db = (b_min -a_max).abs();
            let del = if da>db{
                db
            } else{
                da
            };
            let x = Vector3::new(1., 0., 0.,).rotate_by(a_off.rotation*a_trans.trans.rotation);
            let y = Vector3::new(0., 1., 0.,).rotate_by(a_off.rotation*a_trans.trans.rotation);
            let z = Vector3::new(0., 0., 1.,).rotate_by(a_off.rotation*a_trans.trans.rotation);
            let depth = max(i.dot(x), max(i.dot(y), i.dot(z)));
            if del<col_depth && depth>0.77{
                col_depth = del;
                col_norm = i;
            }
        }
    }
    Some(Col{hit_ref:Entity{idx:0, generation:0}, norm:col_norm})
}
pub fn check_collision_pair(a:PhysicsComp, b:PhysicsComp,v:usize, i:usize,new_loc:TransformComp,phys:&mut [Option<PhysicsComp>], trans:&mut [Option<TransformComp>])->Option<Col>{
    let mut col:Option<Col> = None;
    match a.collision{
        Collision::Box { collision } => {
            let ac = collision;
            match b.collision{
                Collision::Box { collision } => {
                    let bc = collision;
                    let c = check_collision(ac, a.offset, new_loc.clone(),bc, b.offset, trans[i].clone().unwrap());
                    if c.is_some(){
                        col = c;
                    }
                }
                Collision::Collection { values } => {
                    for j in values{
                        let c = check_collision_pair(a.clone(), j, v, i, new_loc.clone(), phys, trans);
                        if c.is_some(){
                            col = c;
                            break;
                        } 
                    }
                }
            }
        }
        Collision::Collection { values } => {
            for a in values{
                let c = check_collision_pair(a.clone(), b.clone(), v, i, new_loc.clone(), phys, trans);
                if c.is_some(){
                    col = c;
                    break;
                } 
            }
        }
    }
    col
}
pub fn check_collision_objects(){}
fn check_collision_single(new_loc:TransformComp,v:usize,phys:&mut [Option<PhysicsComp>], trans:&mut [Option<TransformComp>], to_iter:&[usize])->Option<Col>{
    if to_iter.len() != 0{ 
    }

    for i in to_iter.iter().map(|i| *i){
        if trans[i].is_none() || phys[i].is_none(){
            continue;
        }
        if i != v{
            let a = phys[v].clone().unwrap();
            let b = phys[i].clone().unwrap();
            let col = check_collision_pair(a, b, v, i, new_loc.clone(), phys, trans);
            // 
            if let Some(mut col) =col{
                col.hit_ref = Entity{idx:i as u32, generation:get_level().component_indexes.read().unwrap()[i]};
                return Some(col);
            }
        }
    }
    return None;
}
const COUNT:usize = 10;
fn update_phys(v:usize,phys:&mut [Option<PhysicsComp>], trans:&mut [Option<TransformComp>], to_iter:&[[[Vec<usize>;COUNT];COUNT];COUNT], min_loc:Vector3, max_loc:Vector3){
    let old = trans[v].clone().unwrap();
    let mut new = old.clone();
    new.trans.translation += phys[v].as_ref().unwrap().velocity*1./60.;
    let d = max_loc-min_loc;
    let delt = ((phys[v].as_ref().unwrap().collision.max()-phys[v].as_ref().unwrap().collision.min()).length()*max(d.x, max(d.y,d.z))) as i64+2;
    let del = new.trans.translation-min_loc;
    let x = (del.x/d.x) as usize;
    let y = (del.y/d.y) as usize;
    let z = (del.z/d.z) as usize;
    for dx in-delt..delt{
        for dy  in -delt..delt{
            for dz in -delt..delt{
                let x = x  as i64+dx;
                let y = y as i64+dy;
                let z = z as i64+dz;
                if x<0 || x>=COUNT as i64{
                    continue;
                }
                if y<0 || y>=COUNT as i64{
                    continue;
                }
                if z<0 || z>=COUNT as i64{
                    continue;
                }
                if let Some(s) = check_collision_single(new.clone(), v, phys, trans, &to_iter[x as usize][y as usize][z as usize]){
                    phys[v].as_mut().unwrap().velocity.reflect(s.norm);
                    phys[s.hit_ref.idx as usize].as_mut().unwrap().velocity.reflect(-s.norm);
                    trans[v].as_mut().unwrap().trans.translation += phys[v].as_mut().unwrap().velocity*1./60.;
                    return;

                }

            }
        }
    }
    trans[v] = Some(new);
}
pub fn update(){
    let mut trans = get_level().transform_comps.list.read().unwrap().clone();
    let mut phys_lock = get_level().physics_comps.list.write().unwrap();
    let phys = phys_lock.as_mut();
    let mut vecs = [const {[const{[const {Vec::<usize>::new()};COUNT]}; COUNT]};COUNT];
    let mut min_loc = Vector3::new(10000.0, 10000.0, 10000.0);
    let mut max_loc = -min_loc;
    let mut iter = Vec::new();
    for i in 0..phys.len(){
        if phys[i].is_some(){
            let t = trans[i].clone().unwrap();
            let loc = t.trans.translation;
            if loc.x<min_loc.x{
                min_loc.x = loc.x;
            }
            if loc.y <min_loc.y{
                min_loc.y = loc.y;
            }
            if loc.z<min_loc.z{
                min_loc.z = loc.z;
            }
            if loc.x>max_loc.x{
                max_loc.x = loc.x;
            }
            if loc.y>max_loc.y{
                max_loc.y = loc.y;
            }
            if loc.z>max_loc.z{
                max_loc.z = loc.z;
            }
            iter.push(i);
        }
    }
    for i in&iter{
        let d = max_loc-min_loc;
        let delt = ((phys[*i].as_ref().unwrap().collision.max()-phys[*i].as_ref().unwrap().collision.min()).length()*max(d.x, max(d.y,d.z))) as i64+3;
        let del = trans[*i].as_ref().unwrap().trans.translation-min_loc;
        let x = (del.x/d.x) as usize;
        let y = (del.y/d.y) as usize;
        let z = (del.z/d.z) as usize;
        for dx in-delt..delt{
            for dy  in -delt..delt{
                for dz in -delt..delt{
                    let x = x  as i64+dx;
                    let y = y as i64+dy;
                    let z = z as i64+dz;
                    if x<0 || x>=COUNT as i64{
                        continue;
                    }
                    if y<0 || y>=COUNT as i64{
                        continue;
                    }
                    if z<0 || z>=COUNT as i64{
                        continue;
                    }
                    vecs[x as usize][y as usize][z as usize].push(*i);
                }
            }
        }
    }
    for i in iter{
        update_phys(i, phys, &mut trans, &vecs, min_loc, max_loc);
    }
    *get_level().transform_comps.list.write().unwrap() = trans
}
pub fn create_box(size:Vector3,location:Vector3,velocity:Vector3,tint:Color)->Entity{
     let cube = create_entity().unwrap();
    add_model_comp(cube, ModelComp::new("box", tint));
    let mut trans =TransformComp{trans:Transform::default()};
    trans.trans.translation = location;
    trans.trans.rotation = Vector4::new(0., 0. ,0., 1.);
    add_transform_comp(cube, trans);
    let mut trans2 = Transform::default();
    trans2.translation= Vector3::new(0., 0., 0.);
    trans2.rotation = Quaternion::new(0., 0., 0., 1.0);
    let bb = BoundingBox { min: -size/2., max:size/2. };
    let phys = PhysicsComp{
        collision: Collision::Box { collision: bb },
        velocity: velocity,
        offset: trans2,
        did_collide: false,
    };
    add_physics_comp(cube, phys);
    cube
}