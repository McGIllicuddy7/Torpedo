pub use anyhow::Result;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::math::Quaternion;
pub use raylib::math::{BoundingBox, RayCollision, Vector3};
use raylib::prelude::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt};
use raylib::shaders::Shader;
use raylib::{RaylibHandle, RaylibThread};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
pub use std::sync::Arc;
pub use std::sync::{Mutex, RwLock};
use std::sync::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};
pub const MAX_ENTITY_COUNT: usize = 65536;
use crate::system::physics::{ColData3D, Collider3D};
use crate::{GameMode, make_system_vtable, make_vtable};
lazy_static::lazy_static!(
    pub static ref VTABLE:HashMap<EntityKind, EntityVTableEntry> = make_vtable();
);
lazy_static::lazy_static!(
    pub static ref SYSTEM_VTABLE:HashMap<GameMode, GameEngineVTableEntry> = make_system_vtable();
);
pub use crate::{EntityComponentKind, EntityKind};
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum RenderKind {
    #[default]
    Cube,
    Cylinder,
    Sphere,
    Cone,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum DamageType {
    #[default]
    Piercing,
    Crushing,
    Radiation,
    Laser,
    Incendiary,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Default, Hash, PartialEq, Eq)]
pub struct Entity {
    index: u32,
    generation: u32,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityStruct {
    pub self_id: Entity,
    pub name: Option<Arc<str>>,
    pub kind: EntityKind,
    pub is_player: bool,
    pub is_projectile: bool,
    pub is_static: bool,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub angular_velocity: Vector3,
    pub velocity: Vector3,
    pub cached_bounds: BoundingBox,
    pub component_table: HashMap<Arc<str>, EntityComponent>,
    pub data_component_table: HashMap<Arc<str>, EntityDataComponent>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityComponent {
    pub parent_id: Entity,
    pub kind: EntityComponentKind,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub offset: Vector3,
    pub health: u32,
    pub u32_data: [u32; 8],
    pub i32_data: [i32; 8],
    pub f32_data: [f32; 8],
    pub entity_data: [Entity; 4],
    pub string_data: [Option<Arc<str>>; 4],
    pub children: Vec<Arc<str>>,
    pub vector_data: [Vector3; 4],
    pub render_as: RenderKind,
    pub color: Color,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntityDataComponent {
    pub parent_id: Entity,
    pub kind: EntityComponentKind,
    pub u32_data: [u32; 8],
    pub i32_data: [i32; 8],
    pub f32_data: [f32; 8],
    pub entity_data: [Entity; 4],
    pub string_data: [Option<Arc<str>>; 4],
    pub vector_data: [Vector3; 4],
}

#[derive(Debug, Clone)]
pub struct PointDamageInfo {
    pub source: Entity,
    pub point: Vector3,
    pub hit_component: Arc<str>,
    pub amount: u32,
    pub damage_type: DamageType,
}
#[derive(Debug, Clone)]
pub struct RadialDamageInfo {
    pub source: Entity,
    pub source_point: Vector3,
    pub hit_components_in_order: Vec<Arc<str>>,
    pub amount: u32,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub other: Entity,
    pub this_comp: Arc<str>,
    pub other_comp: Arc<str>,
    pub relative_velocity: Vector3,
}

#[derive(Debug, Clone)]
pub struct InteractInfo {
    pub interactor: Entity,
}

pub struct EntityVTableEntry {
    pub on_update: fn(&mut EntityStruct, f32) -> anyhow::Result<()>,
    pub on_point_damage: fn(&mut EntityStruct, PointDamageInfo) -> anyhow::Result<()>,
    pub on_radial_damage: fn(&mut EntityStruct, RadialDamageInfo) -> anyhow::Result<()>,
    pub on_collision: fn(&mut EntityStruct, CollisionInfo) -> anyhow::Result<()>,
}
impl Default for EntityVTableEntry {
    fn default() -> Self {
        Self {
            on_update: |_, _| Ok(()),
            on_point_damage: |_, _| Ok(()),
            on_radial_damage: |_, _| Ok(()),
            on_collision: |_, _| Ok(()),
        }
    }
}
pub struct GameEngineVTableEntry {
    pub on_update: fn(f32) -> anyhow::Result<()>,
    pub on_level_load: fn() -> anyhow::Result<()>,
    pub on_entity_created: fn(Entity) -> anyhow::Result<()>,
    pub on_entity_destroyed: fn(Entity) -> anyhow::Result<()>,
}
impl Default for GameEngineVTableEntry {
    fn default() -> Self {
        Self {
            on_update: |_| Ok(()),
            on_level_load: || Ok(()),
            on_entity_created: |_| Ok(()),
            on_entity_destroyed: |_| Ok(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityNicheData {
    pub is_valid: bool,
    pub generation: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityNiche {
    pub entity: RwLock<EntityStruct>,
    pub cached: RwLock<EntityStruct>,
    pub data: Mutex<EntityNicheData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CamData {
    pub pos: Vector3,
    pub up: Vector3,
    pub target: Vector3,
    pub fov: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub entities: Box<[EntityNiche]>, //always MAX_ENTITY_COUNT in length,
    pub player_entity: Mutex<Option<Entity>>,
    pub camera: Mutex<CamData>,
}

pub struct Runtime {
    pub game_mode: Mutex<GameMode>,
    pub level: Level,
    pub vtable: &'static HashMap<EntityKind, EntityVTableEntry>,
    pub game_vtable: &'static HashMap<GameMode, GameEngineVTableEntry>,
    pub event_queue: Mutex<VecDeque<Event>>,
    pub loader_info: Mutex<LoaderInfo>,
}

#[derive(Debug)]
pub enum Event {
    Collision {
        this: Entity,
        other: Entity,
        this_comp: Arc<str>,
        other_comp: Arc<str>,
        relative_velocity: Vector3,
    },
    PointDamage {
        target: Entity,
        info: PointDamageInfo,
    },
    RadialDamage {
        target: Entity,
        info: RadialDamageInfo,
    },
    DestroyEntity {
        to_destroy: Entity,
    },
}

//both files use same format, one transfers entities, one does not.
#[derive(Debug, Clone)]
pub enum LoaderInfo {
    None,
    LoadSave {
        path: Arc<str>,
    },
    LoadLevel {
        entities_to_bring_with: Vec<Entity>,
        origin_moved_to: Vector3,
        path: Arc<str>,
    },
}

lazy_static::lazy_static!(
    pub static ref RUNTIME: Runtime = Runtime {
        level: Level::new(),
        vtable: &VTABLE,
        event_queue:Mutex::new(VecDeque::new()),
        game_mode:Mutex::new(GameMode::Menu),
        game_vtable:&SYSTEM_VTABLE,
        loader_info:Mutex::new(LoaderInfo::None),
    };
);

impl Entity {
    pub fn read(&self) -> RwLockReadGuard<'static, EntityStruct> {
        if *self == Entity::default() {
            panic!("null entity");
        }
        let data = &RUNTIME.level.entities[self.index as usize];
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        assert!(genr == self.generation);
        match data.entity.try_read() {
            Ok(p) => p,
            Err(u) => match u {
                std::sync::TryLockError::Poisoned(poison_error) => poison_error.into_inner(),
                std::sync::TryLockError::WouldBlock => match data.cached.read() {
                    Ok(p) => p,
                    Err(r) => r.into_inner(),
                },
            },
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'static, EntityStruct> {
        if *self == Entity::default() {
            panic!("null entity");
        }
        let data = &RUNTIME.level.entities[self.index as usize];
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        assert!(genr == self.generation);
        match data.entity.write() {
            Ok(p) => p,
            Err(u) => u.into_inner(),
        }
    }

    pub fn read_checked(&self) -> Option<RwLockReadGuard<'static, EntityStruct>> {
        let data = RUNTIME.level.entities.get(self.index as usize)?;
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        if genr != self.generation {
            return None;
        }
        match data.entity.try_read() {
            Ok(p) => Some(p),
            Err(u) => match u {
                std::sync::TryLockError::Poisoned(poison_error) => Some(poison_error.into_inner()),
                std::sync::TryLockError::WouldBlock => match data.cached.read() {
                    Ok(p) => Some(p),
                    Err(r) => Some(r.into_inner()),
                },
            },
        }
    }

    pub fn write_checked<'a>(&'a self) -> Option<RwLockWriteGuard<'a, EntityStruct>> {
        let data = RUNTIME.level.entities.get(self.index as usize)?;
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        if genr != self.generation {
            return None;
        }
        match data.entity.write() {
            Ok(p) => Some(p),
            Err(u) => Some(u.into_inner()),
        }
    }

    pub fn try_read<'a>(&'a self) -> Result<RwLockReadGuard<'a, EntityStruct>, TryGetError> {
        let Some(data) = RUNTIME.level.entities.get(self.index as usize) else {
            return Err(TryGetError::IsInvalid);
        };
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        if genr != self.generation {
            return Err(TryGetError::IsInvalid);
        }
        match data.entity.try_read() {
            Ok(p) => Ok(p),
            Err(u) => match u {
                std::sync::TryLockError::Poisoned(poison_error) => Ok(poison_error.into_inner()),
                std::sync::TryLockError::WouldBlock => Err(TryGetError::WouldBlock),
            },
        }
    }

    pub fn try_write<'a>(&'a self) -> Result<RwLockWriteGuard<'a, EntityStruct>, TryGetError> {
        let Some(data) = RUNTIME.level.entities.get(self.index as usize) else {
            return Err(TryGetError::IsInvalid);
        };
        let genr = match data.data.lock() {
            Ok(p) => p.generation,
            Err(p) => p.into_inner().generation,
        };
        if genr != self.generation {
            return Err(TryGetError::IsInvalid);
        }
        match data.entity.try_write() {
            Ok(p) => Ok(p),
            Err(u) => match u {
                std::sync::TryLockError::Poisoned(poison_error) => Ok(poison_error.into_inner()),
                std::sync::TryLockError::WouldBlock => Err(TryGetError::WouldBlock),
            },
        }
    }
}

impl EntityStruct {
    pub fn recache_collision(&mut self) {
        let points: Vec<Vector3> = self
            .as_colliders()
            .iter()
            .flat_map(|i| i.as_vertices())
            .collect();
        let mut min = Vector3::zero();
        let mut max = Vector3::zero();
        for i in points {
            if i.x < min.x {
                min.x = i.x;
            }
            if i.y < min.y {
                min.y = i.y;
            }
            if i.z < min.z {
                min.z = i.z;
            }
            if i.x > max.x {
                max.x = i.x;
            }
            if i.y > max.y {
                max.y = i.y;
            }
            if i.z > max.z {
                max.z = i.z;
            }
        }
        min += self.position;
        max += self.position;
        self.cached_bounds = BoundingBox::new(min, max);
    }
}

pub enum TryGetError {
    WouldBlock,
    IsInvalid,
}

impl Level {
    pub fn new() -> Self {
        let list: Vec<EntityNiche> = (0..MAX_ENTITY_COUNT)
            .map(|_| EntityNiche {
                entity: RwLock::new(EntityStruct::default()),
                cached: RwLock::new(EntityStruct::default()),
                data: Mutex::new(EntityNicheData {
                    is_valid: false,
                    generation: 0,
                }),
            })
            .collect();
        Self {
            entities: list.into(),
            player_entity: Mutex::new(None),
            camera: Mutex::new(CamData {
                pos: Vector3::zero(),
                up: Vector3::up(),
                target: Vector3::forward(),
                fov: 90.,
            }),
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityStruct {
    pub fn as_colliders(&self) -> Vec<Collider3D> {
        let base = self.position;
        let mut out = Vec::new();
        for i in &self.component_table {
            let offset = i.1.offset.rotate_by(self.rotation);
            let col = Collider3D {
                pos: base + offset,
                rotation: self.rotation,
                width: i.1.width,
                height: i.1.height,
                depth: i.1.depth,
                velocity: self.velocity,
                mass: self.mass(),
                parent_entity: self.self_id,
                parent_name: i.0.clone(),
            };
            out.push(col);
        }
        out
    }

    pub fn mass(&self) -> f32 {
        1.0
    }
}

impl Level {
    //make it not O(n^2)
    pub fn physics_update(&self, delta_time: f32) {
        let cell_size = 2;
        let dt = if delta_time.is_normal() && delta_time > 0.0 {
            delta_time
        } else {
            1. / 60.
        };
        let mut collider_set = Vec::new();
        let mut old_locs = HashMap::new();
        let mut acc_table: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
        acc_table.reserve(self.entities.len() * 4);
        for (idx, i) in self.entities().enumerate() {
            let mut y = i.write();
            old_locs.insert(i, (y.position, y.rotation));
            let vel = y.velocity;
            y.position += vel * dt;
            let tmp = y.as_colliders();
            y.recache_collision();
            collider_set.push((i, tmp));
            let pos = (
                y.position.x as i32 / cell_size,
                y.position.y as i32 / cell_size,
                y.position.z as i32 / cell_size,
            );
            if let Some(p) = acc_table.get_mut(&pos) {
                p.push(idx);
            } else {
                acc_table.insert(pos, vec![idx]);
            }
        }
        let hitset = Mutex::new(HashSet::new());
        (0..collider_set.len()).into_par_iter().for_each(|i| {
            let mut collided = false;
            let mut t1r = collider_set[i].0.read();
            if t1r.component_table.is_empty() {
                return;
            }
            if t1r.is_static {
                return;
            }
            let pos = (
                t1r.position.x as i32 / cell_size,
                t1r.position.y as i32 / cell_size,
                t1r.position.z as i32 / cell_size,
            );
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let mut tmp_pos = pos;
                        tmp_pos.0 += dx;
                        tmp_pos.1 += dy;
                        tmp_pos.2 += dz;
                        if let Some(t) = acc_table.get(&tmp_pos) {
                            for j in t.iter().copied() {
                                if i == j {
                                    continue;
                                }
                                let t2 = &collider_set[j];
                                let t2r = t2.0.read();
                                if t2r.is_projectile {
                                    continue;
                                }
                                if t2r.position.distance_to(t1r.position) > 10. {
                                    continue;
                                }
                                if t2r.component_table.is_empty() {
                                    continue;
                                }
                                let b1 = t1r.cached_bounds;
                                let b2 = t2r.cached_bounds;
                                if !b1.check_collision_boxes(b2) {
                                    continue;
                                }
                                'ob: for k in &collider_set[i].1 {
                                    for l in &t2.1 {
                                        if k.check_collision(l) {
                                            let mut lck = hitset.lock().unwrap();

                                            collided = true;
                                            if !lck.contains(&(i, j)) {
                                                collision_event(
                                                    k.parent_entity,
                                                    l.parent_entity,
                                                    k.parent_name.clone(),
                                                    l.parent_name.clone(),
                                                    k.velocity - l.velocity,
                                                );
                                                collision_event(
                                                    l.parent_entity,
                                                    k.parent_entity,
                                                    l.parent_name.clone(),
                                                    k.parent_name.clone(),
                                                    k.velocity - l.velocity,
                                                );
                                                lck.insert((i, j));
                                                lck.insert((j, i));
                                            }
                                            break 'ob;
                                        }
                                    }
                                }
                            }
                            if collided {
                                drop(t1r);
                                collider_set[i].0.write().position = old_locs[&collider_set[i].0].0;
                                t1r = collider_set[i].0.read();
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn update(&self, delta_time: f32) {
        for i in self.entities() {
            let mut wrt = match self.entities[i.index as usize].cached.write() {
                Ok(x) => x,
                Err(x) => x.into_inner(),
            };
            let w = match self.entities[i.index as usize].entity.read() {
                Ok(x) => x,
                Err(x) => x.into_inner(),
            };
            *wrt = w.clone();
        }
        for i in self.entities() {
            let mut x = i.write();
            if let Some(updater) = RUNTIME.vtable.get(&x.kind).map(|i| i.on_update) {
                let res = (updater)(&mut x, delta_time);
                if res.is_err() {
                    todo!()
                }
                let tmp = x.clone();
                let mut cached = match self.entities[i.index as usize].cached.write() {
                    Ok(x) => x,
                    Err(x) => x.into_inner(),
                };
                *cached = tmp;
                x.recache_collision();
            }
        }
    }

    pub fn tick(
        &'static self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        assets: &mut Assetpack,
    ) {
        let delta_time = handle.get_frame_time();
        _ = thread;
        self.update(delta_time);
        if let Some(x) = RUNTIME.game_vtable.get(&*RUNTIME.game_mode.lock().unwrap()) {
            let rs = (x.on_update)(delta_time);
            if rs.is_err() {
                todo!()
            }
        }
        let join = std::thread::spawn(move || {
            self.physics_update(delta_time);
        });
        self.graphics_update(handle, thread, assets);
        join.join().unwrap();
        self.poll_events();
    }

    pub fn graphics_update(
        &self,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
        assets: &mut Assetpack,
    ) {
        _ = assets;
        let mut cam_data = self.camera.lock().unwrap();
        let mut cam =
            Camera3D::perspective(cam_data.pos, cam_data.target, cam_data.up, cam_data.fov);
        if let Some(player) = self.player_entity.lock().unwrap().as_ref() {
            let et = player.read();
            cam.position = et.position;
            cam.target = et.position + Vector3::forward().rotate_by(et.rotation);
            cam.up = Vector3::up().rotate_by(et.rotation);
        } else {
            handle.update_camera(&mut cam, raylib::ffi::CameraMode::CAMERA_FREE);
        }
        cam_data.fov = cam.fovy;
        cam_data.pos = cam.position;
        cam_data.target = cam.target;
        cam_data.up = cam.up;
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(Color::BLACK);
        let mut mode = draw.begin_mode3D(cam);
        for i in self.entities() {
            let j = i.read();
            let pos = j.position;
            for k in j.component_table.values() {
                let comp_pos = pos + k.offset.rotate_by(j.rotation);
                if comp_pos.distance_to(cam_data.pos) > 200. {
                    continue;
                }
                match k.render_as {
                    RenderKind::Cube => {
                        mode.draw_cube(comp_pos, k.width, k.height, k.depth, k.color);
                    }
                    RenderKind::Cylinder => {
                        mode.draw_cylinder(comp_pos, k.width, k.width, k.height, 30, k.color);
                    }
                    RenderKind::Sphere => {
                        mode.draw_sphere(comp_pos, k.width, k.color);
                    }
                    RenderKind::Cone => {
                        mode.draw_cylinder(comp_pos, 0.0, k.width, k.height, 30, k.color);
                    }
                }
            }
        }
        drop(mode);
        draw.draw_fps(1500, 60);
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> {
        pub struct EntityIterator {
            pub current: usize,
        }
        impl Iterator for EntityIterator {
            type Item = Entity;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    let tmp = &RUNTIME.level.entities[self.current].data.lock().unwrap();
                    if tmp.is_valid {
                        let out = Entity {
                            index: self.current as u32,
                            generation: tmp.generation,
                        };
                        self.current += 1;
                        return Some(out);
                    } else {
                        self.current += 1;
                        if self.current >= RUNTIME.level.entities.len() {
                            return None;
                        }
                    }
                }
            }
        }
        EntityIterator { current: 0 }
    }

    pub fn poll_events(&self) {
        while let Some(e) = next_event() {
            //  println!("event:{:#?}", e);
            match e {
                Event::Collision {
                    this,
                    other: _,
                    this_comp: _,
                    other_comp: _,
                    relative_velocity: _,
                } => {
                    if let Some(mut r) = this.write_checked() {
                        r.velocity *= -0.8;
                    }
                }
                Event::PointDamage { target, info } => {
                    if let Some(mut et) = target.write_checked() {
                        let kind = et.kind;
                        if let Some(vt) = RUNTIME.vtable.get(&kind) {
                            let rs = (vt.on_point_damage)(&mut et, info);
                            if rs.is_err() {
                                todo!()
                            }
                        }
                    } else {
                        todo!()
                    }
                }
                Event::RadialDamage { target, info } => {
                    if let Some(mut et) = target.write_checked() {
                        let kind = et.kind;
                        if let Some(vt) = RUNTIME.vtable.get(&kind) {
                            let rs = (vt.on_radial_damage)(&mut et, info);
                            if rs.is_err() {
                                todo!()
                            }
                        }
                    } else {
                        todo!()
                    }
                }
                Event::DestroyEntity { to_destroy } => {
                    let tmp = &self.entities[to_destroy.index as usize];
                    let mut lck = tmp.data.lock().unwrap();
                    if lck.generation == to_destroy.generation {
                        lck.is_valid = false;
                    }
                    if let Some(gm) = RUNTIME.game_vtable.get(&get_game_mode()) {
                        let rs = (gm.on_entity_destroyed)(to_destroy);
                        if rs.is_err() {
                            todo!()
                        }
                    }
                }
            }
        }
    }
}

pub fn collision_event(
    this: Entity,
    other: Entity,
    this_comp: Arc<str>,
    other_comp: Arc<str>,
    relative_velocity: Vector3,
) {
    let mut guard = event_queue();
    guard.push_back(Event::Collision {
        this,
        other,
        this_comp,
        other_comp,
        relative_velocity,
    });
}

pub fn event_queue() -> MutexGuard<'static, VecDeque<Event>> {
    match RUNTIME.event_queue.lock() {
        Ok(x) => x,
        Err(x) => x.into_inner(),
    }
}

pub fn next_event() -> Option<Event> {
    event_queue().pop_front()
}

pub fn create_entity() -> Entity {
    for i in 0..RUNTIME.level.entities.len() {
        let mut g = RUNTIME.level.entities[i].data.lock().unwrap();
        if !g.is_valid {
            g.generation += 1;
            g.is_valid = true;
            let mut r = RUNTIME.level.entities[i].cached.write().unwrap();
            let mut r2 = RUNTIME.level.entities[i].entity.write().unwrap();
            *r = EntityStruct::default();
            *r2 = EntityStruct::default();
            let out = Entity {
                index: i as u32,
                generation: g.generation,
            };
            return out;
        }
    }
    Entity::default()
}

pub fn destroy_entity(entity: Entity) {
    event_queue().push_back(Event::DestroyEntity { to_destroy: entity });
}

pub fn create_debug_cube(at: Vector3, vel: Vector3, size: f32, color: Color) -> Entity {
    let out = create_entity();
    let mut f = out.write();
    f.position = at;
    f.rotation = Quaternion::identity();
    f.self_id = out;
    f.velocity = vel;
    let mut comp = EntityComponent::default();
    comp.parent_id = out;
    comp.color = color;
    comp.depth = size;
    comp.offset = Vector3::zero();
    comp.health = 1;
    comp.height = size;
    comp.width = size;
    comp.render_as = RenderKind::Cube;
    f.component_table.insert("box".into(), comp);
    out
}

pub fn game_loop(setup: impl FnOnce()) {
    let (mut handle, thread) = raylib::RaylibBuilder::default().build();
    setup();
    let mut assets = Assetpack {
        textures: HashMap::new(),
        meshes: HashMap::new(),
        shaders: HashMap::new(),
    };
    while !handle.window_should_close() {
        RUNTIME.level.tick(&mut handle, &thread, &mut assets);
    }
}

pub fn raycast(start: Vector3, end: Vector3, ignored: &[Entity]) -> Option<ColData3D> {
    let mut min_dist = end.distance_to(start);
    let mut out = None;
    for i in RUNTIME.level.entities() {
        if ignored.contains(&i) {
            continue;
        }
        let tmp = i.read();
        if tmp.is_projectile {
            continue;
        }
        for j in tmp.as_colliders() {
            if let Some(mut x) = j.raycast(start, (end - start).normalized())
                && x.dist < min_dist
            {
                x.hit_entity = i;
                out = Some(x);
                min_dist = x.dist;
            }
        }
    }
    out
}

pub fn raycast_projectiles(start: Vector3, end: Vector3, ignored: &[Entity]) -> Option<ColData3D> {
    let mut min_dist = end.distance_to(start);
    let mut out = None;
    for i in RUNTIME.level.entities() {
        if ignored.contains(&i) {
            continue;
        }
        let tmp = i.read();
        if tmp.is_projectile {
            continue;
        }
        for j in tmp.as_colliders() {
            if let Some(mut x) = j.raycast(start, (end - start).normalized())
                && x.dist < min_dist
            {
                x.hit_entity = i;
                out = Some(x);
                min_dist = x.dist;
            }
        }
    }
    out
}

pub fn raycast_by_kinds(
    start: Vector3,
    end: Vector3,
    ignored: &[Entity],
    kinds: &[EntityKind],
) -> Option<ColData3D> {
    let mut min_dist = end.distance_to(start);
    let mut out = None;
    for i in RUNTIME.level.entities() {
        if ignored.contains(&i) {
            continue;
        }

        let tmp = i.read();
        if tmp.is_projectile {
            continue;
        }
        if !kinds.contains(&tmp.kind) {
            continue;
        }
        for j in tmp.as_colliders() {
            if let Some(mut x) = j.raycast(start, (end - start).normalized())
                && x.dist < min_dist
            {
                x.hit_entity = i;
                out = Some(x);
                min_dist = x.dist;
            }
        }
    }
    out
}

pub fn get_game_mode() -> GameMode {
    *RUNTIME.game_mode.lock().unwrap()
}

pub fn set_game_mode(mode: GameMode) {
    *RUNTIME.game_mode.lock().unwrap() = mode;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub entities: HashMap<Entity, EntityStruct>,
    pub assets: HashMap<String, Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Asset {
    Texture(DataKind),
    Mesh(DataKind),
    Sound(DataKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    Path(Arc<str>),
    Data(Arc<[u8]>),
}

pub struct Assetpack {
    pub textures: HashMap<Arc<str>, raylib::texture::Texture2D>,
    pub meshes: HashMap<Arc<str>, raylib::models::Model>,
    pub shaders: HashMap<Arc<str>, Shader>,
}

pub fn transfer_entities_to(data: &mut LevelData, entities: HashMap<Entity, EntityStruct>) {
    let mut lut = HashMap::new();
    for i in &entities {
        for j in 0..MAX_ENTITY_COUNT {
            let tmp = Entity {
                index: j as u32,
                generation: 1,
            };
            if !data.entities.contains_key(&tmp) {
                lut.insert(*i.0, tmp);
                data.entities.insert(tmp, i.1.clone());
                break;
            }
        }
    }
    let new_list = lut.iter().map(|i| i.1.clone()).collect::<Vec<_>>();
    for i in new_list {
        let gt = data.entities.get_mut(&i).unwrap();
        for i in gt.component_table.iter_mut() {
            i.1.parent_id = lut[&i.1.parent_id];
            for j in &mut i.1.entity_data {
                if lut.contains_key(&*j) {
                    *j = lut[&*j];
                }
            }
        }
        for i in gt.data_component_table.iter_mut() {
            i.1.parent_id = lut[&i.1.parent_id];
            for j in &mut i.1.entity_data {
                if lut.contains_key(&*j) {
                    *j = lut[&*j];
                }
            }
        }
    }
    let entity_list = data
        .entities
        .iter()
        .map(|i| i.0.clone())
        .collect::<Vec<_>>();
    let mut lut2 = HashMap::new();
    for i in entity_list {
        lut2.insert(
            i,
            Entity {
                index: i.index,
                generation: 1,
            },
        );
    }
    drop(lut);
    let mut data2 = HashMap::new();
    for i in data.entities.iter_mut() {
        for i in i.1.component_table.iter_mut() {
            i.1.parent_id = lut2[&i.1.parent_id];
            for j in &mut i.1.entity_data {
                if lut2.contains_key(&*j) {
                    *j = lut2[&*j];
                } else {
                    *j = Entity::default();
                }
            }
        }
        for i in i.1.data_component_table.iter_mut() {
            i.1.parent_id = lut2[&i.1.parent_id];
            for j in &mut i.1.entity_data {
                if lut2.contains_key(&*j) {
                    *j = lut2[&*j];
                } else {
                    *j = Entity::default();
                }
            }
        }
        data2.insert(lut2[&i.0], i.1.clone());
    }
    data.entities = data2;
}
pub enum EntityRef<'a> {
    Reference(&'a mut EntityStruct),
    Write(RwLockWriteGuard<'a, EntityStruct>),
    Read(RwLockReadGuard<'a, EntityStruct>),
}

impl<'a> EntityRef<'a> {
    pub fn read(&self) -> &EntityStruct {
        match self {
            Self::Reference(r) => *r,
            Self::Write(r) => &**r,
            Self::Read(r) => &**r,
        }
    }
    pub fn write(&mut self) -> &mut EntityStruct {
        match self {
            Self::Reference(r) => *r,
            Self::Write(r) => &mut **r,
            Self::Read(_) => {
                todo!()
            }
        }
    }
    pub fn try_write(&mut self) -> Option<&mut EntityStruct> {
        match self {
            Self::Reference(r) => Some(*r),
            Self::Write(r) => Some(&mut **r),
            Self::Read(_) => None,
        }
    }
}

pub trait EntityWrapper<'a>: Sized + 'a {
    fn create(x: EntityRef<'a>) -> Result<Self, EntityRef<'a>>;
}

#[macro_export]
macro_rules! make_trait_wrapper {
    ($name:ident,
        $kind:expr,
        $constructor:expr, (constructor_args $(
            $arg_name:ident:$arg_t:ty
        $(,)?)*),
        (data component names: $(($data_comp_name:literal:$data_comp_kind:expr)$(,)?)*),
        ($(($var_name:ident, $var_type:ty,$source_comp:literal,$source_name:ident, $source_idx:literal, $data_getter_name:ident, $data_getter_mut_name:ident)$(,)?)*), ($(
            ($comp_name:literal, $comp_kind:expr, $comp_type:ty, $getter_name:ident, $getter_mut_name:ident)
        )*)) => {
        pub struct $name<'a> {
            value: EntityRef<'a>,
        }
        impl<'a> $name<'a>
        {
            pub fn get(&self)->&EntityStruct{
                self.value.read()
            }

            pub fn get_mut(&mut self)->&mut EntityStruct{
                self.value.write()
            }
            $(
                pub fn $data_getter_name(&self)->&$var_type{
                    &self.value.read().data_component_table.get($source_comp).unwrap().$source_name[$source_idx]
                }

                pub fn $data_getter_mut_name(&mut self)->&mut $var_type{
                    &mut self.value.write().data_component_table.get_mut($source_comp).unwrap().$source_name[$source_idx]
                }
            )*
            $(
                pub fn $getter_name<'b>(&'b self)->$comp_type{
                   <$comp_type>::from_ref(self.get().component_table.get($comp_name).unwrap())
                }
                pub fn $getter_mut_name<'b>(&'b mut self)->$comp_type{
                   <$comp_type>::from_mut(self.get_mut().component_table.get_mut($comp_name).unwrap())
                }
            )*
            pub fn new(pos:raylib::math::Vector3,$($var_name:$var_type,)* $($arg_name:$arg_t,)*)->Entity{
                let  out = create_entity();
                let out_act = out.clone();
                let mut ent = out.write();
                ent.position = pos;
                $(
                    {
                    let mut comp =crate::system::EntityDataComponent::default();
                    comp.parent_id = out;
                    comp.kind = $data_comp_kind;
                    ent.data_component_table.insert(
                        $data_comp_name.into(),comp
                    );
                    }
                )*
                $(
                    {
                    let mut comp = crate::system::EntityComponent::default();
                    comp.kind = $comp_kind;
                    comp.width = 1.0;
                    comp.height = 1.0;
                    comp.depth = 1.0;
                    comp.parent_id= out;
                    ent.component_table.insert($comp_name.into(), comp);
                    }
                )*
                let mut v = Self{
                    value:EntityRef::Write(ent)
                };
                $(
                    *v.$data_getter_mut_name() = $var_name;
                )*
                $constructor(&mut v,$($arg_name,)*);
                out_act
            }
        }
        impl<'a> EntityWrapper<'a> for $name<'a>{
            fn create(x:EntityRef<'a>)->Result<Self, EntityRef<'a>>{
                if x.read().kind != $kind{
                    Err(x)
                } else{Ok(Self{value:x})}
            }
        }
    };
}

pub enum ComponentRef<'a> {
    Reference(&'a EntityComponent),
    ReferenceMut(&'a mut EntityComponent),
}
impl<'a> ComponentRef<'a> {
    pub fn get(&self) -> &EntityComponent {
        match self {
            Self::Reference(x) => x,
            Self::ReferenceMut(x) => x,
        }
    }
    pub fn get_mut(&mut self) -> &mut EntityComponent {
        match self {
            Self::Reference(_) => {
                todo!();
            }
            Self::ReferenceMut(x) => x,
        }
    }
}

#[macro_export]
macro_rules! make_component_wrapper {
    ($name:ident,
        $kind:expr,
        ($(($var_name:ident, $var_type:ty,$source_name:ident, $source_idx:literal, $getter_name:ident, $getter_mut_name:ident)$(,)?)*)) => {
        pub struct $name<'a> {
            value: ComponentRef<'a>,
        }
        impl<'a> $name<'a>
        {
            pub fn from_ref(v:&'a EntityComponent)->Self{
                Self{
                    value:ComponentRef::Reference(v)
                }
            }
            pub fn from_mut(v:&'a mut EntityComponent)->Self{
                Self{
                    value:ComponentRef::ReferenceMut(v)
                }
            }
            pub fn get(&self)->&EntityComponent{
                self.value.get()
            }

            pub fn get_mut(&mut self)->&mut EntityComponent{
                self.value.get_mut()
            }
            $(
                pub fn $getter_name(&self)->&$var_type{
                    &self.value.get().$source_name[$source_idx]
                }

                pub fn $getter_mut_name(&mut self)->&mut $var_type{
                    &mut self.value.get_mut().$source_name[$source_idx]
                }
            )*
        }
    }
}
