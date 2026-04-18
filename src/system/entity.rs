pub use anyhow::Result;
use raylib::camera::Camera3D;
use raylib::color::Color;
pub use raylib::math::{BoundingBox, RayCollision, Vector3};
use raylib::math::{Matrix, Quaternion, Vector2, Vector4};
use raylib::models::{Mesh, Model, RaylibMesh, RaylibModel};
use raylib::prelude::RenderTexture2D;
use raylib::prelude::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt, RaylibShaderModeExt};
use raylib::shaders::{RaylibShader, Shader};
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
    pub lights: Vec<Light>,
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
    pub vector_data_lists: [Vec<Vector3>; 4],
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
    pub on_update: fn(f32, handle: &mut RaylibHandle) -> anyhow::Result<()>,
    pub on_level_load: fn() -> anyhow::Result<()>,
    pub on_entity_created: fn(Entity) -> anyhow::Result<()>,
    pub on_entity_destroyed: fn(Entity) -> anyhow::Result<()>,
}

impl Default for GameEngineVTableEntry {
    fn default() -> Self {
        Self {
            on_update: |_, _| Ok(()),
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
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Light {
    pub position: Vector3,
    pub color: Vector4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LightingData {
    pub ambient_light_color: Vector4,
    pub directional_light_direction: Vector3,
    pub directional_light_color: Vector4,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub lighting_data: Mutex<LightingData>,
    pub entities: Box<[EntityNiche]>, //always MAX_ENTITY_COUNT in length,
    pub player_entity: Mutex<Option<Entity>>,
    pub camera: Mutex<CamData>,
    pub star_positions: Mutex<(Box<[Vector2]>, Box<[f32]>)>,
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
            lighting_data: Mutex::new(LightingData {
                ambient_light_color: Vector4::new(1.0 * 0.1, 0.95 * 0.1, 0.9 * 0.1, 1.0),
                directional_light_color: Vector4::new(1.0 * 0.5, 0.95 * 0.5, 0.9 * 0.5, 1.0),
                directional_light_direction: (Vector3::forward() + Vector3::up()).normalized(),
            }),
            star_positions: Mutex::new((Box::new([]), Box::new([]))),
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
            let rs = (x.on_update)(delta_time, handle);
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
        unsafe {
            raylib::ffi::rlSetClipPlanes(0.001, 100000.0);
        }
        let mut lights: Vec<Light> = Vec::new();
        for i in self.entities() {
            let g = i.read();
            for j in &g.component_table {
                for t in &j.1.lights {
                    let mut tmp = *t;
                    let offset = j.1.offset + tmp.position;
                    let base = g.position;
                    tmp.position = base + offset.rotate_by(g.rotation);
                    lights.push(tmp);
                }
            }
        }
        let shade = assets.shaders.get_mut("shader").unwrap();
        let positions_idx = shade.get_shader_location("light_positions");
        let colors_idx = shade.get_shader_location("light_colors");
        let len_idx = shade.get_shader_location("light_count");
        {
            let data = RUNTIME.level.lighting_data.lock().unwrap();
            let cam_pos_idx = shade.get_shader_location("camera_position");
            let ambient_idx = shade.get_shader_location("ambient");
            let direction_idx = shade.get_shader_location("directional_light_direction");
            let direction_color_idx = shade.get_shader_location("directional_light_color");
            shade.set_shader_value(len_idx, lights.len() as i32);
            shade.set_shader_value(cam_pos_idx, cam.position);
            shade.set_shader_value(ambient_idx, data.ambient_light_color);
            shade.set_shader_value(direction_idx, data.directional_light_direction);
            shade.set_shader_value(direction_color_idx, data.directional_light_color);
        }
        let _ = shade;
        let mut mode = draw.begin_mode3D(cam);
        for i in self.entities() {
            let j = i.read();
            let pos = j.position;
            for k in j.component_table.values() {
                let comp_pos = pos + k.offset.rotate_by(j.rotation);
                if comp_pos.distance_to(cam_data.pos) > 200. {
                    //          continue;
                }
                let mut positions = [Vector3::zero(); 16];
                let mut colors = [Vector4::new(0.0, 0.0, 0.0, 0.0); 16];
                let mut con = lights.clone();
                con.sort_by(|i, j| {
                    i.position
                        .distance_to(comp_pos)
                        .partial_cmp(&j.position.distance_to(comp_pos))
                        .unwrap()
                });
                con = con.into_iter().take(16).collect();
                for (idx, i) in con.iter().enumerate() {
                    positions[idx] = i.position;
                    colors[idx] = i.color;
                }
                {
                    let shade = assets.shaders.get_mut("shader").unwrap();
                    shade.set_shader_value_v(positions_idx, &positions);
                    shade.set_shader_value_v(colors_idx, &colors);
                    shade.set_shader_value(len_idx, con.len() as i32);
                }
                let mesh = match k.render_as {
                    RenderKind::Cube => assets.meshes.get_mut("cube").unwrap(),
                    RenderKind::Cylinder => assets.meshes.get_mut("cylinder").unwrap(),
                    RenderKind::Sphere => assets.meshes.get_mut("sphere").unwrap(),
                    RenderKind::Cone => assets.meshes.get_mut("cone").unwrap(),
                };
                let old = mesh.materials_mut()[0].shader;
                mesh.materials_mut()[0].shader = **assets.shaders.get_mut("shader").unwrap();
                let old_tex = unsafe { (*mesh.materials_mut()[0].maps.add(1)).texture };
                unsafe {
                    (*mesh.materials_mut()[0].maps.add(1)).texture = assets.lightmap.texture;
                }
                let old_trans = mesh.transform;
                mesh.transform =
                    (j.rotation.to_matrix() * Matrix::scale(k.width, k.height, k.depth)).into();
                mode.draw_model(&mesh, comp_pos, 1.0, Color::WHITE);
                mesh.materials_mut()[0].shader = old;
                mesh.transform = old_trans;
                unsafe {
                    (*mesh.materials_mut()[0].maps.add(1)).texture = old_tex;
                }
            }
        }
        unsafe {
            raylib::ffi::rlDisableBackfaceCulling();
            let cb = assets.meshes.get_mut("sky").unwrap();
            let old = cb.materials()[0].shader;
            let shader = assets.shaders.get_mut("sky_shader").unwrap();
            cb.materials_mut()[0].shader = **shader;
            (*cb.materials_mut()[0].maps).texture = *assets.textures["sky"];
            mode.draw_model(&*cb, Vector3::zero(), 5000.0, Color::WHITE);
            cb.materials_mut()[0].shader = old;
            raylib::ffi::rlEnableBackfaceCulling();
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
    let mut assets = create_asset_pack(&mut handle, &thread);
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
    pub lightmap: RenderTexture2D,
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

pub fn create_asset_pack(handle: &mut RaylibHandle, thread: &RaylibThread) -> Assetpack {
    let shader = handle.load_shader(thread, None, Some("shaders/star_frag.glsl"));
    let cube_mesh = raylib::prelude::Mesh::gen_mesh_cube(&thread, 1., 1., 1.);
    let sphere_mesh = raylib::prelude::Mesh::gen_mesh_sphere(thread, 1., 20, 20);
    let sky_mesh = raylib::prelude::Mesh::gen_mesh_sphere(thread, 1., 3, 3);
    let cone_mesh = raylib::prelude::Mesh::gen_mesh_cone(thread, 0.1, 1.0, 32);
    let cylinder_mesh = raylib::prelude::Mesh::gen_mesh_cylinder(thread, 0.1, 1., 32);
    let mut mesh_map: HashMap<Arc<str>, Model> = HashMap::new();
    unsafe {
        mesh_map.insert(
            "cube".into(),
            handle
                .load_model_from_mesh(thread, cube_mesh.make_weak())
                .unwrap(),
        );
        mesh_map.insert(
            "sphere".into(),
            handle
                .load_model_from_mesh(thread, sphere_mesh.make_weak())
                .unwrap(),
        );
        mesh_map.insert(
            "cone".into(),
            handle
                .load_model_from_mesh(thread, cone_mesh.make_weak())
                .unwrap(),
        );
        mesh_map.insert(
            "cylinder".into(),
            handle
                .load_model_from_mesh(thread, cylinder_mesh.make_weak())
                .unwrap(),
        );
        mesh_map.insert(
            "sky".into(),
            handle
                .load_model_from_mesh(thread, sky_mesh.make_weak())
                .unwrap(),
        );
    }
    let mut shaders: HashMap<_, Shader> = HashMap::new();
    shaders.insert("sky_shader".into(), shader);
    let gshader = handle.load_shader(thread, Some("shaders/vert.glsl"), Some("shaders/frag.glsl"));
    shaders.insert("shader".into(), gshader);
    let mut textures = HashMap::new();
    let h = handle.load_texture(thread, "hazy_nebulae_1.png").unwrap();

    textures.insert("sky".into(), h);
    let lightmap = handle.load_render_texture(thread, 4096, 4096).unwrap();
    Assetpack {
        textures,
        meshes: mesh_map,
        shaders: shaders,
        lightmap,
    }
}

pub fn get_star_data() -> MutexGuard<'static, (Box<[Vector2]>, Box<[f32]>)> {
    RUNTIME.level.star_positions.lock().unwrap()
}
