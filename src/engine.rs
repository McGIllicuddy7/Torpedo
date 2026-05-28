use std::{
    any::type_name,
    collections::{BTreeMap, HashMap, VecDeque},
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use rand::random;
use raylib::{
    RaylibHandle, RaylibThread,
    camera::Camera3D,
    color::Color,
    drawing::{RaylibDraw, RaylibDraw3D, RaylibMode3DExt},
    math::{BoundingBox, Matrix, Quaternion, Vector3, Vector4},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeOwned},
};

use crate::mesh::GameMesh;
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum EventKind {
    OnDamage,
    OnDestroy,
    DestroyObject,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventInfo {
    OnDamage {},
    OnDestroy {},
    DestroyObject {},
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
}

pub trait GameObject: Send + Sync + 'static {
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

pub enum DrawEvent {}
pub enum DrawEvent3D {}
pub static ENGINE: Engine = Engine {
    objects: [const {
        RwLock::new(GameObjectBox {
            ptr: None,
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

pub fn step(handle: &mut RaylibHandle, thread: &RaylibThread) {
    for i in &ENGINE.objects {
        if let Some(obj) = i.try_write().unwrap().ptr.as_mut() {
            obj.on_update(handle, thread);
        }
    }
    while let Some(ev) = ENGINE.events.try_lock().unwrap().pop_front() {
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
                if tmp.generation == ev.target.generation {
                    if let Some(t) = tmp.ptr.as_mut() {
                        t.on_event(handle, thread, ev);
                    }
                }
            }
        }
    }
    update_physics();
    let mut c = ENGINE.camera_data.lock().unwrap();
    let mut cm = Camera3D::perspective(c.position, c.target, c.up, 90.);
    handle.update_camera(&mut cm, raylib::ffi::CameraMode::CAMERA_FREE);
    c.position = cm.position;
    c.target = cm.target;
    c.up = cm.up;
    if false {
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
    drop(draw3d);
    draw.draw_fps(1600, 20);
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
            EventInfo::OnDamage {} => EventKind::OnDamage,
            EventInfo::OnDestroy {} => EventKind::OnDestroy,
            EventInfo::DestroyObject {} => EventKind::OnDestroy,
        }
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
            if let Some(t) = r.ptr.as_ref() {
                Some(func(t.as_ref()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn with_mut<W>(&self, func: impl FnOnce(&mut dyn GameObject) -> W) -> Option<W> {
        let mut r = ENGINE.objects[self.idx as usize].write().unwrap();
        if r.generation == self.generation {
            if let Some(t) = r.ptr.as_mut() {
                Some(func(t.as_mut()))
            } else {
                None
            }
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
            return out;
        }
    }
    GObject::new()
}

pub fn delete_object(source: GObject, v: GObject) {
    ENGINE.events.lock().unwrap().push_back(Event {
        source: source,
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

pub fn run(setup: impl FnOnce(&mut RaylibHandle, &RaylibThread)) {
    let (mut handle, thread) = raylib::RaylibBuilder::default().build();
    handle.set_target_fps(60);
    setup(&mut handle, &thread);
    while !handle.window_should_close() {
        step(&mut handle, &thread);
    }
}

pub fn generate_cube(pos: Vector3, size: f32) -> GObject {
    let mut msh = GameMesh {
        points: Vec::new(),
        lines: Vec::new(),
    };
    msh.add_cube(Vector3::zero(), size, size, size);
    let v = make_object(Object {
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
        },
    });
    v
}

pub fn update_physics() {
    let mut list = Vec::new();
    for i in 0..ENGINE.objects.len() {
        let t0 = ENGINE.objects[i].read().unwrap();
        if let Some(y) = t0.ptr.as_ref() {
            if !y.get_data().is_projectile {
                list.push((i, y.get_data().clone()));
            }
        }
    }
    (0..ENGINE.objects.len()).into_par_iter().for_each(|i| {
        let mut t1 = ENGINE.objects[i].write().unwrap();
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
                for (j, v0) in &list {
                    if *j == i {
                        continue;
                    }
                    if let Some(y) = tmp.check_collision(v0) {
                        hv = y;
                        hit = true;
                        break;
                    }
                }
                if hit {
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
}
pub fn random_vector() -> Vector3 {
    let resolution = 100;
    let r2 = resolution as f32 / 2.;
    let x = ((random::<u64>() % resolution) as f32 - r2) / r2;
    let y = ((random::<u64>() % resolution) as f32 - r2) / r2;
    let z = ((random::<u64>() % resolution) as f32 - r2) / r2;
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
        1.,
        0.8,
        0.5,
        6,
        Vector3::new(0., 0., 0.25),
        Vector3::new(0.0, 0.0, 1.),
        Vector3::new(1., 0.0, 0.0),
    );
    let v = make_object(Object {
        data: GameObjectData {
            model: Some(msh),
            location: pos,
            rotation: Quaternion::identity(),
            width: 1.,
            depth: 1.,
            height: 0.4,
            velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            camera_data: None,
            is_projectile: false,
            is_static: false,
        },
    });
    v
}
