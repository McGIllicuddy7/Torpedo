use std::{
    ops::Deref,
    sync::{Arc, MutexGuard},
};

use rand::random;
use raylib::{
    RaylibHandle, RaylibThread,
    camera::Camera3D,
    color::Color,
    drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt},
    math::{Quaternion, Vector3},
};
use serde::{Deserialize, Serialize};

use crate::engine::{ENGINE, GameMode, get_engine, random_unit_vector, random_vector};

pub enum DrawEvent {
    DrawText {
        x: i32,
        y: i32,
        height: i32,
        contents: Arc<str>,
        color: Color,
    },
    DrawRectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: Color,
    },

    DrawCircle {
        x: i32,
        y: i32,
        radius: f32,
        color: Color,
    },
    DrawLine {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Color,
    },
}
pub enum DrawEvent3D {
    DrawPoint {
        point: Vector3,
        color: Color,
    },
    DrawLine {
        start: Vector3,
        end: Vector3,
        color: Color,
    },
}

pub fn run_graphics(
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    game_mode: &mut dyn GameMode,
) {
    let mut c = ENGINE.camera_data.lock().unwrap();
    let mut cm = Camera3D::perspective(c.position, c.target, c.up, 90.);
    handle.update_camera(&mut cm, raylib::ffi::CameraMode::CAMERA_FREE);
    c.position = cm.position;
    c.target = cm.target;
    c.up = cm.up;
    {
        let tmp = ENGINE.player_object.lock().unwrap();
        if let Some(g) = tmp.get_checked() {
            let data = g.get_data();
            if let Some(y) = data.camera_data.as_ref() {
                let brot = data.rotation.inverted() * y.rotation;
                let bpos = data.location + y.position.rotate_by(data.rotation.inverted());
                let target = Vector3::new(1.0, 0.0, 0.0).rotate_by(brot) + data.location;
                let up = Vector3::new(0.0, 0.0, 1.0).rotate_by(brot);
                cm.position = bpos;
                cm.target = target;
                cm.up = up;
            }
        }
    }
    let mut draw = handle.begin_drawing(&thread);
    draw.clear_background(Color::BLACK);
    let mut draw3d = draw.begin_mode3D(&cm);
    for i in &ENGINE.objects {
        let t = i.read().unwrap();
        if let Some(t2) = t.ptr.as_ref() {
            let data = t2.get_data();
            let trans = data.rotation.to_matrix();
            if let Some(md) = data.model.as_ref() {
                let mut points = md.points.clone();
                for i in &mut points {
                    *i = i.transform_with(trans);
                    *i += data.location;
                }
                for (start, end) in &md.lines {
                    let start = points[*start as usize];
                    let end = points[*end as usize];
                    let dist =
                        (cm.position.distance_to(start) + cm.position.distance_to(end)) / (20.);
                    let colb = Color::GREEN;
                    let vc =
                        Vector3::new(colb.r as f32, colb.g as f32, colb.b as f32) / (dist.sqrt());
                    if vc.length() < 20. {
                        continue;
                    }
                    let cl = Color {
                        r: vc.x as u8,
                        g: vc.y as u8,
                        b: vc.z as u8,
                        a: 255,
                    };
                    draw3d.draw_line_3D(start, end, cl);
                }
            }
        }
    }
    let mut calls = ENGINE.draw_events_3d.lock().unwrap();
    while let Some(c) = calls.pop_front() {
        match c {
            DrawEvent3D::DrawPoint { point, color } => {
                draw3d.draw_sphere_ex(point, 0.5, 3, 3, color);
            }
            DrawEvent3D::DrawLine { start, end, color } => {
                draw3d.draw_line_3D(start, end, color);
            }
        }
    }
    for i in 0..get_engine().particle_systems.len() {
        let g = get_engine().particle_systems[i].lock().unwrap();
        if let Some(y) = g.v.as_ref() {
            y.render(&mut draw3d, thread);
        }
    }
    drop(draw3d);
    drop(calls);
    let mut calls = ENGINE.draw_events.lock().unwrap();
    while let Some(c) = calls.pop_front() {
        match c {
            DrawEvent::DrawText {
                x,
                y,
                height,
                contents,
                color,
            } => {
                draw.draw_text(&contents, x, y, height, color);
            }
            DrawEvent::DrawRectangle {
                x,
                y,
                width,
                height,
                color,
            } => {
                draw.draw_rectangle(x, y, width, height, color);
            }
            DrawEvent::DrawCircle {
                x,
                y,
                radius,
                color,
            } => {
                draw.draw_circle(x, y, radius, color);
            }
            DrawEvent::DrawLine {
                x0,
                y0,
                x1,
                y1,
                color,
            } => {
                draw.draw_line(x0, y0, x1, y1, color);
            }
        }
    }
    game_mode.on_render(&mut draw, thread);
    draw.draw_fps(1600, 20);
}

pub fn draw_text(text: &str, x: i32, y: i32, font_size: i32, color: Color) {
    get_engine()
        .draw_events
        .lock()
        .unwrap()
        .push_back(DrawEvent::DrawText {
            x,
            y,
            height: font_size,
            contents: text.into(),
            color,
        });
}
pub fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    get_engine()
        .draw_events
        .lock()
        .unwrap()
        .push_back(DrawEvent::DrawLine {
            x0,
            y0,
            x1,
            y1,
            color,
        });
}

pub fn draw_rectangle(x: i32, y: i32, width: i32, height: i32, color: Color) {
    get_engine()
        .draw_events
        .lock()
        .unwrap()
        .push_back(DrawEvent::DrawRectangle {
            x,
            y,
            width,
            height,
            color,
        });
}

pub fn draw_circle(x: i32, y: i32, radius: f32, color: Color) {
    get_engine()
        .draw_events
        .lock()
        .unwrap()
        .push_back(DrawEvent::DrawCircle {
            x,
            y,
            radius,
            color,
        });
}

pub fn draw_point(point: Vector3, color: Color) {
    get_engine()
        .draw_events_3d
        .lock()
        .unwrap()
        .push_back(DrawEvent3D::DrawPoint { point, color });
}

pub fn draw_line_3d(start: Vector3, end: Vector3, color: Color) {
    get_engine()
        .draw_events_3d
        .lock()
        .unwrap()
        .push_back(DrawEvent3D::DrawLine { start, end, color });
}

#[derive(Clone, Copy, Debug)]
pub struct ParticleSystem {
    pub particles: [Option<Particle>; 2048],
    pub position: Vector3,
    pub velocity: Vector3,
    pub spawn_data: SpawnData,
    pub show_particles: bool,
    pub force_fields: [Option<ForceField>; 8],
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub position: Vector3,
    pub velocity: Vector3,
    pub connections: [i16; 32],
    pub radius: f32,
    pub start_color: Color,
    pub end_color: Color,
    pub glowing: bool,
    pub starting_lifetime: f32,
    pub lifetime: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SpawnData {
    pub probability_to_spawn_per_second: f32,
    pub amount_to_spawn: u32,
    pub spawn_radius: f32,
    pub velocity_info: SpawnVelocityData,
    pub max_connection_count: u32,
    pub max_connection_distance: f32,
    pub min_lifetime: f32,
    pub max_lifetime: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub spawning_duration: f32,
    pub can_stop_spawning: bool,
    pub ending_color: Color,
    pub glowing: bool,
    pub color: Color,
}

#[derive(Clone, Copy, Debug)]
pub enum ForceField {
    Global { force: Vector3 },
    Point { offset: Vector3, amount: f32 },
    Drag { coef: f32 },
}
#[derive(Clone, Copy, Debug)]
pub enum SpawnVelocityData {
    Cone {
        direction: Vector3,
        max_angle: f32,
        length: f32,
    },
    Sphere {
        radius: f32,
    },
}

impl ParticleSystem {
    pub fn update(&mut self, dt: f32) {
        let t0 = self.particles;
        for i in &mut self.particles {
            let mut should_destroy = false;
            if let Some(i) = i.as_mut() {
                i.lifetime -= dt;
                if i.lifetime < 0.0 {
                    should_destroy = true;
                }
                let mut f = Vector3::zero();
                for j in &self.force_fields {
                    if let Some(j) = j.as_ref() {
                        match j {
                            ForceField::Global { force } => {
                                f += *force;
                            }
                            ForceField::Point { offset, amount } => {
                                let pos = *offset + self.position;
                                let mut distance = i.position.distance_to(pos);

                                let dir = if distance < 0.01 {
                                    random_unit_vector()
                                } else {
                                    (pos - i.position).normalized()
                                };
                                if distance == 0.0 {
                                    distance = 0.01;
                                }
                                f += dir * *amount / distance;
                            }
                            ForceField::Drag { coef } => {
                                f -= i.velocity * *coef;
                            }
                        }
                    }
                }
                i.position += i.velocity * dt + f * 0.5 * dt * dt;
                i.velocity += f * dt;
            }
            if should_destroy {
                *i = None;
            }
        }
        for i in 0..self.particles.len() {
            if let Some(i) = self.particles[i].as_mut() {
                for j in &mut i.connections {
                    if *j >= 0 {
                        if t0[*j as usize].is_none() {
                            *j = -1;
                        }
                    }
                }
            }
        }
        let should_spawn = (dt * self.spawn_data.probability_to_spawn_per_second * 10000.) as u64
            > (random::<u64>() % 10000);
        self.position += self.velocity * dt;
        if !self.spawn_data.can_stop_spawning && self.spawn_data.spawning_duration > 0. {
            self.spawn_data.spawning_duration -= dt;
        }
        if should_spawn
            && (self.spawn_data.spawning_duration > 0.0 || !self.spawn_data.can_stop_spawning)
        {
            for _ in 0..self.spawn_data.amount_to_spawn {
                loop {
                    let idx = {
                        let mut tidx = -1;
                        for i in 0..self.particles.len() {
                            if self.particles[i].is_none() {
                                tidx = i as i16;
                                break;
                            }
                        }
                        if tidx < 0 {
                            break;
                        }
                        tidx as usize
                    };
                    let pos = self.position + random_vector() * self.spawn_data.spawn_radius;
                    let velocity = match self.spawn_data.velocity_info {
                        SpawnVelocityData::Cone {
                            direction,
                            max_angle,
                            length,
                        } => {
                            let thet0 =
                                (((random::<u64>() % 1000) as f32 - 500.) / 500.) * max_angle;
                            let thet1 =
                                (((random::<u64>() % 1000) as f32 - 500.) / 500.) * max_angle;
                            let r = Quaternion::from_euler(thet0, thet1, 0.0);
                            let vel = direction.rotate_by(r) * length;
                            vel
                        }
                        SpawnVelocityData::Sphere { radius } => random_vector() * radius,
                    };
                    let velocity = self.velocity + velocity;
                    let mut connections = [-1; _];
                    let try_count = (self.spawn_data.max_connection_count)
                        .clamp(0, connections.len() as u32)
                        as usize;
                    for i in 0..try_count {
                        let mut min_idx = -1;
                        let mut min_dist = 10000.;
                        for (j, k) in self.particles.iter_mut().enumerate() {
                            if connections.contains(&(j as i16)) {
                                continue;
                            }
                            if let Some(k) = k.as_mut() {
                                if k.position.distance_to(pos) < min_dist {
                                    min_dist = k.position.distance_to(pos);
                                    min_idx = j as i16;
                                }
                            }
                        }
                        if min_idx == -1 {
                            break;
                        }
                        if min_dist > self.spawn_data.max_connection_distance {
                            break;
                        }
                        connections[i] = min_idx;
                    }
                    let lt_lerp = (random::<u64>() % 1000) as f32 / 1000.;
                    let lt = (1. - lt_lerp) * self.spawn_data.min_lifetime
                        + lt_lerp * self.spawn_data.max_lifetime;
                    let rad_lerp = (random::<u64>() % 1000) as f32 / 1000.;
                    let rad = (1. - rad_lerp) * self.spawn_data.min_radius
                        + rad_lerp * self.spawn_data.max_radius;
                    self.particles[idx] = Some(Particle {
                        position: pos,
                        velocity,
                        connections,
                        start_color: self.spawn_data.color,
                        end_color: self.spawn_data.ending_color,
                        lifetime: lt,
                        radius: rad,
                        starting_lifetime: lt,
                        glowing: self.spawn_data.glowing,
                    });
                    break;
                }
            }
        }
    }

    pub fn render(&self, draw: &mut impl RaylibDraw3D, _thread: &RaylibThread) {
        for i in 0..self.particles.len() {
            if let Some(y) = self.particles[i].as_ref() {
                let col = {
                    let lerp = 1. - y.lifetime / y.starting_lifetime;
                    let r = (1. - lerp) * y.start_color.r as f32 + lerp * y.end_color.r as f32;
                    let g = (1. - lerp) * y.start_color.g as f32 + lerp * y.end_color.g as f32;
                    let b = (1. - lerp) * y.start_color.b as f32 + lerp * y.end_color.b as f32;
                    let a = (1. - lerp) * y.start_color.a as f32 + lerp * y.end_color.a as f32;
                    Color {
                        r: r as u8,
                        g: g as u8,
                        b: b as u8,
                        a: a as u8,
                    }
                };
                if self.show_particles {
                    let should_glow = (col.r >= 250 || col.g > 250 || col.b > 250);
                    if y.glowing && should_glow {
                        draw.draw_sphere_ex(
                            y.position,
                            y.radius * 0.8,
                            3,
                            3,
                            Color {
                                r: 255,
                                g: 255,
                                b: 255,
                                a: col.a,
                            },
                        );
                        draw.draw_sphere_ex(
                            y.position,
                            y.radius * 1.1,
                            6,
                            6,
                            Color {
                                r: col.r,
                                g: col.g,
                                b: col.b,
                                a: 100,
                            },
                        );
                    } else {
                        draw.draw_sphere_ex(y.position, y.radius, 3, 3, col);
                    }
                }
                for i in y.connections {
                    if i > -1 {
                        if let Some(x) = self.particles[i as usize].as_ref() {
                            draw.draw_line_3D(y.position, x.position, col);
                        }
                    }
                }
            }
        }
    }
    pub fn new(
        location: Vector3,
        velocity: Vector3,
        show_particles: bool,
        data: SpawnData,
    ) -> Self {
        Self {
            particles: [None; _],
            position: location,
            velocity,
            spawn_data: data,
            show_particles,
            force_fields: [None; _],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParticleSystemHandle {
    pub idx: u32,
    pub generation: u32,
}

pub struct ParticleSystemGuard<'a> {
    inner: MutexGuard<'a, ParticleSystemContainer>,
}
impl<'a> ParticleSystemGuard<'a> {
    pub fn get(&self) -> &ParticleSystem {
        self.inner.v.as_ref().unwrap()
    }
    pub fn get_mut(&mut self) -> &mut ParticleSystem {
        self.inner.v.as_mut().unwrap()
    }
}

pub struct ParticleSystemContainer {
    pub v: Option<ParticleSystem>,
    pub generation: u32,
}

impl ParticleSystemHandle {
    pub fn get_checked<'a>(&'a self) -> Option<ParticleSystemGuard<'a>> {
        let tmp = get_engine().particle_systems[self.idx as usize]
            .lock()
            .unwrap();
        if tmp.generation != self.generation {
            return None;
        }
        Some(ParticleSystemGuard { inner: tmp })
    }
    pub fn get<'a>(&'a self) -> ParticleSystemGuard<'a> {
        self.get_checked().unwrap()
    }
    pub const fn new() -> Self {
        Self {
            idx: 0,
            generation: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.generation == 0 {
            return false;
        }
        self.get_checked().is_some()
    }
}

pub fn create_particle_system(system: ParticleSystem) -> ParticleSystemHandle {
    for i in 0..get_engine().particle_systems.len() {
        let mut guard = get_engine().particle_systems[i].lock().unwrap();
        if guard.v.is_none() {
            guard.generation += 1;
            guard.v = Some(system);
            return ParticleSystemHandle {
                idx: i as u32,
                generation: guard.generation as u32,
            };
        }
    }
    ParticleSystemHandle::new()
}

pub fn destroy_particle_system(system: ParticleSystemHandle) {
    let mut g = get_engine().particle_systems[system.idx as usize]
        .lock()
        .unwrap();
    if g.generation != system.generation {
        return;
    }
    g.v = None;
}
