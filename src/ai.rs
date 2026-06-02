use std::{collections::HashMap, f32::consts::PI};

use rand::{random, random_bool};
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    math::{Quaternion, Vector3},
};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{GObject, get_all_objects_with_tag, random_vector, raycast},
    graphics::draw_point,
    ship::{Input, Ship},
};

impl Ship {
    pub fn ai_input(&mut self, _handle: &mut RaylibHandle, _thread: &RaylibThread) -> Input {
        let mut should_target = false;
        let mut target_pos = Vector3::zero();
        for (_idx, i) in &self.scanner_info.objects {
            if i.is_visible && i.is_enemy {
                should_target = true;
                let distance = i.predicated_location.distance_to(self.data.location);
                if distance > self.scanner_info.max_range {
                    continue;
                }
                let npos = i.predicated_location
                    + i.predicated_velocity * (distance / (35.))
                    + random_vector() * distance * 0.05;
                target_pos = npos;
                //  draw_point(self.data.location, Color::RED);
                break;
            }
        }
        should_target = false;
        if self
            .data
            .location
            .distance_to(self.ai_state.move_to_location)
            < 5.
            || self.ai_state.move_to_location.length() < 0.1
        {
            self.ai_state.move_to_location = random_vector() * 40.;
        }
        let acc = calculate_acceleration(
            self.data.location,
            self.data.velocity,
            self.ai_state.move_to_location,
            Vector3::zero(),
            0.25,
        )
        .rotate_by(self.data.rotation)
            * 4.;
        // println!("acc:{:#?}", acc);
        draw_point(self.ai_state.move_to_location, Color::RED);
        let mut should_fire = false;
        let racc = if should_target {
            let dot = (target_pos - self.data.location)
                .normalized()
                .dot(Vector3::new(1., 0., 0.0).rotate_by(self.data.rotation.inverted()));
            if dot > 0.85 {
                should_fire = true && random_bool(0.25);
            }
            let tp = self.delta_rotate_to_look_at_towards(target_pos) * 120.0;
            /*   println!(
                "target {:#?}, target_pos:{:#?}, dot:{:#?}",
                tp, target_pos, dot
            );*/
            tp
        } else {
            let tp = self.delta_rotate_to_look_at_towards(
                self.data.location + acc.rotate_by(self.data.rotation.inverted()),
            ) * 120.0;
            tp
        };
        Input {
            wants_to_stop: false,
            rotational_acc: racc,
            lin_acc: acc,
            fire_cannon: should_fire,
            fire_missile: random_bool((0.1 * _handle.get_frame_time()) as f64),
        }
    }

    pub fn delta_rotate_to_look_at_towards(&self, point: Vector3) -> Vector3 {
        let self_rot = self.data.rotation.inverted();
        let self_pos = self.data.location;
        let out = delta_rotation_to_look_at(self_pos, self_rot, point);
        out
    }

    pub fn update_scan(&mut self, mut dt: f32) {
        if dt < 0.01 {
            dt = 0.01;
        }
        let ships = get_all_objects_with_tag("ship");
        let missiles = get_all_objects_with_tag("missile");
        let objects = {
            let mut tmp = Vec::new();
            tmp.reserve_exact(ships.len() + missiles.len());
            for i in ships {
                tmp.push(i);
            }
            for j in missiles {
                tmp.push(j);
            }
            tmp
        };
        for i in objects {
            let base = i.get();
            let data = base.get_data();
            if data.location.distance_to(self.data.location) < self.scanner_info.max_range {
                let team = data.allegience;
                if team < 0 {
                    continue;
                }
                let is_enemy = team != self.data.allegience;
                let velocity = data.velocity;
                let location = data.location;
                let facing = Vector3::new(1., 0., 0.).rotate_by(data.rotation);
                let visible = raycast(
                    self.data.location,
                    (location - self.data.location).normalized(),
                    self.scanner_info.max_range,
                    &[self.data.self_id, i],
                    false,
                )
                .is_none();
                if let Some(rf) = self.scanner_info.objects.get_mut(&i) {
                    if visible {
                        let v0 = rf.predicated_velocity;
                        let info = ObjectInformation {
                            acceleration: (data.velocity - v0) / (dt),
                            ptr: i,
                            predicated_location: location,
                            predicated_velocity: velocity,
                            forward_vector: facing,
                            is_enemy,
                            is_ship: team >= 0,
                            is_visible: visible,
                            duration_since_last_seen: 0.0,
                        };
                        *rf = info;
                    } else {
                        rf.duration_since_last_seen += dt;
                        if rf.duration_since_last_seen > 5. {
                            self.scanner_info.objects.remove(&i);
                            continue;
                        }
                        rf.predicated_location += rf.predicated_velocity * dt;
                        rf.predicated_velocity += rf.acceleration * dt;
                        rf.is_visible = visible;
                    }
                } else {
                    if !visible {
                        continue;
                    }
                    let info = ObjectInformation {
                        acceleration: Vector3::zero(),
                        ptr: i,
                        predicated_location: location,
                        predicated_velocity: velocity,
                        forward_vector: facing,
                        is_enemy,
                        is_ship: team >= 0,
                        is_visible: visible,
                        duration_since_last_seen: 0.0,
                    };
                    self.scanner_info.objects.insert(i, info);
                }
            }
        }
        let list: Vec<_> = self
            .scanner_info
            .objects
            .iter()
            .map(|i| i.0.clone())
            .collect();
        for i in list {
            if !i.is_valid() {
                self.scanner_info.objects.remove(&i);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ObjectInformation {
    pub ptr: GObject,
    pub predicated_location: Vector3,
    pub predicated_velocity: Vector3,
    pub acceleration: Vector3,
    pub is_enemy: bool,
    pub is_ship: bool,
    pub forward_vector: Vector3,
    pub is_visible: bool,
    pub duration_since_last_seen: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannerInfo {
    pub objects: HashMap<GObject, ObjectInformation>,
    pub max_range: f32,
}
impl ScannerInfo {
    pub fn new() -> Self {
        ScannerInfo {
            objects: HashMap::new(),
            max_range: 100.,
        }
    }
}

pub fn delta_rotation_to_look_at(
    self_location: Vector3,
    self_rotation: Quaternion,
    target_location: Vector3,
) -> Vector3 {
    let mut out = Vector3::new(0.0, 0.0, 0.0);
    /*   let base_forward = Vector3::new(1.0, 0.0, 0.0)
    .rotate_by(self_rotation)
    .normalized();*/
    let delta = -(target_location - self_location).normalized();
    let mut dot = -1000.;
    let dv = 100.;
    for dx in -10..=10 {
        for dy in -10..=10 {
            for dz in -10..=10 {
                let nq = self_rotation
                    * Quaternion::from_euler(dx as f32 / dv, dy as f32 / dv, dz as f32 / dv);
                let d1 = Vector3::new(1.0, 0.0, 0.0).rotate_by(nq);
                let d2 = d1.dot(delta);
                if d2 > dot {
                    dot = d2;
                    out = Vector3::new(dx as f32 / dv, dy as f32 / dv, dz as f32 / dv);
                }
            }
        }
    }
    if out.length() < 0.001 {
        let delta = (target_location - self_location).normalized();
        let mut dot = -1000.;
        let dv = 500.;
        for dx in -10..=10 {
            for dy in -10..=10 {
                for dz in -10..=10 {
                    let nq = self_rotation
                        * Quaternion::from_euler(dx as f32 / dv, dy as f32 / dv, dz as f32 / dv);
                    let d1 = Vector3::new(1.0, 0.0, 0.0).rotate_by(nq);
                    let d2 = d1.dot(delta);
                    if d2 > dot {
                        dot = d2;
                        out = Vector3::new(dx as f32 / dv, dy as f32 / dv, dz as f32 / dv);
                    }
                }
            }
        }
    }
    out
}

pub fn random_rotation() -> Quaternion {
    let x = (((random::<u64>() % 10000) as f32) / 10000.) * 2. * PI;
    let y = (((random::<u64>() % 10000) as f32) / 10000.) * 2. * PI;
    let z = (((random::<u64>() % 10000) as f32) / 10000.) * 2. * PI;
    Quaternion::from_euler(x, y, z)
}

pub fn delta_rotation_to_look_at_test() {
    let sp = random_vector() * 10.;
    let op = random_vector() * 10.;
    let mut sr = random_rotation();
    let delta = (op - sp).normalized();
    let mut sf = Vector3::new(1., 0., 0.).rotate_by(sr);
    let mut i = 0;
    while delta.dot(sf) < 0.99 {
        let tmp = delta_rotation_to_look_at(sp, sr, op);
        let qt = Quaternion::from_euler(tmp.x, tmp.y, tmp.z);
        sr = qt * sr;
        sf = Vector3::new(1., 0., 0.).rotate_by(sr);
        let dot = delta.dot(sf);
        println!(
            "i:{}, sp:{:#?},op:{:#?}, sf:{:#?}, delta:{:#?}, dot:{:#?}",
            i, sp, op, sf, delta, dot
        );
        i += 1;
    }
    let dot = delta.dot(sf);
    println!(
        "i:{}, sp:{:#?},op:{:#?}, sf:{:#?}, delta:{:#?}, dot:{:#?}",
        i, sp, op, sf, delta, dot
    );
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiState {
    pub state: EAiState,
    pub move_to_location: Vector3,
    pub base_location: Vector3,
    pub acceleration_phase: AccelerationPhase,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EAiState {
    Travel,
    Patrol,
    Guard,
    Search,
    Rush,
    Skirmish,
    Fleeing,
}

impl AiState {
    pub fn new() -> Self {
        Self {
            state: EAiState::Guard,
            move_to_location: Vector3::zero(),
            base_location: Vector3::zero(),
            acceleration_phase: AccelerationPhase::Drifting,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccelerationPhase {
    SpeedingUp,
    Drifting,
    SlowingDown,
}
pub fn calculate_acceleration(
    point: Vector3,
    current_velocity: Vector3,
    target_location: Vector3,
    target_velocity: Vector3,
    max_acceleration: f32,
) -> Vector3 {
    /*
    minimize t such that
    f(0) = p_0, f(t) = p_1,
     f'(0) = v_0, f'(t) = v_1,
     |f''(t_i)|<max_acc \forall t_i
     f'(t_i)-v_0< max_acc*t_i \forall t_i
     f(t_i)-v_0(t_i) - p_0< 1/2max_acc t_i^2\forall t_i
     p_1 -v_0(t) - p_0 < 1/2 max_acc *t^2
     v_1-v_0< max_acc*t
    t>= (v_1-v_0)/max_acc;
    t = (v_1 - v_0)/max_acc;
    p_1 -v_0(t) - p_0 < 1/2 max_acc *t^2
    1/2 max_acc t^2 +v_0*t >= p_1 -p_0
    1/2 max_acc t^2 + v_0 t -(p_1-p_0) = 0
    t = (-v_0 \pm \sqrt{v_0 *v_0 +4(1/2 max_acc)(p_1-p_0)}){(max_acc)}
     */
    let dt = 30.;
    let mut out;
    let cost = |pos: Vector3, vel: Vector3| {
        let t0 = (vel - target_velocity).length() * 1.1;
        let t1 = (-(vel).length()
            + (vel.length() * vel.length()
                + 2. * max_acceleration * (pos - target_location).length())
            .sqrt())
            / (max_acceleration);
        let t = if t1 > t0 { t1 } else { t0 };
        t
    };
    //let base_cost = cost(point, current_velocity);
    let acc_1 = if (current_velocity - target_velocity).length() > 0.0 {
        (target_velocity - current_velocity).normalized() * max_acceleration
    } else {
        Vector3::zero()
    };
    let cost_1 = cost(
        point + (current_velocity + (acc_1 * 0.1) / dt) / dt,
        current_velocity + (acc_1 * 0.1) / dt,
    );
    let acc_2 = if (point - target_location).length() > 0.0 {
        (target_location - point).normalized() * max_acceleration
    } else {
        Vector3::zero()
    };
    let cost_2 = cost(
        point + (current_velocity + acc_2 / dt) / dt,
        current_velocity + acc_2 / dt,
    );
    let acc_3 = acc_1 * 0.5 + acc_2 * 0.5;
    let cost_3 = cost(
        point + (current_velocity + acc_3 / dt) / dt,
        current_velocity + acc_3 / dt,
    );

    let costs = [(cost_1, acc_1), (cost_2, acc_2), (cost_3, acc_3)];
    // println!("costs:{:#?}", costs);
    let mut min_cost = cost_1;
    out = acc_1;
    for i in costs {
        if i.0 < min_cost {
            out = i.1;
            min_cost = i.0;
        }
    }
    out
}
