use crate::{level::{Entity, TransformComp}, math::{BoundingBox, Transform, Vector3}};

use super::Col;
#[allow(unused)]
fn get_vertices(a:BoundingBox,offset:Transform, a_trans:Transform)->[Vector3;8]{
    let mut verts= [Vector3::new(1.,1., 1.), Vector3::new(1., -1., 1.), Vector3::new(-1., 1., 1.), Vector3::new(-1., -1., 1.0), 
    Vector3::new(1.,1., -1.), Vector3::new(1., -1., -1.), Vector3::new(-1., 1., -1.), Vector3::new(-1., -1., -1.0)
    ];
    let dx = a.max.x-a.min.x;
    let dy = a.max.y-a.min.y;
    let dz = a.max.z-a.min.z;
    for i in &mut verts{
        let x = i.x*dx/2.;
        let y = i.y*dy/2.;
        let z = i.z*dz/2.;
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
#[allow(unused)]
const fn vec_contains(a:&[Vector3], v:Vector3)->bool{
    let mut idx = 0;
    while idx<a.len(){
        if a[idx].x == v.x && a[idx].y == v.y && a[idx].z == v.z{
            return true;
        }
        idx += 1;
        assert!(idx != 0);
    }
    false
}
#[allow(unused)]
fn get_normals(a_trans:Transform, a_off:Transform)->[Vector3;13]{
    #[allow(long_running_const_eval)]
    let mut normals = const{
        let mut norms = [const{Vector3::new(0., 0., 0.,)};13];
        let mut count = 0;
        let mut x = -1;
        let mut y = -1;
        let mut z = -1;
        while x<2{
            while y<2{
                while z<2{
                    if x == 0 && y == 0 && z == 0{
                        z+= 1;
                        continue;
                    }
                    let v = Vector3::new(x as f64, y as f64 ,z as f64);
                    if vec_contains(&norms, Vector3::new(-v.x, -v.y, -v.z)){
                        z+= 1;
                        continue;
                    }
                    norms[count] =v;
                    count += 1;
                    z+= 1;
                }
                z = -1;
                y += 1;
            }
            y = -1;
            x += 1;
        }
        let mut idx = 0;
        while idx<norms.len(){
            let l = norms[idx].x*norms[idx].x + norms[idx].y *norms[idx].y+norms[idx].z* norms[idx].z;
            norms[idx].x /= l;
            norms[idx].y /= l;
            norms[idx].x /= l;
            idx += 1;
        }
        norms
    };
    let rot = (a_trans.rotation*a_off.rotation).to_matrix();
    for i in &mut normals{
        i.transform(rot);
    }
    normals
}
#[allow(unused)]
fn get_normals_basic(a_trans:Transform, a_off:Transform)->[Vector3;6]{
    let mut normals = [Vector3::new(1.0, 0., 0.), Vector3::new(-1., 0., 0.), Vector3::new(0., 1., 0.), Vector3::new(0., -1., 0.), Vector3::new(0., 0., 1.), Vector3::new(0., 0.,-1.0)];
    let rot = (a_trans.rotation*a_off.rotation).to_matrix();
    for i in &mut normals{
        i.transform(rot);
    }
    for i in &mut normals{
        i.normalize();
    }
    normals
}
#[allow(unused)]
pub fn check_collision(a:BoundingBox, a_off:Transform,a_trans:TransformComp, b:BoundingBox,b_off:Transform, b_trans:TransformComp)->Option<Col>{
    {
        let a_lock = a_off.translation.transform_with(a_trans.trans.rotation.to_matrix())+a_trans.trans.translation;
        let b_lock = b_off.translation.transform_with(b_trans.trans.rotation.to_matrix())+b_trans.trans.translation;
        let a = BoundingBox { min: a.min+a_lock, max: a.max+a_lock };
        let b= BoundingBox { min: b.min+b_lock, max: b.max+b_lock };
        if !a.check_collision(&b){
            return None;
        }
    }
    let a_verts = get_vertices(a, a_off,a_trans.trans);
    let b_verts = get_vertices(b, b_off,b_trans.trans);
    let a_norms = get_normals(a_trans.trans,a_off);
    let b_norms = get_normals(b_trans.trans, b_off);
    let mut norms = [const{Vector3::new(0., 0., 0.,)}; 26];
    let mut idx = 0;
    for i in a_norms{
        norms[idx] = i;
        idx +=1;
    }
    for i in b_norms{
        norms[idx] = i;
        idx +=1;
    }
    let mut col_norm = Vector3::new(0., 0., 0.);
    let mut col_depth = 1000000.0;
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
        if a_min>b_max+0.0001 || b_min>a_max+0.0001{
           return None;
        } 
    }
    idx = 0;
    let a_norms = get_normals_basic(a_trans.trans, a_off);
    let b_norms = get_normals_basic(b_trans.trans, a_off);
    let mut trans = [const{crate::math::Vector3::new(0., 0., 0.,)}; 12];
    for i in a_norms{
        trans[idx] = i;
        idx += 1;
    }
    for i in b_norms{
        trans[idx] = i;
        idx += 1;
    }
    for i in trans{
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
        let da =(b_min-a_max).abs();
        let db = (a_min-b_max).abs();
        let del = if da>db{
            db
        } else{
            da
        };
        if del<col_depth{
            col_depth = del;
            col_norm = i;
        }
    }

    Some(Col{hit_ref:Entity{idx:0, generation:0}, norm:col_norm.normalized(), depth:col_depth})
}
