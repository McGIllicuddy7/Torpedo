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
}

impl Collider3D {
    pub fn as_vertices(&self) -> [Vector3; 8] {
        [
            self.pos
                + Vector3::new(-self.width / 2., -self.height / 2., -self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(-self.width / 2., self.height / 2., -self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(self.width / 2., -self.height / 2., -self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(self.width / 2., self.height / 2., -self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(-self.width / 2., -self.height / 2., self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(-self.width / 2., self.height / 2., self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(self.width / 2., -self.height / 2., self.depth / 2.)
                    .rotate_by(self.rotation),
            self.pos
                + Vector3::new(self.width / 2., self.height / 2., self.depth / 2.)
                    .rotate_by(self.rotation),
        ]
    }

    pub fn sap_vectors(&self) -> [Vector3; 26] {
        [
            Vector3::new(-self.width / 2., -self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(-self.width / 2., self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., -self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(-self.width / 2., -self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(-self.width / 2., self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., -self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            //tmp
            Vector3::new(-self.width / 2., 0., 0.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., 0., 0.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0., -self.height / 2., 0.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0., self.height / 2., 0.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0., 0., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0., 0., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            //tmp2
            Vector3::new(-self.width / 2., -self.height / 2., 0.0)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(-self.width / 2., self.height / 2., 0.0)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., -self.height / 2., 0.0)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., self.height / 2., 0.0)
                .rotate_by(self.rotation)
                .normalized(),
            //tmp3
            Vector3::new(0.0, -self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0.0, -self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0.0, self.height / 2., -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(0.0, self.height / 2., self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            //tmp4
            Vector3::new(-self.width / 2., 0.0, -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(-self.width / 2., 0.0, self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., 0.0, -self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
            Vector3::new(self.width / 2., 0.0, self.depth / 2.)
                .rotate_by(self.rotation)
                .normalized(),
        ]
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
                if d < smin {
                    omin = d;
                }
            }
            if smin < omin && smax < omin || smax > omax && smin > omax {
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
            })
        } else {
            None
        }
    }
}
