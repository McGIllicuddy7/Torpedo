use std::{
    any::{Any, TypeId, type_name},
    collections::{BTreeMap, VecDeque},
    f32::consts::PI,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use rand::random;
use raylib::{
    RaylibHandle, RaylibThread,
    drawing::RaylibDrawHandle,
    math::{BoundingBox, Quaternion, Ray, Vector3, Vector4},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    graphics::{DrawEvent, DrawEvent3D, ParticleSystemContainer, run_graphics},
    mesh::GameMesh,
};
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum EventKind {
    OnDamage,
    OnDestroy,
    DestroyObject,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventInfo {
    OnDamage {
        direction: Vector3,
        damage_amount: i32,
        penetration: i32,
        aoe: bool,
    },
    OnDestroy {},
    DestroyObject {},
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GObject {
    pub idx: u32,
    pub generation: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct CameraData {
    pub position: Vector3,
    pub rotation: Vector4,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameObjectData {
    pub model: Option<GameMesh>,
    pub location: Vector3,
    pub rotation: Vector4,
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub velocity: Vector3,
    pub angular_velocity: Vector3,
    pub camera_data: Option<CameraData>,
    pub is_projectile: bool,
    pub is_static: bool,
    pub tags: Arc<[Arc<str>]>,
    pub self_id: GObject,
    pub projectile_damage: i32,
    pub projectile_penetration: i32,
    pub allegience: i32,
}

pub trait GameObject: Send + Sync + Any + 'static {
    fn on_update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread);
    fn on_event(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, ev: Event);
    fn get_data(&self) -> &GameObjectData;
    fn get_data_mut(&mut self) -> &mut GameObjectData;
}

pub struct GameObjectBox {
    pub ptr: Option<Box<dyn GameObject>>,
    pub generation: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameObjectSerialized {
    pub type_name: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub source: GObject,
    pub target: GObject,
    pub info: EventInfo,
}

pub struct Engine {
    pub objects: [RwLock<GameObjectBox>; 16384],
    pub particle_systems: [Mutex<ParticleSystemContainer>; 256],
    pub events: Mutex<VecDeque<Event>>,
    pub draw_events: Mutex<VecDeque<DrawEvent>>,
    pub draw_events_3d: Mutex<VecDeque<DrawEvent3D>>,
    pub player_object: Mutex<GObject>,
    pub camera_data: Mutex<GCameraData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GCameraData {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
}

pub struct HitInfo {
    pub hit_location: Vector3,
    pub start: Vector3,
    pub distance: f32,
    pub normal: Vector3,
    pub hit_object: GObject,
}
pub static ENGINE: Engine = Engine {
    objects: [const {
        RwLock::new(GameObjectBox {
            ptr: None,
            generation: 0,
        })
    }; _],
    particle_systems: [const {
        Mutex::new(ParticleSystemContainer {
            v: None,
            generation: 0,
        })
    }; _],
    events: Mutex::new(VecDeque::new()),
    draw_events: Mutex::new(VecDeque::new()),
    draw_events_3d: Mutex::new(VecDeque::new()),
    player_object: Mutex::new(GObject {
        idx: 0,
        generation: 0,
    }),
    camera_data: Mutex::new(GCameraData {
        position: Vector3::new(-1.0, 0.0, 0.0),
        target: Vector3::new(0.0, 0.0, 0.0),
        up: Vector3::new(0.0, 0.0, 1.0),
    }),
};

pub fn step(handle: &mut RaylibHandle, thread: &RaylibThread, game_mode: &mut dyn GameMode) {
    for i in &ENGINE.objects {
        if let Some(obj) = i.try_write().unwrap().ptr.as_mut() {
            obj.on_update(handle, thread);
        }
    }
    {
        let dt = handle.get_frame_time();
        (0..ENGINE.particle_systems.len())
            .into_par_iter()
            .for_each(|i| {
                let mut g = ENGINE.particle_systems[i].lock().unwrap();
                let mut should_gc = false;
                if let Some(g) = g.v.as_mut() {
                    g.update(dt);
                    if g.should_be_gced() {
                        should_gc = true;
                    }
                }
                if should_gc {
                    g.v = None;
                }
            });
    }
    game_mode.on_update(handle, thread);
    loop {
        let mut _tmp = ENGINE.events.lock().unwrap();
        let Some(ev) = _tmp.pop_front() else {
            break;
        };
        drop(_tmp);
        match ev.kind() {
            EventKind::DestroyObject => {
                let mut tmp = ENGINE.objects[ev.target.idx as usize].write().unwrap();
                if tmp.generation == ev.target.generation {
                    if let Some(t) = tmp.ptr.as_mut() {
                        let ev2 = Event {
                            source: ev.source,
                            target: ev.target,
                            info: EventInfo::OnDestroy {},
                        };
                        t.on_event(handle, thread, ev2);
                    }
                    tmp.ptr = None;
                }
            }
            _ => {
                let mut tmp = ENGINE.objects[ev.target.idx as usize].write().unwrap();
                if tmp.generation == ev.target.generation
                    && let Some(t) = tmp.ptr.as_mut()
                {
                    t.on_event(handle, thread, ev);
                }
            }
        }
    }
    update_physics();
    run_graphics(handle, thread, game_mode);
    let mut pobj = ENGINE.player_object.lock().unwrap();
    if !pobj.is_valid() {
        *pobj = GObject::new();
        drop(pobj);
        //    let mut lck = ENGINE.camera_data.lock().unwrap();
        // lck.target = Vector3::new(0.0, 0.0, 0.0);
        // lck.position = Vector3::new(-1., 0.0, 0.0);
        //lck.up = Vector3::new(0., 0.0, 1.0);
    }
}
impl Event {
    pub fn kind(&self) -> EventKind {
        match self.info {
            EventInfo::OnDamage {
                direction: _,
                damage_amount: _,
                penetration: _,
                aoe: _,
            } => EventKind::OnDamage,
            EventInfo::OnDestroy {} => EventKind::OnDestroy,
            EventInfo::DestroyObject {} => EventKind::DestroyObject,
        }
    }
}

impl Default for GObject {
    fn default() -> Self {
        Self::new()
    }
}

impl GObject {
    pub const fn new() -> Self {
        Self {
            idx: 0,
            generation: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        if self.generation == 0 && self.idx == 0 {
            return false;
        }
        let tmp = ENGINE.objects[self.idx as usize].read().unwrap();
        tmp.generation == self.generation && tmp.ptr.is_some()
    }

    pub fn with<W>(&self, func: impl FnOnce(&dyn GameObject) -> W) -> Option<W> {
        let r = ENGINE.objects[self.idx as usize].read().unwrap();
        if r.generation == self.generation {
            r.ptr.as_ref().map(|t| func(t.as_ref()))
        } else {
            None
        }
    }

    pub fn with_mut<W>(&self, func: impl FnOnce(&mut dyn GameObject) -> W) -> Option<W> {
        let mut r = ENGINE.objects[self.idx as usize].write().unwrap();
        if r.generation == self.generation {
            r.ptr.as_mut().map(|t| func(t.as_mut()))
        } else {
            None
        }
    }

    pub fn get_checked<'a>(&'a self) -> Option<GObjectRead<'a>> {
        let r = ENGINE.objects[self.idx as usize].read().unwrap();
        if r.generation == self.generation {
            if r.ptr.is_some() {
                Some(GObjectRead { inner: r })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut_checked<'a>(&'a self) -> Option<GObjectWrite<'a>> {
        let r = ENGINE.objects[self.idx as usize].write().unwrap();
        if r.generation == self.generation {
            if r.ptr.is_some() {
                Some(GObjectWrite { inner: r })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get<'a>(&'a self) -> GObjectRead<'a> {
        self.get_checked().unwrap()
    }

    pub fn get_mut<'a>(&'a self) -> GObjectWrite<'a> {
        self.get_mut_checked().unwrap()
    }
}

pub struct GObjectRead<'a> {
    inner: RwLockReadGuard<'a, GameObjectBox>,
}
impl<'a> Deref for GObjectRead<'a> {
    type Target = dyn GameObject;
    fn deref(&self) -> &Self::Target {
        self.inner.ptr.as_ref().unwrap().as_ref()
    }
}

pub struct GObjectWrite<'a> {
    inner: RwLockWriteGuard<'a, GameObjectBox>,
}
impl<'a> Deref for GObjectWrite<'a> {
    type Target = dyn GameObject;
    fn deref(&self) -> &Self::Target {
        self.inner.ptr.as_ref().unwrap().as_ref()
    }
}

impl<'a> DerefMut for GObjectWrite<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.ptr.as_mut().unwrap().as_mut()
    }
}
impl<'a> GObjectRead<'a> {
    pub fn get_as<T: 'static>(&self) -> Option<&T> {
        let v = self.inner.ptr.as_ref().unwrap().as_ref() as &dyn Any;
        v.downcast_ref::<T>()
    }
}

impl<'a> GObjectWrite<'a> {
    pub fn get_as<T: 'static>(&self) -> Option<&T> {
        let v = self.inner.ptr.as_ref().unwrap().as_ref() as &dyn Any;
        v.downcast_ref::<T>()
    }
    pub fn get_as_mut<'b, T: 'static>(&'b mut self) -> Option<&'b mut T> {
        let v = self.inner.ptr.as_mut().unwrap().as_mut() as &mut dyn Any;
        v.downcast_mut::<T>()
    }
}
pub fn make_object(v: impl GameObject) -> GObject {
    for i in 0..ENGINE.objects.len() {
        let mut tmp = match ENGINE.objects[i].try_write() {
            Ok(x) => x,
            Err(y) => match y {
                std::sync::TryLockError::Poisoned(poison_error) => poison_error.into_inner(),
                std::sync::TryLockError::WouldBlock => {
                    continue;
                }
            },
        };
        if tmp.ptr.is_none() {
            tmp.generation = tmp.generation.wrapping_add(1);
            let out = GObject {
                idx: i as u32,
                generation: tmp.generation,
            };
            tmp.ptr = Some(Box::new(v));
            tmp.ptr.as_mut().unwrap().get_data_mut().self_id = out;
            return out;
        }
    }
    GObject::new()
}

pub fn delete_object(source: GObject, v: GObject) {
    ENGINE.events.lock().unwrap().push_back(Event {
        source,
        target: v,
        info: EventInfo::DestroyObject {},
    });
}

pub struct Object {
    pub data: GameObjectData,
}
impl GameObject for Object {
    fn get_data(&self) -> &GameObjectData {
        &self.data
    }
    fn get_data_mut(&mut self) -> &mut GameObjectData {
        &mut self.data
    }
    fn on_event(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, ev: Event) {
        _ = handle;
        _ = thread;
        _ = ev;
    }
    fn on_update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        _ = handle;
        _ = thread;
    }
}

pub fn run(setup: impl FnOnce(&mut RaylibHandle, &RaylibThread), game_mode: &mut dyn GameMode) {
    let (mut handle, thread) = raylib::RaylibBuilder::default().build();
    handle.set_target_fps(60);
    handle.disable_cursor();
    setup(&mut handle, &thread);
    while !handle.window_should_close() {
        step(&mut handle, &thread, game_mode);
    }
}

pub fn generate_cube(pos: Vector3, size: f32) -> GObject {
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cube(Vector3::zero(), size, size, size);

    make_object(Object {
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: Quaternion::identity(),
            width: size,
            depth: size,
            height: size,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: None,
            is_projectile: false,
            is_static: false,
            tags: Arc::new([]),
            self_id: GObject::new(),
            projectile_damage: 0,
            projectile_penetration: 0,
            allegience: -1,
        },
    })
}

pub fn update_physics() {
    let mut list = Vec::new();
    for i in 0..ENGINE.objects.len() {
        let t0 = ENGINE.objects[i].read().unwrap();
        let gn = t0.generation;
        if let Some(y) = t0.ptr.as_ref()
            && !y.get_data().is_projectile
        {
            list.push((i, gn, y.get_data().clone()));
        }
    }
    (0..ENGINE.objects.len()).into_par_iter().for_each(|i| {
        let mut t1 = ENGINE.objects[i].write().unwrap();
        let gn = t1.generation as u32;
        if let Some(y) = t1.ptr.as_mut() {
            let tmp = y.get_data_mut();
            if tmp.is_static {
                return;
            }
            let delta = tmp.velocity * 1. / 60.;
            let mut count = (delta.length() / 10.) as i32;
            if count < 1 {
                count = 1;
            }
            let dv = count as f32;
            let end = tmp.location + tmp.velocity / 60.;
            let end_rot = Vector4::from_euler(
                tmp.angular_velocity.x / 60.,
                tmp.angular_velocity.y / 60.,
                tmp.angular_velocity.z / 60.,
            ) * tmp.rotation;
            for _j in 0..count {
                let old = tmp.location;
                let old_rot = tmp.rotation;
                tmp.location += tmp.velocity * 1. / (60. * dv);
                tmp.rotation = Vector4::from_euler(
                    tmp.angular_velocity.x / (60. * dv),
                    tmp.angular_velocity.y / (60. * dv),
                    tmp.angular_velocity.z / (60. * dv),
                ) * tmp.rotation;
                if _j == count - 1 {
                    tmp.location = end;
                    tmp.rotation = end_rot;
                }
                let mut hit = false;
                let mut hv = Vector3::new(1.0, 0.0, 0.0);
                let mut hit_id = GObject::new();
                for (j, x, v0) in &list {
                    if *j == i {
                        continue;
                    }
                    if let Some(y) = tmp.check_collision(v0) {
                        hv = y;
                        hit = true;
                        hit_id = GObject {
                            idx: *j as u32,
                            generation: *x,
                        };
                        break;
                    }
                }
                if hit {
                    if tmp.is_projectile {
                        let source = GObject {
                            idx: i as u32,
                            generation: gn,
                        };
                        get_engine().events.lock().unwrap().push_back(Event {
                            source,
                            target: hit_id,
                            info: EventInfo::OnDamage {
                                aoe: false,
                                direction: tmp.velocity.normalized(),
                                damage_amount: tmp.projectile_damage,
                                penetration: tmp.projectile_penetration,
                            },
                        });
                        delete_object(hit_id, source);
                    }
                    tmp.angular_velocity *= -0.95;
                    tmp.velocity = tmp.velocity.reflect_from(hv) * 0.95;
                    tmp.location = old;
                    tmp.rotation = old_rot;
                    return;
                }
            }
        }
    });
}
impl GameObjectData {
    pub fn check_collision(&self, other: &Self) -> Option<Vector3> {
        let srad = if self.width > self.height {
            if self.width > self.depth {
                self.width
            } else {
                self.depth
            }
        } else {
            if self.height > self.depth {
                self.height
            } else {
                self.depth
            }
        };
        let orad = if other.width > other.height {
            if other.width > other.depth {
                other.width
            } else {
                other.depth
            }
        } else {
            if other.height > other.depth {
                other.height
            } else {
                other.depth
            }
        };
        let ds = srad + orad;
        if self.location.distance_to(other.location) >= ds {
            return None;
        }
        let mut v0 = [
            Vector3::new(-self.depth / 2., -self.width / 2., -self.height / 2.),
            Vector3::new(-self.depth / 2., -self.width / 2., self.height / 2.),
            Vector3::new(-self.depth / 2., self.width / 2., -self.height / 2.),
            Vector3::new(-self.depth / 2., self.width / 2., self.height / 2.),
            Vector3::new(self.depth / 2., -self.width / 2., -self.height / 2.),
            Vector3::new(self.depth / 2., -self.width / 2., self.height / 2.),
            Vector3::new(self.depth / 2., self.width / 2., -self.height / 2.),
            Vector3::new(self.depth / 2., self.width / 2., self.height / 2.),
        ];
        let mut v1 = [
            Vector3::new(-other.depth / 2., -other.width / 2., -other.height / 2.),
            Vector3::new(-other.depth / 2., -other.width / 2., other.height / 2.),
            Vector3::new(-other.depth / 2., other.width / 2., -other.height / 2.),
            Vector3::new(-other.depth / 2., other.width / 2., other.height / 2.),
            Vector3::new(other.depth / 2., -other.width / 2., -other.height / 2.),
            Vector3::new(other.depth / 2., -other.width / 2., other.height / 2.),
            Vector3::new(other.depth / 2., other.width / 2., -other.height / 2.),
            Vector3::new(other.depth / 2., other.width / 2., other.height / 2.),
        ];
        for i in &mut v0 {
            *i = i.rotate_by(self.rotation);
            *i += self.location;
        }
        for i in &mut v1 {
            *i = i.rotate_by(other.rotation);
            *i += other.location;
        }
        let normals = [
            Vector3::new(1.0, 0.0, 0.0).normalized(),
            Vector3::new(-1.0, 0.0, 0.0).normalized(),
            Vector3::new(0.0, 1.0, 0.0).normalized(),
            Vector3::new(0.0, -1.0, 0.0).normalized(),
            Vector3::new(0.0, 0.0, 1.0).normalized(),
            Vector3::new(0.0, 0.0, -1.0).normalized(),
            Vector3::new(0.0, 1.0, 1.0).normalized(),
            Vector3::new(0.0, 1.0, -1.0).normalized(),
            Vector3::new(0.0, -1.0, 1.0).normalized(),
            Vector3::new(0.0, -1.0, -1.0).normalized(),
            Vector3::new(1.0, 0.0, 1.0).normalized(),
            Vector3::new(1.0, 0.0, -1.0).normalized(),
            Vector3::new(-1.0, 0.0, 1.0).normalized(),
            Vector3::new(-1.0, 0.0, -1.0).normalized(),
            Vector3::new(1.0, 1.0, 0.0).normalized(),
            Vector3::new(-1.0, 1.0, 0.0).normalized(),
            Vector3::new(1.0, -1.0, 0.0).normalized(),
            Vector3::new(-1.0, -1.0, 0.0).normalized(),
            //
            Vector3::new(1.0, 1.0, 1.0).normalized(),
            Vector3::new(1.0, 1.0, -1.0).normalized(),
            Vector3::new(1.0, -1.0, 1.0).normalized(),
            Vector3::new(1.0, -1.0, -1.0).normalized(),
            Vector3::new(1.0, 1.0, 1.0).normalized(),
            Vector3::new(1.0, 1.0, -1.0).normalized(),
            Vector3::new(-1.0, 1.0, 1.0).normalized(),
            Vector3::new(-1.0, 1.0, -1.0).normalized(),
            Vector3::new(1.0, 1.0, 1.0).normalized(),
            Vector3::new(-1.0, 1.0, 1.0).normalized(),
            Vector3::new(1.0, -1.0, 1.0).normalized(),
            Vector3::new(-1.0, -1.0, 1.0).normalized(),
            //
            Vector3::new(-1.0, 1.0, 1.0).normalized(),
            Vector3::new(-1.0, 1.0, -1.0).normalized(),
            Vector3::new(-1.0, -1.0, 1.0).normalized(),
            Vector3::new(-1.0, -1.0, -1.0).normalized(),
            Vector3::new(1.0, -1.0, 1.0).normalized(),
            Vector3::new(1.0, -1.0, -1.0).normalized(),
            Vector3::new(-1.0, -1.0, 1.0).normalized(),
            Vector3::new(-1.0, -1.0, -1.0).normalized(),
            Vector3::new(1.0, 1.0, -1.0).normalized(),
            Vector3::new(-1.0, 1.0, -1.0).normalized(),
            Vector3::new(1.0, -1.0, -1.0).normalized(),
            Vector3::new(-1.0, -1.0, -1.0).normalized(),
        ];
        let mut n1 = normals;
        for i in &mut n1 {
            *i = i.rotate_by(self.rotation);
        }
        let mut n2 = normals;
        for i in &mut n2 {
            *i = i.rotate_by(other.rotation);
        }
        let mut min_delta = 5000000000.;
        let mut min_vec = n1[0];
        for i in n1 {
            let mut smin = v0[0].dot(i);
            let mut smax = v0[0].dot(i);
            for j in v0 {
                let tmp = j.dot(i);
                if tmp < smin {
                    smin = tmp
                }
                if tmp > smax {
                    smax = tmp;
                }
            }
            let mut omin = v1[0].dot(i);
            let mut omax = v1[0].dot(i);
            for j in v1 {
                let tmp = j.dot(i);
                if tmp < omin {
                    omin = tmp
                }
                if tmp > omax {
                    omax = tmp;
                }
            }
            if (omin - smax).abs() < min_delta {
                min_delta = (omin - smax).abs();
                min_vec = i;
            }
            if (smin - omax).abs() < min_delta {
                min_delta = (omin - smax).abs();
                min_vec = i;
            }
            if smax < omin || omax < smin {
                return None;
            }
        }
        for i in n2 {
            let mut smin = v0[0].dot(i);
            let mut smax = v0[0].dot(i);
            for j in v0 {
                let tmp = j.dot(i);
                if tmp < smin {
                    smin = tmp
                }
                if tmp > smax {
                    smax = tmp;
                }
            }
            let mut omin = v1[0].dot(i);
            let mut omax = v1[0].dot(i);
            for j in v1 {
                let tmp = j.dot(i);
                if tmp < omin {
                    omin = tmp
                }
                if tmp > omax {
                    omax = tmp;
                }
            }
            if (omin - smax).abs() < min_delta {
                min_delta = (omin - smax).abs();
                min_vec = i;
            }
            if (smin - omax).abs() < min_delta {
                min_delta = (omin - smax).abs();
                min_vec = i;
            }
            if smax < omin || omax < smin {
                return None;
            }
        }
        //println!("{}, {:#?}", min_delta, min_vec);
        Some(min_vec)
    }

    pub fn raycast_against(&self, start: Vector3, end: Vector3) -> Option<HitInfo> {
        let bb = BoundingBox::new(
            Vector3::new(-self.depth / 2., -self.width / 2., -self.height / 2.),
            Vector3::new(self.depth / 2., self.width / 2., self.height / 2.),
        );
        let s0 = start - self.location;
        let e0 = end - self.location;
        let mat = self.rotation.inverted();
        let s1 = s0.rotate_by(mat);
        let e1 = e0.rotate_by(mat);
        let col = bb.get_ray_collision_box(Ray {
            position: s1,
            direction: (e1 - s1).normalized(),
        });
        if col.hit {
            let max_dist = e1.distance_to(s1);
            if col.distance > max_dist {
                return None;
            }
            let out = HitInfo {
                hit_object: GObject::new(),
                hit_location: col.point.rotate_by(self.rotation) + self.location,
                start,
                distance: col.distance,
                normal: col.normal.rotate_by(self.rotation),
            };
            return Some(out);
        }
        None
    }
}

pub fn random_vector() -> Vector3 {
    let resolution = 100;
    let r2 = resolution as f32 / 2.;
    let x = ((random::<u64>() % resolution) as f32 - r2) / r2;
    let y = ((random::<u64>() % resolution) as f32 - r2) / r2;
    let z = ((random::<u64>() % resolution) as f32 - r2) / r2;
    Vector3::new(x, y, z)
}
pub fn random_unit_vector() -> Vector3 {
    let resolution = 100;
    let r2 = resolution as f32 / 2.;
    let theta = ((random::<u64>() % resolution) as f32 - r2) / r2 * 2. * PI;
    let phi = ((random::<u64>() % resolution) as f32 - r2) / r2 * 2. * PI;
    let x = phi.cos() * theta.sin();
    let y = phi.sin() * theta.sin();
    let z = theta.cos();
    Vector3::new(x, y, z)
}

pub fn set_player(id: GObject) {
    *ENGINE.player_object.lock().unwrap() = id;
}

pub fn clear_player() {
    *ENGINE.player_object.lock().unwrap() = GObject::new();
}

pub struct TypeRegistery {
    pub types: Mutex<BTreeMap<String, fn(&GameObjectSerialized) -> Box<dyn GameObject>>>,
}

pub static TYPE_REGISTERY: TypeRegistery = TypeRegistery {
    types: Mutex::new(BTreeMap::new()),
};

pub fn register_type<T: Serialize + DeserializeOwned + GameObject>() {
    let name = type_name::<T>().to_string();
    fn deserialize_func<T: Serialize + DeserializeOwned + GameObject>(
        obj: &GameObjectSerialized,
    ) -> Box<dyn GameObject> {
        assert!(obj.type_name == type_name::<T>());
        let out: T = serde_json::from_str(&obj.data).unwrap();
        Box::new(out)
    }
    TYPE_REGISTERY
        .types
        .lock()
        .unwrap()
        .insert(name, deserialize_func::<T>);
}

pub fn generate_ufo(pos: Vector3, size: f32) -> GObject {
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

    make_object(Object {
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: Quaternion::identity(),
            width: 2. * size,
            depth: 2. * size,
            height: 0.5 * size,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: None,
            is_projectile: false,
            is_static: false,
            tags: Arc::new([]),
            self_id: GObject::new(),
            projectile_damage: 0,
            projectile_penetration: 0,
            allegience: -1,
        },
    })
}

pub trait GameMode {
    fn on_update(&mut self, _handle: &mut RaylibHandle, _thread: &RaylibThread) {}
    fn on_init(&mut self, _handle: &mut RaylibHandle, _thread: &RaylibThread) {}
    fn on_render(&mut self, _handle: &mut RaylibDrawHandle, _thread: &RaylibThread) {}
}

pub struct DefaultGameMode {}

impl GameMode for DefaultGameMode {}

pub fn raycast(
    start: Vector3,
    direction: Vector3,
    max_range: f32,
    ignored_entities: &[GObject],
    include_projectiles: bool,
) -> Option<HitInfo> {
    let mut hs = HitInfo {
        hit_location: Vector3::zero(),
        start,
        distance: 0.0,
        normal: Vector3::zero(),
        hit_object: GObject::new(),
    };
    let mut has_hit = false;
    let mut min_dist = 100000000.0;
    let end = start + direction * max_range;
    for i in 0..ENGINE.objects.len() {
        let lck = match ENGINE.objects[i].try_read() {
            Ok(x) => x,
            Err(e) => match e {
                std::sync::TryLockError::Poisoned(x) => x.into_inner(),
                std::sync::TryLockError::WouldBlock => {
                    continue;
                }
            },
        };
        let g = GObject {
            idx: i as u32,
            generation: lck.generation,
        };

        if ignored_entities.contains(&g) {
            continue;
        }
        if let Some(t) = lck.ptr.as_ref() {
            if !include_projectiles && t.get_data().is_projectile {
                continue;
            }
            if let Some(hit) = t.get_data().raycast_against(start, end)
                && hit.distance < min_dist
            {
                min_dist = hit.distance;
                hs = hit;
                hs.hit_object = g;
                has_hit = true;
            }
        }
    }
    if has_hit { Some(hs) } else { None }
}

pub fn get_engine() -> &'static Engine {
    &ENGINE
}

pub fn get_all_objects_with_tag(tag: &str) -> Vec<GObject> {
    let mut out = Vec::new();
    for i in 0..get_engine().objects.len() {
        let tmp = match get_engine().objects[i].try_read() {
            Ok(x) => x,
            Err(e) => match e {
                std::sync::TryLockError::Poisoned(x) => x.into_inner(),
                std::sync::TryLockError::WouldBlock => {
                    continue;
                }
            },
        };
        if let Some(t) = tmp.ptr.as_ref() {
            for j in t.get_data().tags.as_ref() {
                if *tag == **j {
                    out.push(GObject {
                        idx: i as u32,
                        generation: tmp.generation as u32,
                    });
                    break;
                }
            }
        }
    }
    out
}

pub fn get_all_objects() -> Vec<GObject> {
    let mut out = Vec::new();
    for i in 0..get_engine().objects.len() {
        let tmp = match get_engine().objects[i].try_read() {
            Ok(x) => x,
            Err(e) => match e {
                std::sync::TryLockError::Poisoned(x) => x.into_inner(),
                std::sync::TryLockError::WouldBlock => {
                    continue;
                }
            },
        };
        if tmp.ptr.as_ref().is_some() {
            out.push(GObject {
                idx: i as u32,
                generation: tmp.generation as u32,
            });
        }
    }
    out
}

pub fn get_all_objects_with_tags(tags: &[&str]) -> Vec<GObject> {
    let mut out = Vec::new();
    for i in 0..get_engine().objects.len() {
        let tmp = match get_engine().objects[i].try_read() {
            Ok(x) => x,
            Err(e) => match e {
                std::sync::TryLockError::Poisoned(x) => x.into_inner(),
                std::sync::TryLockError::WouldBlock => {
                    continue;
                }
            },
        };
        if let Some(t) = tmp.ptr.as_ref() {
            for k in tags {
                let mut hit = false;
                for j in t.get_data().tags.as_ref() {
                    if **k == **j {
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    continue;
                }
            }
            out.push(GObject {
                idx: i as u32,
                generation: tmp.generation,
            });
        }
    }
    out
}
