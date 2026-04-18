use std::f32::consts::TAU;

use rand::Rng;
use raylib::{
    color::Color,
    math::{Vector2, Vector3, Vector4},
};

use crate::system::{Entity, EntityComponent, create_entity, get_star_data};

pub fn new_asteroid(at: Vector3, size: f32) -> Entity {
    let out = create_entity();
    let mut g = out.write();
    g.self_id = out;
    g.is_static = true;
    g.kind = crate::EntityKind::Object;
    g.position = at;
    g.rotation = Vector4::identity();
    let mut body = EntityComponent::default();
    body.render_as = crate::system::RenderKind::Sphere;
    body.color = Color::GRAY;
    body.health = (size * size * size * 6.8) as u32;
    body.width = size;
    body.height = size;
    body.depth = size;
    g.component_table.insert("body".into(), body);
    out
}

pub fn new_sun(at: Vector3, size: f32) -> Entity {
    let out = create_entity();
    let mut g = out.write();
    g.self_id = out;
    g.is_static = true;
    g.kind = crate::EntityKind::Object;
    g.position = at;
    g.rotation = Vector4::identity();
    let mut body = EntityComponent::default();
    body.render_as = crate::system::RenderKind::Sphere;
    body.color = Color::YELLOW;
    body.health = (size * size * size * 6.8) as u32;
    body.width = size;
    body.height = size;
    body.depth = size;
    g.component_table.insert("body".into(), body);
    out
}

pub fn create_solar_system() {
    for _ in 0..256 {
        let pos = random_position();
        new_asteroid(pos, (random_float() + 0.25));
    }
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    for _ in 0..250 {
        let p = Vector2::new(random_float(), random_float());
        let r = random_float() * 0.00001;
        l1.push(p);
        l2.push(r);
    }
    let mut data = get_star_data();
    data.0 = l1.into_boxed_slice();
    data.1 = l2.into_boxed_slice();
}

pub fn random_float() -> f32 {
    let tmp = rand::rng().next_u32() % 1_000_000;
    tmp as f32 / 1_000_000.0
}

pub fn random_position() -> Vector3 {
    let radius = (random_float() + 0.25) * 50.0;
    let phi = random_float() * TAU;
    let theta = random_float() * TAU;
    from_spherical(radius, phi, theta)
}

pub fn from_spherical(r: f32, theta: f32, phi: f32) -> Vector3 {
    let x = r * theta.sin() * phi.cos();
    let y = r * theta.sin() * phi.sin();
    let z = r * theta.cos();
    Vector3::new(x, y, z)
}

pub fn to_spherical(v: Vector3) -> (f32, f32, f32) {
    let theta = v.y.atan2(v.x);
    let r = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    let phi = (v.z / r).acos();
    (r, theta, phi)
}
