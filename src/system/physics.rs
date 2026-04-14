use std::sync::Arc;

use raylib::math::{BoundingBox, Quaternion, Ray, Vector3};

use crate::system::Entity;

#[derive(Clone, Debug)]
pub struct Collider3D {
    pub pos: Vector3, //center position
    pub rotation: Quaternion,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub velocity: Vector3,
    pub mass: f32,
    pub parent_entity: Entity,
    pub parent_name: Arc<str>,
}

#[derive(Clone, Copy, Debug)]
pub struct ColData3D {
    pub pos: Vector3,
    pub normal: Vector3,
    pub dist: f32,
    pub hit_entity: Entity,
}

impl Collider3D {
    pub fn as_vertices(&self) -> [Vector3; 8] {
        let mut idx = 0;
        let base = Vector3::new(self.width / 2., self.height / 2., self.depth / 2.);
        let mut out = [base; 8];
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x != 0 && y != 0 && z != 0 {
                        let mut next = base;
                        next.x *= x as f32;
                        next.y *= y as f32;
                        next.z *= z as f32;
                        out[idx] = next;
                        idx += 1;
                    }
                }
            }
        }
        out
    }

    pub fn sap_vectors(&self) -> [Vector3; 26] {
        let mut out = [Vector3::zero(); 26];
        let mut idx = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    out[idx] = Vector3::new(dx as f32, dy as f32, dz as f32)
                        .normalized()
                        .rotate_by(self.rotation);
                    idx += 1;
                }
            }
        }
        out
    }

    pub fn check_collision(&self, other: &Self) -> bool {
        let norms = [self.sap_vectors(), other.sap_vectors()];
        let sverts = self.as_vertices();
        let overts = other.as_vertices();
        for i in norms.iter().flatten() {
            let mut smin = sverts[0].dot(*i);
            let mut smax = smin;
            let mut omin = overts[0].dot(*i);
            let mut omax = omin;
            for j in 1..8 {
                let d = sverts[j].dot(*i);
                if d > smax {
                    smax = d;
                }
                if d < smin {
                    smin = d;
                }
            }
            for j in 1..8 {
                let d = overts[j].dot(*i);
                if d > omax {
                    omax = d;
                }
                if d < omin {
                    omin = d;
                }
            }
            if (smin < omin && smax < omin) || (smax > omax && smin > omax) {
                if self.pos.distance_to(other.pos) < 1.0 {
                    println!(
                        " self.vertices:{:#?}, other.vertices:{:#?}, self.pos:{:#?}, other.pos:{:#?},omin:{}, omax:{}, smin:{}, smax:{},",
                        sverts, overts, self.pos, other.pos, omin, omax, smin, smax,
                    );
                    // todo!()
                }
                return false;
            }
        }
        true
    }

    pub fn normals(&self) -> [Vector3; 6] {
        [
            Vector3::new(-1., 0., 0.).rotate_by(self.rotation),
            Vector3::new(1., 0., 0.).rotate_by(self.rotation),
            Vector3::new(0., -1., 0.).rotate_by(self.rotation),
            Vector3::new(0., 1., 0.).rotate_by(self.rotation),
            Vector3::new(0., 0., -1.).rotate_by(self.rotation),
            Vector3::new(0., 0., 1.).rotate_by(self.rotation),
        ]
    }

    pub fn raycast(&self, start: Vector3, direction: Vector3) -> Option<ColData3D> {
        let start = start - self.pos;
        let trans = Quaternion::inverted(&self.rotation);
        let dir = direction.rotate_by(trans);
        let bounds = BoundingBox::new(
            Vector3::new(-self.width / 2.0, -self.height / 2.0, -self.depth / 2.0),
            Vector3::new(self.width / 2.0, self.height / 2.0, self.depth / 2.0),
        );
        let c = bounds.get_ray_collision_box(Ray {
            position: start,
            direction: dir,
        });
        if c.hit {
            let n = c.normal.rotate_by(self.rotation);
            let pos = c.point.rotate_by(self.rotation) + self.pos;
            Some(ColData3D {
                pos,
                normal: n,
                dist: c.distance,
                hit_entity: Entity::default(),
            })
        } else {
            None
        }
    }
}
