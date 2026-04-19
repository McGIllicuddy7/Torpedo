use std::f32::consts::TAU;

use rand::Rng;
use raylib::{
    color::Color,
    math::{Vector2, Vector3, Vector4},
};

use crate::system::{
    Entity, EntityComponent, create_entity, get_star_data, mark_graphics_should_update,
    random_float, random_position,
};

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

pub fn new_cube(at: Vector3, height: f32, width: f32, depth: f32) -> Entity {
    let out = create_entity();
    let mut g = out.write();
    g.self_id = out;
    g.is_static = true;
    g.kind = crate::EntityKind::Object;
    g.position = at;
    g.rotation = Vector4::identity();
    let mut body = EntityComponent::default();
    body.render_as = crate::system::RenderKind::Cube;
    body.color = Color::GRAY;
    body.health = (height * width * depth * 6.8) as u32;
    body.width = width;
    body.height = height;
    body.depth = depth;
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
    new_cube(Vector3::zero(), 1., 100.0, 100.);
    new_cube(Vector3::up() * 20., 10., 20., 20.);
    new_cube(-Vector3::up() * 20., 10., 20., 20.);
    for _ in 0..5 {
        let pos = Vector3::new(
            ((random_float()) * 2. - 1.) * 100.0,
            ((random_float()) * 2. - 1.) * 100.0,
            ((random_float()) * 2. - 1.) * 100.0,
        );
        if pos.y.abs() < 10. {
            continue;
        }
        new_cube(pos, 10., 10., 10.);
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
    mark_graphics_should_update();
}
