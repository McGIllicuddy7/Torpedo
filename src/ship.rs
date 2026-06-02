use std::sync::Arc;

use rand::{random, random_bool};
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    ffi::KeyboardKey::{self},
    math::{Quaternion, Vector3},
};
use serde::{Deserialize, Serialize};

use crate::{
    ai::{AiState, ScannerInfo},
    engine::{
        CameraData, GObject, GameObject, GameObjectData, delete_object, make_object, random_vector,
        set_player,
    },
    graphics::{
        ParticleSystem, SpawnData, SpawnVelocityData, create_particle_system, draw_flame, draw_text,
    },
    mesh::GameMesh,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Ship {
    pub data: GameObjectData,
    pub dead: bool,
    pub is_ai: bool,
    pub quadrants: [[[Vec<ShipComponent>; 3]; 3]; 3],
    pub debug_hit_count: usize,
    pub acc_time: f32,
    pub scanner_info: ScannerInfo,
    pub ai_target: GObject,
    pub ai_state: AiState,
}
impl GameObject for Ship {
    fn on_update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        self.update(handle, thread);
    }
    fn on_event(
        &mut self,
        _handle: &mut raylib::prelude::RaylibHandle,
        _thread: &raylib::prelude::RaylibThread,
        _ev: crate::engine::Event,
    ) {
        match _ev.info {
            crate::engine::EventInfo::OnDamage {
                direction,
                damage_amount,
                penetration,
                aoe,
            } => {
                self.on_damage(damage_amount, penetration, direction, aoe);
            }
            _ => {}
        }
    }
    fn get_data(&self) -> &GameObjectData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut GameObjectData {
        &mut self.data
    }
}

impl Ship {
    pub fn update(
        &mut self,
        handle: &mut raylib::prelude::RaylibHandle,
        thread: &raylib::prelude::RaylibThread,
    ) {
        let dt = handle.get_frame_time();
        self.update_scan(dt);
        let lin_acc_amount = 0.25;
        let rot_acc_amount = 20.;
        let input = if self.dead {
            Input {
                rotational_acc: Vector3::zero(),
                lin_acc: Vector3::zero(),
                wants_to_stop: false,
                fire_missile: false,
                fire_cannon: false,
            }
        } else {
            if self.is_ai {
                self.ai_input(handle, thread)
            } else {
                self.player_input(handle, thread)
            }
        };
        self.handle_movement(&input, dt, rot_acc_amount, lin_acc_amount);
        self.iter_over_all_comps_mut(|_, cmp| {
            cmp.update(dt);
        });
        if input.fire_cannon {
            self.handle_cannon();
        }
        self.handle_health();
        if !self.is_ai {
            self.update_ui();
        }
    }

    pub fn player_input(&self, handle: &mut RaylibHandle, _thread: &RaylibThread) -> Input {
        let mut wants_to_stop = false;
        let mut lin_acc = Vector3::zero();
        let mut racc = Vector3::zero();
        if handle.is_key_down(KeyboardKey::KEY_W) {
            lin_acc.x += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_S) {
            lin_acc.x -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_A) {
            lin_acc.y += 1.
        }
        if handle.is_key_down(KeyboardKey::KEY_D) {
            lin_acc.y -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_Z) {
            lin_acc.z -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_SPACE) {
            lin_acc.z += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            wants_to_stop = true;
            if self.data.velocity.length() > 0.0 {
                lin_acc = -self.data.velocity.normalized();
            } else {
                lin_acc = Vector3::zero();
            }
        }
        if handle.is_key_down(KeyboardKey::KEY_Q) {
            racc.z -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_E) {
            racc.z += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_R) {
            racc.y += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_F) {
            racc.y -= 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_T) {
            racc.x += 1.;
        }
        if handle.is_key_down(KeyboardKey::KEY_G) {
            racc.x -= 1.;
        }
        let v = handle.get_mouse_delta() * 4.;
        racc.z += v.x;
        racc.y -= v.y;
        if racc.length() > 0.0 {
            if racc.z > 1. {
                racc.z = 1.;
            }
            if racc.z < -1. {
                racc.z = -1.;
            }
            if racc.y > 1. {
                racc.y = 1.;
            }
            if racc.y < -1. {
                racc.y = -1.;
            }
        }
        let mut should_fire_cannon = false;
        let mut should_fire_missile = false;
        if handle.is_key_down(KeyboardKey::KEY_X) {
            should_fire_cannon = true;
        }
        if handle.is_key_down(KeyboardKey::KEY_C) {
            should_fire_missile = true;
        }
        Input {
            rotational_acc: racc,
            lin_acc,
            wants_to_stop,
            fire_cannon: should_fire_cannon,
            fire_missile: should_fire_missile,
        }
    }

    pub fn max_thrust(&mut self) -> (Vector3, Vector3) {
        let mut out_pos = Vector3::zero();
        let mut out_neg = Vector3::zero();
        let mut max_fuel_amount = 0;
        for i in 0..self.quadrants.len() {
            for j in 0..self.quadrants[i].len() {
                for l in 0..self.quadrants[i][j].len() {
                    for k in 0..self.quadrants[i][j][l].len() {
                        let tmp = &self.quadrants[i][j][l][k];
                        if tmp.health > 0 {
                            match &tmp.data {
                                ShipComponentData::FuelTank {
                                    remaining_fuel,
                                    max_fuel: _,
                                } => {
                                    if *remaining_fuel > max_fuel_amount {
                                        max_fuel_amount = *remaining_fuel;
                                    }
                                }
                                ShipComponentData::Engine { direction } => {
                                    if direction.x < 0.0 {
                                        out_neg.x += direction.x;
                                    } else {
                                        out_pos.x += direction.x;
                                    }
                                    if direction.y < 0.0 {
                                        out_neg.y += direction.y;
                                    } else {
                                        out_pos.y += direction.y;
                                    }
                                    if direction.z < 0.0 {
                                        out_neg.z += direction.z;
                                    } else {
                                        out_pos.z += direction.z;
                                    }
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
        if max_fuel_amount < 1 {
            (Vector3::zero(), Vector3::zero())
        } else {
            (out_pos, out_neg)
        }
    }

    pub fn consume_fuel_for_manuever(&mut self, len: i32) {
        for _ in 0..len {
            let mut max_amount = 0;
            let mut max_idx = (0, 0, 0, 0);
            for i in 0..self.quadrants.len() {
                for j in 0..self.quadrants[i].len() {
                    for k in 0..self.quadrants[i][j].len() {
                        for l in 0..self.quadrants[i][j][k].len() {
                            let tmp = &self.quadrants[i][j][k][l];
                            if tmp.health > 0 {
                                match &tmp.data {
                                    ShipComponentData::FuelTank {
                                        remaining_fuel,
                                        max_fuel: _,
                                    } => {
                                        if *remaining_fuel > max_amount {
                                            max_amount = *remaining_fuel;
                                            max_idx = (i, j, k, l);
                                        }
                                    }
                                    _ => {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if max_amount == 0 {
                break;
            }
            let tmp = &mut self.quadrants[max_idx.0][max_idx.1][max_idx.2][max_idx.3];
            match &mut tmp.data {
                ShipComponentData::FuelTank {
                    remaining_fuel,
                    max_fuel: _,
                } => {
                    *remaining_fuel -= 1;
                }
                _ => {}
            }
        }
    }

    pub fn on_damage(&mut self, damage: i32, penetration: i32, direction: Vector3, aoe: bool) {
        self.debug_hit_count += 1;
        let dir = -direction.rotate_by(self.data.rotation);
        let mut hit_set = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let delt = Vector3::new(-dx as f32, -dy as f32, -dz as f32);
                    hit_set.push(((dx, dy, dz), delt));
                }
            }
        }
        let mut min_idx = hit_set[0].0;
        let mut min = dir.dot(hit_set[0].1);
        for (p, d) in &hit_set {
            if d.dot(dir) < min {
                min = d.dot(dir);
                min_idx = *p;
            }
        }
        let mut damage = damage;
        let mut hit = false;
        for _ in 0..3 {
            if min_idx.0 < -1
                || min_idx.1 < -1
                || min_idx.2 < -1
                || min_idx.0 > 1
                || min_idx.1 > 1
                || min_idx.2 > 1
            {
                break;
            }
            let mut penetrated = false;
            if aoe {
                let mut min_health = 100000;
                for i in self.get_comps_mut(min_idx.0, min_idx.1, min_idx.2) {
                    if i.health < min_health {
                        min_health = i.health;
                    }
                    if i.health < 0 {
                        penetrated = true;
                        continue;
                    }
                    i.on_damage(damage);
                    hit = true;
                    if i.health < damage * penetration {
                        penetrated = true;
                    }
                }
                damage -= min_health / penetration;
            } else {
                let mut total = 0;
                for i in self.get_comps(min_idx.0, min_idx.1, min_idx.2) {
                    total += i.volume;
                }
                if total == 0 {
                    break;
                }
                let r = random::<u64>() % total;
                let mut base = 0;
                for i in self.get_comps_mut(min_idx.0, min_idx.1, min_idx.2) {
                    base += i.volume;
                    if base >= r {
                        //   println!("bhit:{:#?}: at:{:#?}", i, min_idx);
                        let oh = i.health;
                        if oh < 0 {
                            penetrated = true;
                            break;
                        }
                        i.on_damage(damage);
                        hit = true;
                        if i.health < damage.saturating_mul(penetration) {
                            damage -= oh / penetration;
                            penetrated = true;
                        }
                        break;
                    }
                }
            }
            if !penetrated || damage < 0 {
                break;
            }
            //println!("penetrated");
            //  let old = min_idx;
            min_idx.0 += (random::<u64>() % 3) as i32 - 1;
            min_idx.1 += (random::<u64>() % 3) as i32 - 1;
            min_idx.2 += (random::<u64>() % 3) as i32 - 1;
            //println!("traversed:{:#?} to :{:#?}", old, min_idx);
        }
        if !hit {
            self.iter_over_all_comps_mut(|_, i| {
                if i.health > 0 {
                    i.on_damage(damage / 10);
                }
            });
        }
    }

    pub fn get_comps(&self, x: i32, y: i32, z: i32) -> &Vec<ShipComponent> {
        &self.quadrants[(x + 1) as usize][(y + 1) as usize][(z + 1) as usize]
    }

    pub fn get_comps_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Vec<ShipComponent> {
        &mut self.quadrants[(x + 1) as usize][(y + 1) as usize][(z + 1) as usize]
    }

    pub fn update_ui(&self) {
        let mut fuel_cap = 0;
        let mut fuel_amount = 0;
        let mut missile_amount = 0;
        let mut missile_cap = 0;
        let mut bullet_amount = 0;
        let mut bullet_cap = 0;
        self.iter_over_all_comps(|_, comp| match &comp.data {
            ShipComponentData::FuelTank {
                remaining_fuel,
                max_fuel,
            } => {
                if comp.health > 0 {
                    fuel_amount += *remaining_fuel;
                }
                fuel_cap += *max_fuel;
            }
            ShipComponentData::Magazine {
                max_missile_count,
                max_bullet_count,
                missile_count,
                bullet_count,
            } => {
                missile_cap += *max_missile_count;
                bullet_cap += *max_bullet_count;
                missile_amount += *missile_count;
                bullet_amount += *bullet_count;
            }
            _ => {}
        });
        let text = format!("{} seconds of thrust remaining", fuel_amount);
        let text2 = format!(
            "{} seconds specific impulse",
            (fuel_cap as f32 / 10.) as i32
        );
        let v = self.data.velocity.rotate_by(self.data.rotation);
        let t3 = format!(
            "relative velocity(m/s): x:{}, y:{}, z:{}",
            (v.x * 100.).ceil() as i32,
            (v.y * 100.).ceil() as i32,
            (v.z * 100.).ceil() as i32
        );
        draw_text(&text, 100, 500, 16, Color::GREEN);
        draw_text(&text2, 100, 520, 16, Color::GREEN);
        draw_text(&t3, 100, 540, 16, Color::GREEN);
        let t4 = format!("missiles:{}/{}", missile_amount, missile_cap);
        let t5 = format!("bullets:{}/{}", bullet_amount, bullet_cap);
        draw_text(&t5, 100, 560, 16, Color::GREEN);
        draw_text(&t4, 100, 580, 16, Color::GREEN);
        if self.dead {
            draw_text("DEAD", 500, 500, 32, Color::WHITE);
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                let mut max = 0;
                let mut current = 0;
                for dz in -1..=1 {
                    for i in self.get_comps(dx, dy, dz) {
                        max += i.max_health;
                        current += i.health;
                    }
                }
                let ps = (current as f32 * 100. / max as f32) as i32;
                let x = (dy * 50) + 50;
                let y = ((1 - dx) * 40) + 600;
                let text = format!("[{}%]", ps);
                draw_text(&text, x, y, 16, Color::WHITE);
            }
        }
    }

    pub fn handle_health(&mut self) {
        let mut should_explode = false;
        let mut health_base = 0;
        let mut health_current = 0;
        let mut should_die = false;
        self.iter_over_all_comps(|_, i| {
            health_base += i.max_health;
            health_current += i.health;
            if i.health < 0 && i.on_fire {
                should_explode = true;
            }
            if i.health < 0 && i.is_brain {
                should_die = true;
            }
        });
        if health_base >= health_current * 4 {
            should_explode = true;
        }
        if should_die {
            self.dead = true;
        }
        if should_explode {
            println!("hit {} times before destroyed", self.debug_hit_count);
            self.dead = true;
            delete_object(self.data.self_id, self.data.self_id);
            spawn_explosion(self.data.location, 10.);
        }
    }

    pub fn handle_cannon(&mut self) {
        let mut can_fire = false;
        let mut to_fire_idx = (0, 0, 0, 0);
        let mut max_ammo_count = 0;
        let mut max_ammo_idx = (0, 0, 0, 0);
        self.iter_over_all_comps(|idx, v| {
            if v.health > 0 {
                match &v.data {
                    ShipComponentData::Magazine {
                        max_missile_count: _,
                        max_bullet_count: _,
                        missile_count: _,
                        bullet_count,
                    } => {
                        if *bullet_count > max_ammo_count {
                            max_ammo_count = *bullet_count;
                            max_ammo_idx = idx;
                        }
                    }
                    ShipComponentData::Turret {
                        cool_down_time: _,
                        remaining_cool_down_time,
                    } => {
                        if can_fire {
                            return;
                        }
                        if *remaining_cool_down_time <= 0.0 {
                            to_fire_idx = idx;
                            can_fire = true;
                        }
                    }
                    _ => {}
                }
            }
        });
        if can_fire && max_ammo_count >= 2 {
            let tmp = &mut self.quadrants[(to_fire_idx.0 + 1) as usize]
                [(to_fire_idx.1 + 1) as usize][(to_fire_idx.2 + 1) as usize][to_fire_idx.3];
            match &mut tmp.data {
                ShipComponentData::Turret {
                    cool_down_time,
                    remaining_cool_down_time,
                } => {
                    *remaining_cool_down_time = *cool_down_time;
                }
                _ => {}
            }
            let tmp = &mut self.quadrants[(max_ammo_idx.0 + 1) as usize]
                [(max_ammo_idx.1 + 1) as usize][(max_ammo_idx.2 + 1) as usize][max_ammo_idx.3];
            match &mut tmp.data {
                ShipComponentData::Magazine {
                    max_missile_count: _,
                    max_bullet_count: _,
                    missile_count: _,
                    bullet_count,
                } => {
                    *bullet_count -= 2;
                }
                _ => {}
            }
            let dir = Vector3::new(1.0, 0.0, 0.0).rotate_by(self.data.rotation.inverted());
            let dir_perp =
                Vector3::new(0.0, 1.0, 0.0).rotate_by(self.data.rotation.inverted()) * 0.5;
            let base = self.data.location + dir * 1.5;
            fire_bullet(base + dir_perp, dir, self.data.velocity, self.data.rotation);
            fire_bullet(base - dir_perp, dir, self.data.velocity, self.data.rotation);
        }
    }

    pub fn handle_movement(
        &mut self,
        input: &Input,
        dt: f32,
        rot_acc_amount: f32,
        lin_acc_amount: f32,
    ) {
        self.data.angular_velocity += input.rotational_acc * rot_acc_amount * 1. / 60.;
        if input.rotational_acc.length() == 0.0 {
            if self.data.angular_velocity.length() < rot_acc_amount / 60. {
                self.data.angular_velocity = Vector3::zero();
            } else {
                self.data.angular_velocity -=
                    self.data.angular_velocity.normalized() * rot_acc_amount / 60.;
            }
        }
        let input_dir = {
            let mut tmp = input.lin_acc;
            let (acc, nacc) = self.max_thrust();
            if tmp.x > 0. {
                tmp.x *= acc.x;
            } else {
                tmp.x *= nacc.x.abs();
            }
            if tmp.y > 0. {
                tmp.y *= acc.y;
            } else {
                tmp.y *= nacc.y.abs();
            }
            if tmp.z > 0. {
                tmp.z *= acc.z;
            } else {
                tmp.z *= nacc.z.abs();
            }
            tmp
        };
        if input.wants_to_stop && self.data.velocity.length() < 1. {
            self.data.velocity = Vector3::zero();
        } else {
            self.data.velocity +=
                input_dir.rotate_by(self.data.rotation.inverted()) * lin_acc_amount * 1. / 60.;
        }
        self.acc_time += dt * input_dir.length();
        if self.acc_time > 1. {
            self.consume_fuel_for_manuever(1);
            self.acc_time = 0.0;
        }
        if self.data.angular_velocity.length() > 1. {
            let n = self.data.angular_velocity.normalized();
            self.data.angular_velocity = n * 1.;
        }
        if self.data.velocity.length() > 1000. {
            let n = self.data.velocity.normalized();
            self.data.velocity = n * 1000.;
        }
        if !self.dead {
            let d1 = Vector3::new(-1., 0., 0.).rotate_by(self.data.rotation.inverted());
            let d2 = Vector3::new(0., 1., 0.).rotate_by(self.data.rotation.inverted());
            let l = input_dir.length() * 1.25 + 0.025;
            draw_flame(
                d1 * 0.5 + d2 * 0.5 + self.data.location,
                d1,
                0.25,
                l,
                Color::VIOLET,
            );
            draw_flame(
                d1 * 0.5 - d2 * 0.5 + self.data.location,
                d1,
                0.25,
                l,
                Color::VIOLET,
            );
        }
    }
}

pub struct Input {
    pub rotational_acc: Vector3,
    pub lin_acc: Vector3,
    pub wants_to_stop: bool,
    pub fire_missile: bool,
    pub fire_cannon: bool,
}

pub fn create_player_ufo(pos: Vector3, rotation: Quaternion) -> GObject {
    let size = 1.;
    _ = rotation;
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cylinder(
        1. * size,
        0.8 * size,
        0.5 * size,
        6,
        Vector3::new(0., 0., -0.25 * size),
        Vector3::new(0.0, 0.0, 1.),
        Vector3::new(1., 0.0, 0.0),
    );
    let mut s = Ship {
        debug_hit_count: 0,
        dead: false,
        acc_time: 0.0,
        is_ai: false,
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: rotation,
            width: 2. * size,
            depth: 2. * size,
            height: 0.5 * size,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: Some(CameraData {
                position: Vector3::new(0.5, 0.15, 0.05),
                rotation: Quaternion::identity(),
            }),
            is_projectile: false,
            is_static: false,
            tags: Arc::new(["ship".into()]),
            self_id: GObject::new(),
            projectile_damage: 0,
            projectile_penetration: 0,
            allegience: 0,
        },
        quadrants: [const { [const { [const { Vec::new() }; _] }; _] }; _],
        scanner_info: ScannerInfo::new(),
        ai_target: GObject::new(),
        ai_state: AiState::new(),
    };
    s.default_layout();
    let v = make_object(s);
    set_player(v);
    v
}

pub fn create_ai_ufo(pos: Vector3, rotation: Quaternion) -> GObject {
    let size = 1.;
    _ = rotation;
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cylinder(
        1. * size,
        0.8 * size,
        0.5 * size,
        6,
        Vector3::new(0., 0., -0.25 * size),
        Vector3::new(0.0, 0.0, 1.),
        Vector3::new(1., 0.0, 0.0),
    );
    let mut s = Ship {
        debug_hit_count: 0,
        dead: false,
        acc_time: 0.0,
        is_ai: true,
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: rotation,
            width: 2. * size,
            depth: 2. * size,
            height: 0.5 * size,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: None,
            is_projectile: false,
            is_static: false,
            tags: Arc::new(["ship".into()]),
            self_id: GObject::new(),
            projectile_damage: 0,
            projectile_penetration: 0,
            allegience: 1,
        },
        quadrants: [const { [const { [const { Vec::new() }; _] }; _] }; _],
        scanner_info: ScannerInfo::new(),
        ai_target: GObject::new(),
        ai_state: AiState::new(),
    };
    s.default_layout();
    let v = make_object(s);
    v
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ShipComponent {
    pub is_brain: bool,
    pub integral: bool,
    pub health: i32,
    pub max_health: i32,
    pub volume: u64,
    pub on_fire: bool,
    pub data: ShipComponentData,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ShipComponentData {
    Cockpit {},
    FuelTank {
        remaining_fuel: u64,
        max_fuel: u64,
    },
    Antenna {},
    Magazine {
        max_missile_count: u32,
        max_bullet_count: u32,
        missile_count: u32,
        bullet_count: u32,
    },
    CargoHold {},
    Engine {
        direction: Vector3,
    },
    Armor {},
    Turret {
        cool_down_time: f32,
        remaining_cool_down_time: f32,
    },
    MissileBattery {
        loaded: bool,
        loading_time: f32,
        remaining_loading_time: f32,
    },
}

impl Ship {
    pub fn add_component(&mut self, to: (i32, i32, i32), comp: ShipComponent) {
        let to = (
            (to.0 + 1) as usize,
            (to.1 + 1) as usize,
            (to.2 + 1) as usize,
        );
        self.quadrants[to.0][to.1][to.2].push(comp);
    }

    pub fn add_engine(&mut self, to: (i32, i32, i32), thrust: Vector3, hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Engine { direction: thrust },
            },
        );
    }

    pub fn add_fuel_comp(&mut self, to: (i32, i32, i32), capacity: u64, hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::FuelTank {
                    remaining_fuel: capacity,
                    max_fuel: capacity,
                },
            },
        );
    }

    pub fn add_cockpit(&mut self, to: (i32, i32, i32), hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: true,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Cockpit {},
            },
        );
    }

    pub fn add_cargo_hold(&mut self, to: (i32, i32, i32), hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::CargoHold {},
            },
        );
    }

    pub fn add_armor(&mut self, to: (i32, i32, i32), hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Armor {},
            },
        );
    }

    pub fn add_antenna(&mut self, to: (i32, i32, i32), hp: i32, vol: u64) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Antenna {},
            },
        );
    }

    pub fn add_magazine(
        &mut self,
        to: (i32, i32, i32),
        hp: i32,
        vol: u64,
        missile_count: u32,
        max_missile_count: u32,
        bullet_count: u32,
        max_bullet_count: u32,
    ) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Magazine {
                    max_missile_count,
                    max_bullet_count,
                    missile_count,
                    bullet_count,
                },
            },
        );
    }

    pub fn add_turret(&mut self, to: (i32, i32, i32), hp: i32, vol: u64, cool_down_time: f32) {
        self.add_component(
            to,
            ShipComponent {
                is_brain: false,
                integral: false,
                health: hp,
                max_health: hp,
                volume: vol,
                on_fire: false,
                data: ShipComponentData::Turret {
                    cool_down_time,
                    remaining_cool_down_time: 0.09,
                },
            },
        );
    }

    pub fn default_layout(&mut self) {
        for i in -1..=1 {
            for j in -1..=1 {
                for k in -1..=1 {
                    if i != 0 || j != 0 || k != 0 {
                        self.add_armor((i, j, k), 100, 100);
                    }
                }
            }
        }
        self.add_cockpit((1, 0, 0), 10, 20);
        self.add_turret((1, 0, 0), 10, 10, 0.02);
        self.add_magazine((0, 0, 0), 10, 100, 10, 16, 10000, 16000);
        self.add_fuel_comp((0, 0, 0), 1000, 10, 100);
        self.add_engine((-1, 0, 0), Vector3::new(1., 0., 0.), 20, 10);
        self.add_fuel_comp((-1, 0, 0), 1000, 10, 100);
        self.add_engine((1, 0, 0), Vector3::new(-1., 0., 0.), 20, 10);
        self.add_engine((-1, 0, 0), Vector3::new(1., 0., 0.), 20, 10);
        self.add_engine((0, 1, 0), Vector3::new(0., -1., 0.), 20, 10);
        self.add_engine((0, -1, 0), Vector3::new(0., 1., 0.), 20, 10);
        self.add_engine((0, 0, 1), Vector3::new(0., 0., -1.), 20, 10);
        self.add_engine((0, 0, -1), Vector3::new(0., 0., 1.), 20, 10);
    }

    pub fn iter_over_all_comps_mut(
        &mut self,
        mut to_run: impl FnMut((i32, i32, i32, usize), &mut ShipComponent),
    ) {
        for i in 0..self.quadrants.len() {
            for j in 0..self.quadrants[i].len() {
                for k in 0..self.quadrants[i][j].len() {
                    for l in 0..self.quadrants[i][j][k].len() {
                        to_run(
                            (i as i32 - 1, j as i32 - 1, k as i32 - 1, l),
                            &mut self.quadrants[i][j][k][l],
                        );
                    }
                }
            }
        }
    }

    pub fn iter_over_all_comps(
        &self,
        mut to_run: impl FnMut((i32, i32, i32, usize), &ShipComponent),
    ) {
        for i in 0..self.quadrants.len() {
            for j in 0..self.quadrants[i].len() {
                for k in 0..self.quadrants[i][j].len() {
                    for l in 0..self.quadrants[i][j][k].len() {
                        to_run(
                            (i as i32 - 1, j as i32 - 1, k as i32 - 1, l),
                            &self.quadrants[i][j][k][l],
                        );
                    }
                }
            }
        }
    }
}

impl ShipComponent {
    pub fn update(&mut self, dt: f32) {
        if self.health <= 0 {
            return;
        }
        if self.on_fire {
            if random_bool((dt as f64).clamp(0.0, 1.0)) {
                self.health -= 1;
            }
            if random_bool((dt as f64).clamp(0.0, 1.0) / 2.) {
                self.on_fire = false;
            }
        }
        match &mut self.data {
            ShipComponentData::Cockpit {} => {}
            ShipComponentData::FuelTank {
                remaining_fuel: _,
                max_fuel: _,
            } => {}
            ShipComponentData::Antenna {} => {}
            ShipComponentData::Magazine {
                max_missile_count: _,
                max_bullet_count: _,
                missile_count: _,
                bullet_count: _,
            } => {}
            ShipComponentData::CargoHold {} => {}
            ShipComponentData::Engine { direction: _ } => {}
            ShipComponentData::Armor {} => {}
            ShipComponentData::Turret {
                cool_down_time: _,
                remaining_cool_down_time,
            } => {
                *remaining_cool_down_time -= dt;
                if *remaining_cool_down_time < 0.0 {
                    *remaining_cool_down_time = 0.0;
                }
            }
            ShipComponentData::MissileBattery {
                loaded,
                loading_time: _,
                remaining_loading_time,
            } => {
                if *remaining_loading_time > 0.0 {
                    *remaining_loading_time -= dt;
                    if *remaining_loading_time <= 0.0 {
                        *loaded = true;
                        *remaining_loading_time = 0.0;
                    }
                }
            }
        }
    }

    pub fn on_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health < 0 {
            println!("destroyed {:#?}", self);
        }
        match self.data {
            ShipComponentData::Magazine {
                max_missile_count: _,
                max_bullet_count: _,
                missile_count: _,
                bullet_count: _,
            } => {
                if amount > 1 {
                    self.on_fire = random_bool(0.5);
                }
            }
            ShipComponentData::FuelTank {
                remaining_fuel: _,
                max_fuel: _,
            } => {
                if amount > 5 {
                    self.on_fire = random_bool(0.2);
                }
            }
            _ => {}
        }
    }
}

pub struct Bullet {
    data: GameObjectData,
    remaining_time: f32,
}
impl GameObject for Bullet {
    fn get_data(&self) -> &GameObjectData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut GameObjectData {
        &mut self.data
    }
    fn on_event(
        &mut self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        ev: crate::engine::Event,
    ) {
        //delete_object(self.data.self_id, self.data.self_id);
    }

    fn on_update(&mut self, handle: &mut RaylibHandle, _thread: &RaylibThread) {
        self.remaining_time -= handle.get_frame_time();
        if self.remaining_time < 0.0 {
            delete_object(self.data.self_id, self.data.self_id);
        }
    }
}

pub fn fire_bullet(
    start: Vector3,
    direction: Vector3,
    base_velocity: Vector3,
    rotation: Quaternion,
) -> GObject {
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cylinder(
        0.01,
        0.01,
        0.1,
        3,
        Vector3::new(0., 0., -0.05),
        Vector3::new(1.0, 0.0, 0.),
        Vector3::new(0., 0.0, 1.0),
    );
    let s = Bullet {
        data: GameObjectData {
            model: Some(msh),
            location: start,
            rotation,
            width: 0.01,
            depth: 0.1,
            height: 0.01,
            velocity: direction * 35. + base_velocity,
            angular_velocity: Vector3::zero(),
            camera_data: None,
            is_projectile: true,
            is_static: false,
            tags: Arc::new(["bullet".into()]),
            self_id: GObject::new(),
            projectile_damage: 10,
            projectile_penetration: 2,
            allegience: -1,
        },
        remaining_time: 30.,
    };
    let out = make_object(s);
    out
}

pub fn spawn_explosion(at: Vector3, radius: f32) {
    create_particle_system(ParticleSystem::new(
        at,
        Vector3::zero(),
        true,
        SpawnData {
            amount_to_spawn: 100,
            probability_to_spawn_per_second: 1.,
            spawn_radius: 10.,
            velocity_info: SpawnVelocityData::Sphere { radius },
            can_stop_spawning: true,
            max_connection_count: 0,
            max_connection_distance: 0.0,
            min_lifetime: 0.1,
            max_lifetime: 1.,
            min_radius: 0.1,
            max_radius: 1.,
            min_end_radius: 1.,
            max_end_radius: 4.,
            spawning_duration: 0.2,
            ending_color: Color::GRAY,
            glowing: false,
            color: Color::ORANGERED,
        },
    ));
}
