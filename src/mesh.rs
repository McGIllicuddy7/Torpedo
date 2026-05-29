use std::f32::consts::PI;

use raylib::math::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameMesh {
    pub points: Vec<Vector3>,
    pub lines: Vec<(u16, u16)>,
}

impl Default for GameMesh {
    fn default() -> Self {
        Self::new()
    }
}

impl GameMesh {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn add_cube(&mut self, start: Vector3, width: f32, height: f32, depth: f32) {
        let mut points = vec![
            Vector3::new(-depth / 2., -width / 2., -height / 2.),
            Vector3::new(-depth / 2., -width / 2., height / 2.),
            Vector3::new(-depth / 2., width / 2., -height / 2.),
            Vector3::new(-depth / 2., width / 2., height / 2.),
            Vector3::new(depth / 2., -width / 2., -height / 2.),
            Vector3::new(depth / 2., -width / 2., height / 2.),
            Vector3::new(depth / 2., width / 2., -height / 2.),
            Vector3::new(depth / 2., width / 2., height / 2.),
        ];
        let base = self.points.len();
        let mut connections = Vec::new();
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                if points[i].distance_to(points[j]) <= 1. {
                    connections.push(((i + base) as u16, (j + base) as u16));
                }
            }
        }

        for i in &mut points {
            *i += start;
        }
        for i in points {
            self.points.push(i);
        }
        for i in connections {
            self.lines.push(i);
        }
    }

    pub fn add_face(&mut self, points: &[Vector3]) {
        for i in points {
            if !self.points.contains(i) {
                self.points.push(*i);
            }
        }
        let mut list: Vec<(u16, u16)> = Vec::new();
        for i in 0..points.len() {
            let j = (i + 1) % points.len();
            let p0 = points[i];
            let mut idx0 = 0;
            let p1 = points[j];
            let mut idx1 = 0;
            for idx in 0..self.points.len() {
                if self.points[idx].distance_to(p0) < 0.01 {
                    idx0 = idx;
                }
                if self.points[idx] == p1 {
                    idx1 = idx;
                }
            }
            list.push((idx0 as u16, idx1 as u16));
        }
        for i in list {
            self.lines.push(i);
        }
    }

    pub fn add_cylinder(
        &mut self,
        lower_radius: f32,
        upper_radius: f32,
        height: f32,
        segment_count: i32,
        base: Vector3,
        direction: Vector3,
        offset_direction: Vector3,
    ) {
        let dtheta = 2. * PI / (segment_count as f32);
        let ofdir = offset_direction.normalized();
        let updir = direction.normalized();
        for i in 0..segment_count {
            let theta0 = i as f32 * dtheta;
            let theta1 = ((i + 1) % (segment_count)) as f32 * dtheta;
            let v0 = ofdir.rotate_by(Quaternion::from_axis_angle(updir, theta0));
            let v1 = ofdir.rotate_by(Quaternion::from_axis_angle(updir, theta1));
            let points = [
                v0 * lower_radius + base,
                v1 * lower_radius + base,
                v1 * upper_radius + base + updir * height,
                v0 * upper_radius + base + updir * height,
            ];
            self.add_face(&points);
        }
    }
}
