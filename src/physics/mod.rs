/*
todo optimize physics(more)
*/
use std::{collections::HashMap, process::exit, sync::Mutex};
mod col;
use crate::{arena::{AVec, Arena}, level::{add_transform_comp, create_entity, get_level, Entity, Instant, TransformComp}, math::*, renderer::{add_model_comp, ModelComp, ModelData}};
use col::check_collision;
use raylib::{color::Color, prelude::{RaylibDraw3D, RaylibDrawHandle}};
use serde::{Deserialize, Serialize};
//1 meter = 1 km in this game
pub const C:f64 = 299792.;
pub const C2:f64 = C*C;
pub static SAFE_TO_TAKE:Mutex<bool> = Mutex::new(false);
pub const UPDATE_FREQ:usize = 2;
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
pub struct Collision{
    pub col:BoundingBox, 
    pub offset:Transform,
    pub entity_ref:Option<Entity>,
}
impl Collision{
    pub fn max(&self, trans:Transform)->Vector3{
        let mut u = self.col.min;
        u  = u.transform_with(trans.rotation.to_matrix());
        let mut t = self.col.max;
        t = t.transform_with(trans.rotation.to_matrix());
        t =  trans.translation +t.transform_with(trans.rotation.to_matrix());
        u = trans.translation + u.transform_with(trans.rotation.to_matrix());
        if t.x>u.x{
            u.x = t.x;
        }
        if t.y>u.y{
            u.y = t.y;
        }
        if t.y>u.z{
            u.z = t.z;
        }
        u
    }
    pub fn min(&self, trans:Transform)->Vector3{
        let mut u = self.col.min;
        let mut t = self.col.max;
        t =  trans.translation +t.transform_with(trans.rotation.to_matrix());
        u = trans.translation + u.transform_with(trans.rotation.to_matrix());
        if t.x<u.x{
            u.x = t.x;
        }
        if t.y<u.y{
            u.y = t.y;
        }
        if t.y<u.z{
            u.z = t.z;
        }
        u
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct PhysicsComp{
    pub collisions:Vec<Collision>,
    pub velocity:Vector3,
    pub anglular_velocity:Quaternion,
    pub can_ever_move:bool,
    pub named:HashMap<String,usize>,
    pub collided_this_frame:bool,
}
impl  PhysicsComp {
    pub fn new()->Self{
        Self { collisions: Vec::new(), velocity:Vector3::zero(), anglular_velocity: Vector4::zero(), can_ever_move:true,named:HashMap::new() ,collided_this_frame:false}
    }
    pub fn max(&self, trans:Transform)->Vector3{
        let mut max = Vector3::new(-10000., -10000., -10000.);
        if self.collisions.len() == 0{
            return trans.translation;
        }
        for i in &self.collisions{
            let k = i.max(trans);
            if k.x>max.x{
                max.x = k.x;
            }
            if k.y>max.y{
                max.y = k.y;
            }
            if k.z>max.z{
                max.z = k.z;
            }
        }
        max
    }
    pub fn min(&self, trans:Transform)->Vector3{
        let mut max = -Vector3::new(-10000., -10000., -10000.);
        if self.collisions.len() == 0{
            return trans.translation;
        }
        for i in &self.collisions{
            let k = i.max(trans);
            if k.x<max.x{
                max.x = k.x;
            }
            if k.y<max.y{
                max.y = k.y;
            }
            if k.z<max.z{
                max.z = k.z;
            }
        }
        max
    }
    pub fn bb(&self, trans:Transform)->BoundingBox{
        BoundingBox { min: self.min(trans), max: self.max(trans)}
    }
    pub fn gamma(&self)->f64{
        let sv = self.velocity;
        let rv = if let Some(t) = get_physics_comp(get_level().player_entity){
            t.velocity
        } else{
            Vector3::zero()
        };
        let delt = sv-rv;
        let l = delt.length_sqr();
        return 1./((1.-l/C2).sqrt());
    }
    pub fn gamma_distort(&self)->Vector3{
        let sv = self.velocity;
        let rv = if let Some(t) = get_physics_comp(get_level().player_entity){
            t.velocity
        } else{
            Vector3::zero()
        };
        let delt = sv-rv;
        let lx = delt.x.abs().sqrt();
        let ly = delt.y.abs().sqrt();
        let lz = delt.z.abs().sqrt();
       let cx =  1./((1.-lx/C2).sqrt());
       let cy =  1./((1.-ly/C2).sqrt());
       let cz =  1./((1.-lz/C2).sqrt());
       Vector3 { x: 1./cx, y:1./cy, z: 1./cz }
    }
}
pub fn check_collision_pair(a:&PhysicsComp, a_trans:TransformComp, b:&PhysicsComp, b_trans:TransformComp)->Option<Col>{
    for i in &a.collisions{
        for j in &b.collisions{
            let t = check_collision(i.col, i.offset, a_trans.clone(), j.col, j.offset, b_trans.clone());
            if let Some(t) = t{
                Some(t);
            }
        }
    }
    None
}
pub struct Col{
    pub hit_ref:Entity,
    pub norm:Vector3,
    pub depth:f64,
}
crate::gen_comp_functions!(PhysicsComp, physics_comps, add_physics_comp,remove_physics_comp, get_physics_comp, get_physics_mut);

#[derive(Debug)]
pub enum Octree<'a>{
    Values{values:AVec<'a,usize>, bx:BoundingBox},
    Boxes{values:&'a [Octree<'a>;8], bx:BoundingBox},
}
impl <'a>Octree<'a>{
    pub fn draw<T>(&self,handle:&mut raylib::prelude::RaylibMode3D<'_, T>){
        match self{
            Octree::Values { values:_, bx } => {
                handle.draw_bounding_box(bx.as_rl_box(), raylib::prelude::Color::GREEN);
            },
            Octree::Boxes { values, bx } => {
                handle.draw_bounding_box(bx.as_rl_box(), raylib::prelude::Color::GREEN);
                for i in values.iter(){
                    i.draw(handle);
                }
            }
        }
    }
    pub fn query_point(&self,point:Vector3)->Option<&[usize]>{
        match self{
            Octree::Values { values, bx } =>{
                if bx.collides_point(point){
                    return Some(values.as_ref())
                } else{
                    return None;
                }
            }
            Octree::Boxes { values, bx:_ } =>{
                for i in values.iter(){
                    if let Some(out) = i.query_point(point){
                        return Some(out)
                    }
                }
                return None
            }
        }
    }
    pub fn query_box(&self, bb:BoundingBox)->Vec<&[usize]>{
        match self{
            Octree::Values { values, bx:_ } =>{
                return vec![values.as_ref()]
            }
            Octree::Boxes { values, bx } =>{
                if !bx.check_collision(&bb){
                    return vec![];
                }
                let mut out = Vec::new();
                for i in values.iter(){
                    let tmp = i.query_box(bb);
                    out.reserve(tmp.len());
                    for j in tmp{
                        out.push(j);
                    }

                }
                return out;
            }
        }
    }
}
pub fn make_octree<'a>(arena:&'a Arena,values:&[usize],phys:&[Option<PhysicsComp>], trans:&[Option<TransformComp>], bb:BoundingBox, depth:usize)->Octree<'a>{
    let mut quads = [const{Vec::new()};8];
    if values.len()<4|| (bb.max-bb.min).length()<0.1{
        let mut vs = AVec::new(arena);
        for i in values{
            vs.push(*i);
        }
        return Octree::Values { values: vs ,bx:bb}
    }
    let bbs = bb.subdivide();
    for i in values{
        if let Some(id) = phys[*i].as_ref(){
            let trans = trans[*i].as_ref().unwrap();
            let b2 = BoundingBox{min:id.min(trans.trans), max:id.max(trans.trans)};
            //let b3 = BoundingBox{min:id.min(trans.trans)+id.velocity*1./60., max:id.max(trans.trans)+id.velocity*1./60.}.scale(1.0);
            if !bb.check_collision(&b2){
                continue;
            }
            for j in 0..bbs.len(){
                if bbs[j].check_collision(&b2){
                    quads[j].push(*i);
                }
            }
        }
    }
    let mlt = 2.0;
    if depth  <1{
        let should_thread = true;
        if should_thread{
            return std::thread::scope(|f|{
                let p1 = f.spawn(||{
                    (make_octree(arena, &quads[0], phys, trans, bbs[0], depth+1), make_octree(arena, &quads[1], phys, trans, bbs[1], depth+1))
                });
                let p2 = f.spawn(||{
                    (make_octree(arena, &quads[2], phys, trans, bbs[2], depth+1), make_octree(arena, &quads[3], phys, trans, bbs[3], depth+1))
                });
                let p3 = f.spawn(||{
                    (make_octree(arena, &quads[4], phys, trans, bbs[4], depth+1), make_octree(arena, &quads[5], phys, trans, bbs[5], depth+1))
                });
                let p4 = f.spawn(||{
                    (make_octree(arena, &quads[6], phys, trans, bbs[6], depth+1), make_octree(arena, &quads[7], phys, trans, bbs[7], depth+1))
                });
                let p1s = p1.join().unwrap();
                let p2s = p2.join().unwrap();
                let p3s = p3.join().unwrap();
                let p4s = p4.join().unwrap();
                let vs = [p1s.0, p1s.1, p2s.0, p2s.1, p3s.0, p3s.1, p4s.0, p4s.1];
                return Octree::Boxes { values: arena.alloc(vs), bx: bb.scale(mlt) };
            });
        } else{
            let p1s =  (make_octree(arena, &quads[0], phys, trans, bbs[0], depth+1), make_octree(arena, &quads[1], phys, trans, bbs[1], depth+1));
            let p2s =            (make_octree(arena, &quads[4], phys, trans, bbs[4], depth+1), make_octree(arena, &quads[5], phys, trans, bbs[5], depth+1));
            let p3s =             (make_octree(arena, &quads[4], phys, trans, bbs[4], depth+1), make_octree(arena, &quads[5], phys, trans, bbs[5], depth+1));
            let p4s =             (make_octree(arena, &quads[6], phys, trans, bbs[6], depth+1), make_octree(arena, &quads[7], phys, trans, bbs[7], depth+1));
            let vs = [p1s.0, p1s.1, p2s.0, p2s.1, p3s.0, p3s.1, p4s.0, p4s.1];
            return Octree::Boxes { values: arena.alloc(vs), bx: bb.scale(mlt) };
        }

    } 
    let values= [
        make_octree(arena, &quads[0], phys, trans, bbs[0], depth+1), 
        make_octree(arena,&quads[1], phys, trans, bbs[1],depth+1), 
        make_octree(arena, &quads[2], phys, trans, bbs[2],depth+1),
        make_octree(arena,&quads[3], phys, trans, bbs[3], depth+1),
        make_octree(arena,&quads[4], phys, trans, bbs[4], depth+1),
        make_octree(arena,&quads[5], phys, trans, bbs[5], depth+1), 
        make_octree(arena,&quads[6], phys, trans, bbs[6], depth+1),
        make_octree(arena,&quads[7], phys, trans, bbs[7], depth+1)
    ];
    Octree::Boxes { values: arena.alloc(values), bx:bb.scale(mlt)}

}
pub fn make_octree_shallow<'a>(arena:&'a Arena,values:&[usize],_phys:&[Option<PhysicsComp>], _trans:&[Option<TransformComp>], bb:BoundingBox)->Octree<'a>{
   todo!()// Octree::Values { values:values.to_vec() , bx:bb }
}
pub fn check_collision_comps(phys_a:&PhysicsComp, a_trans:&TransformComp, phys_b:&PhysicsComp, b_trans:&TransformComp)->Option<Col>
{
    for i in &phys_a.collisions{
        for j in &phys_b.collisions{
            if let Some(c) = col::check_collision(i.col, i.offset, a_trans.clone(), j.col, j.offset, b_trans.clone()){
                return Some(c)
            }
        }
    }
    None
}

static mut PHYS_ARENA:Option<Box<Arena>> = None;
pub fn update(dt:f64){
    unsafe{
        #[allow(static_mut_refs)]
        if PHYS_ARENA.is_none(){
            PHYS_ARENA = Some(Arena::new_sized(4096*4096*512));
        }
    }
    #[allow(static_mut_refs)]
    let arena =unsafe{PHYS_ARENA.as_ref().unwrap()};
    static UPDATE_TRANSFORMS:Mutex<usize> = Mutex::new(0);
    let trans_ref = unsafe{arena.alloc_array_no_destructor(get_level().transform_comps.list.write().unwrap().as_ref())};
    let phys_ref = unsafe{arena.alloc_array_no_destructor(get_level().physics_comps.list.write().unwrap().as_ref())};
    *SAFE_TO_TAKE.lock().unwrap() = true;
    let phys = phys_ref.as_mut();
    let trans = trans_ref.as_mut();
    let mut iter:Vec<usize> = Vec::new();
    let  mut max_v = -Vector3::new(10000.0, 10000.0, 10000.0)/10.;
    let mut min_v = -max_v;
    for i in 0..phys.len(){
        if let Some(k) = &mut phys[i]{
            if !k.can_ever_move{
                continue;
            }
            let t = trans[i].clone().unwrap();
            let bb = k.bb(t.trans);
            if bb.max.x>max_v.x{
                max_v.x = bb.max.x;
            }
            if bb.max.y>max_v.y{
                max_v.y = bb.max.y;
            }
            if bb.max.z>max_v.z{
                max_v.z = bb.max.z;
            }
            if bb.min.x<min_v.x{
                min_v.x = bb.min.x;
            }
            if bb.min.x<min_v.x{
                min_v.x = bb.min.x;
            }
            if bb.min.y<min_v.y{
                min_v.y = bb.min.y;
            }
            if bb.min.z<min_v.z{
                min_v.z= bb.min.z;
            }
            k.collided_this_frame  =false;
            iter.push(i);
        }
    }
    let oct = make_octree(&arena,&iter, phys, trans, BoundingBox{min:min_v, max:max_v},0);
    for i in &iter{
        let a_phys = phys[*i].as_mut().unwrap();
        let mut a_trans = trans[*i].as_ref().unwrap().clone();
        let old = a_trans.clone();
        if a_phys.velocity.length()>=C{
            a_phys.velocity = a_phys.velocity.normalized()*0.9999*C;
        }
        let a_phys = phys[*i].as_ref().unwrap();
        a_trans.trans.translation += a_phys.velocity*dt/a_phys.gamma();
        let itr:Vec<&usize> = oct.query_box(a_phys.bb(a_trans.trans)).into_iter().flatten().collect();
        for j in itr{
            if *j == *i{
                continue;
            }
            let b_phys = phys[*j].as_ref().unwrap();
            let b_trans = trans[*j].as_ref().unwrap();
            if let Some (c) = check_collision_comps(&a_phys.clone(), &a_trans, b_phys, b_trans){
                let p_i = phys[*i].as_mut().unwrap();
                p_i.collided_this_frame = true;
                p_i.velocity.reflect(c.norm);
                let p_j = phys[*j].as_mut().unwrap();
                p_j.velocity.reflect(c.norm);
                p_j.collided_this_frame = true;
                a_trans.trans.translation = old.trans.translation+c.norm.normalized()*c.depth;
                break;
            }
        }
        trans[*i].as_mut().unwrap().trans = a_trans.trans;
    }
    let mut trans_lock = UPDATE_TRANSFORMS.lock().unwrap();
    if *trans_lock == UPDATE_FREQ-1{
        for i in &iter{
            if let Some(i) = trans[*i].as_mut(){
                i.update();
            }
        }
        *trans_lock = 0;
    } else{
        *trans_lock +=1;
    }

    let mut  phys_ref = get_level().physics_comps.list.write().unwrap();
    let mut trans_ref = get_level().transform_comps.list.write().unwrap();
    unsafe{
        for i in 0..phys_ref.len(){
            phys_ref[i] = std::mem::transmute_copy(&phys[i]);
        }
        for i in 0..trans_ref.len(){
            trans_ref[i]=  std::mem::transmute_copy(&trans[i]);
        }
        arena.reset();
    }
}
pub fn create_box(pos:Vector3, vel:Vector3, tint:Color)->Entity{
    let out = create_entity().unwrap();
    let mut cmp = PhysicsComp::new();
    let sz =0.05;
    cmp.velocity = vel;
    cmp.collisions.push(Collision { col: BoundingBox{min:Vector3::new(-sz, -sz, -sz ), max:Vector3::new(sz, sz, sz)}, offset:Transform::default(), entity_ref: None });
    add_physics_comp(out, cmp);
    let mut trans  = TransformComp::new();
    trans.trans.translation = pos;
    add_transform_comp(out, trans);
    add_model_comp(out, ModelComp{models:vec![ModelData{model: "box".to_string(), diffuse:"".to_string(), normal:"".to_string(), tint, offset:Transform::default(), parent:None}]});
    out
}
pub fn create_box_stationary(pos:Vector3, vel:Vector3, tint:Color)->Entity{
    let out = create_entity().unwrap();
    let mut cmp = PhysicsComp::new();
    let sz =0.05;
    cmp.velocity = vel;
    cmp.can_ever_move = false;
    cmp.collisions.push(Collision { col: BoundingBox{min:Vector3::new(-sz, -sz, -sz ), max:Vector3::new(sz, sz, sz)}, offset:Transform::default(), entity_ref: None });
    add_physics_comp(out, cmp);
    let mut trans  = TransformComp::new();
    trans.trans.translation = pos;
    add_transform_comp(out, trans);
    add_model_comp(out, ModelComp{models:vec![ModelData{model: "box".to_string(), diffuse:"".to_string(), normal:"".to_string(), tint, offset:Transform::default(), parent:None}]});
    out
}
