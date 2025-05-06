use crate::{
    game::{
        handle_player, run_game_systems, ship::{FuelComp, HealthComp, InventoryComp, ShipComp}, PlayerData
    },
    math::{Quaternion, Transform, Vector3},
    physics::{
        self, add_physics_comp, get_physics_comp, get_physics_mut, remove_physics_comp, Collision
    },
    renderer::{self, add_model_comp, get_model_comp, get_model_mut, remove_model_comp}, ui,
};
use raylib::{
    RaylibHandle, RaylibThread, camera::Camera3D, color, ffi::TraceLogLevel, models::RaylibMesh,
    prelude::RaylibDraw,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut, Index},
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
static LEVEL_SHOULD_CONTINUE: Mutex<bool> = Mutex::new(true);
static GAME_SHOULD_CONTINUE: Mutex<bool> = Mutex::new(true);
static DESTROY_QUEUE: Mutex<Vec<Entity>> = Mutex::new(Vec::new());
static ENTITY_LIST_MODIFIED: Mutex<bool> = Mutex::new(true);
use serde::{Deserialize, Serialize};
pub static mut LEVEL: Option<Level> = None;

pub unsafe fn level_check_entity(ent: Entity) -> bool {
    get_level().check_entity_ref(ent)
}
use crate::{
    physics::PhysicsComp,
    renderer::{ModelComp, ModelList},
};
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Instant {
    pub trans: Transform,
    pub is_valid: bool,
}
impl Instant {
    pub const fn new() -> Self {
        Self {
            trans: Transform {
                translation: Vector3::zero(),
                scale: Vector3::new(1., 1., 1.),
                rotation: Quaternion::new(0., 0., 0., 1.),
            },
            is_valid: false,
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct TransformComp {
    pub trans: Transform,
    pub previous: Box<[Instant]>,
}
impl TransformComp {
    pub fn update(&mut self) {
        let mut idx = 0;
        for i in &self.previous {
            if i.is_valid {
                idx += 1;
            } else {
                break;
            }
        }
        if idx >= self.previous.len() {
            for i in 0..self.previous.len() - 1 {
                self.previous[i] = self.previous[i + 1].clone();
            }
            idx = self.previous.len() - 1;
        }
        self.previous[idx] = Instant {
            trans: self.trans,
            is_valid: true,
        };
    }
    pub fn new() -> Self {
        let count = 120;
        let mut out = Vec::new();
        out.reserve_exact(count);
        for _ in 0..count {
            out.push(Instant::new());
        }
        Self {
            trans: Transform::default(),
            previous: out.into_boxed_slice(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParentComp {
    pub parent: Option<Entity>,
}
crate::gen_comp_functions!(
    ParentComp,
    parent_comps,
    add_parent_comp,
    remove_parent_comp,
    get_parent_comp,
    get_parent_mut
);
#[derive(Clone, Serialize, Deserialize)]
pub struct ChildrenComp {
    pub children: Vec<Entity>,
}
crate::gen_comp_functions!(
    ChildrenComp,
    children_comps,
    add_children_comp,
    remove_children_comp,
    get_children_comp,
    get_children_mut
);
crate::gen_comp_functions!(
    TransformComp,
    transform_comps,
    add_transform_comp,
    remove_transform_comp,
    get_transform_comp,
    get_transform_mut
);
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Entity {
    pub idx: u32,
    pub generation: u32,
}
pub struct CompRef<T: 'static> {
    pub lock: RwLockReadGuard<'static, Box<[Option<T>]>>,
    pub idx: usize,
}
impl<T: 'static> Deref for CompRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock[self.idx].as_ref().unwrap()
    }
}
impl<T: 'static> Drop for CompRef<T> {
    fn drop(&mut self) {
        // println!("dropped_ref:{} {}", self.idx,type_name::<T>() );
    }
}

pub struct CompMut<T: 'static> {
    pub lock: RwLockWriteGuard<'static, Box<[Option<T>]>>,
    pub idx: usize,
}
impl<T: 'static> Deref for CompMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock[self.idx].as_ref().unwrap()
    }
}
impl<T: 'static> DerefMut for CompMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock[self.idx].as_mut().unwrap()
    }
}
impl<T: 'static> Drop for CompMut<T> {
    fn drop(&mut self) {
        // println!("dropped_mut:{} {}", self.idx,type_name::<T>() );
    }
}
#[derive(Serialize, Deserialize)]
pub struct ComponentList<T: 'static + Serialize + Send + Sync + for<'a> Deserialize<'a> + Clone> {
    #[serde(with = "RwLock")]
    pub list: RwLock<Box<[Option<T>]>>,
}

impl<T: 'static + Serialize + Send + Sync + for<'a> Deserialize<'a> + Clone> ComponentList<T> {
    pub fn init(size: usize) -> Self {
        let mut list = Vec::new();
        list.reserve_exact(size);
        for _ in 0..size {
            list.push(None)
        }
        Self {
            list: RwLock::new(list.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub frame_time: f64,
    pub loaded_models: Vec<String>,
    pub existing_entities: RwLock<Box<[bool]>>,
    #[serde(with = "RwLock")]
    pub component_indexes: RwLock<Box<[u32]>>,
    pub physics_comps: ComponentList<PhysicsComp>,
    pub transform_comps: ComponentList<TransformComp>,
    pub model_comps: ComponentList<ModelComp>,
    pub player_entity: Entity,
    pub health_comps: ComponentList<HealthComp>,
    pub fuel_comps: ComponentList<FuelComp>,
    pub inventory_comps: ComponentList<InventoryComp>,
    pub ship_comps: ComponentList<ShipComp>,
    pub children_comps: ComponentList<ChildrenComp>,
    pub parent_comps: ComponentList<ParentComp>,
    pub tag_comps: ComponentList<TagComp>,
}
impl Level {
    pub fn check_entity_ref(&self, ent: Entity) -> bool {
        let p = self.component_indexes.read().unwrap();
        if p.len() <= ent.idx as usize {
            return false;
        }
        return p[ent.idx as usize] == ent.generation;
    }
    pub fn new(ent_size: usize) -> Self {
        let mut comp_idexs = Vec::new();
        let mut existing_entities = Vec::new();
        existing_entities.reserve_exact(ent_size);
        comp_idexs.reserve_exact(ent_size);
        for _ in 0..ent_size {
            comp_idexs.push(0);
            existing_entities.push(false);
        }
        Self {
            loaded_models: Vec::new(),
            existing_entities: RwLock::new(existing_entities.into()),
            component_indexes: RwLock::new(comp_idexs.into()),
            physics_comps: ComponentList::init(ent_size),
            transform_comps: ComponentList::init(ent_size),
            model_comps: ComponentList::init(ent_size),
            frame_time: 1. / 60.,
            player_entity: Entity {
                idx: ent_size as u32 + 1,
                generation: ent_size as u32 + 1,
            },
            health_comps: ComponentList::init(ent_size),
            fuel_comps: ComponentList::init(ent_size),
            inventory_comps: ComponentList::init(ent_size),
            ship_comps: ComponentList::init(ent_size),
            parent_comps: ComponentList::init(ent_size),
            children_comps: ComponentList::init(ent_size),
            tag_comps: ComponentList::init(ent_size),
        }
    }
}
pub fn get_level() -> &'static Level {
    unsafe {
        let t = &raw const LEVEL;
        t.as_ref().unwrap().as_ref().unwrap()
    }
}

#[macro_export]
macro_rules! gen_comp_functions {
    ($t:ty, $var_name:ident,$adder:ident, $remover:ident, $getter:ident, $getter_mut:ident) => {
        pub fn $adder(ent: crate::level::Entity, value: $t) {
            unsafe {
                assert!(crate::level::level_check_entity(ent));
                let mut lock = crate::level::get_level().$var_name.list.write().unwrap();
                lock[ent.idx as usize] = Some(value);
            }
        }
        pub fn $remover(ent: crate::level::Entity) {
            unsafe {
                assert!(crate::level::level_check_entity(ent));
                let mut lock = crate::level::get_level().$var_name.list.write().unwrap();
                lock[ent.idx as usize] = None;
            }
        }
        pub fn $getter(ent: crate::level::Entity) -> Option<crate::level::CompRef<$t>> {
            //  println!("got:{} {}", ent.idx, std::any::type_name::<$t>());
            unsafe {
                if !crate::level::level_check_entity(ent) {
                    return None;
                }
                let lock = crate::level::get_level().$var_name.list.read().unwrap();
                if lock[ent.idx as usize].is_some() {
                    Some(crate::level::CompRef {
                        lock,
                        idx: ent.idx as usize,
                    })
                } else {
                    None
                }
            }
        }
        pub fn $getter_mut(ent: crate::level::Entity) -> Option<crate::level::CompMut<$t>> {
            unsafe {
                //     println!("got_mut:{} {}", ent.idx, std::any::type_name::<$t>());
                if !crate::level::level_check_entity(ent) {
                    return None;
                }
                let lock = crate::level::get_level().$var_name.list.write().unwrap();
                if lock.len() <= ent.idx as usize {
                    return None;
                }
                if lock[ent.idx as usize].is_some() {
                    Some(crate::level::CompMut {
                        lock,
                        idx: ent.idx as usize,
                    })
                } else {
                    None
                }
            }
        }
    };
}
pub fn create_entity() -> Option<Entity> {
    let lv = get_level();
    let mut existing = lv.existing_entities.write().unwrap();
    let counts = lv.component_indexes.write().unwrap();
    for i in 0..existing.len() {
        if !existing[i] {
            existing[i] = true;
            *ENTITY_LIST_MODIFIED.lock().unwrap() = true;
            let out = Entity {
                idx: i as u32,
                generation: counts[i] as u32,
            };
            let mut lock = lv.tag_comps.list.write().unwrap();
            lock[i] = Some(TagComp { tags: Vec::new() });
            return Some(out);
        }
    }
    None
}

pub fn destroy_entity_actual(ent: Entity) {
    unsafe {
        if !level_check_entity(ent) {
            return;
        }
        let lv = get_level();
        if let Some(children) = get_children_comp(ent) {
            for i in &children.children {
                destroy_entity(*i);
            }
        }
        let mut existing = lv.existing_entities.try_write().unwrap();
        existing[ent.idx as usize] = false;
        drop(existing);
        let mut counts = lv.component_indexes.try_write().unwrap();
        counts[ent.idx as usize] += 1;
        lv.children_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.fuel_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.health_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.inventory_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.model_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.parent_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.physics_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.ship_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.transform_comps.list.write().unwrap()[ent.idx as usize] = None;
        lv.tag_comps.list.write().unwrap()[ent.idx as usize] = None;
    }
}
pub fn destroy_entity(ent: Entity) {
    DESTROY_QUEUE.lock().unwrap().push(ent);
}
pub fn save_level(file_name: &str) {
    let bytes = serde_json::to_string_pretty(get_level()).unwrap();
    std::fs::write(file_name, bytes).unwrap();
}
pub fn load_level(file_name: &str) -> Level {
    let level: Level = serde_json::from_slice(&std::fs::read(file_name).unwrap()).unwrap();
    level
}
pub fn init_level(entity_count: usize) {
    let level = Level::new(entity_count);
    unsafe {
        LEVEL = Some(level);
    }
}
pub fn default_setup(
    thread: &RaylibThread,
    handle: &mut RaylibHandle,
    entity_count: usize,
) -> ModelList {
    let mut model_list = ModelList {
        list: HashMap::new(),
    };
    let sz = 0.1;
    let ms = raylib::models::Mesh::gen_mesh_cube(thread, sz, sz, sz);
    let box_mesh = handle
        .load_model_from_mesh(thread, unsafe { ms.make_weak() })
        .unwrap();
    model_list.list.insert("box".into(), box_mesh);
    let msh = raylib::models::Mesh::gen_mesh_sphere(thread, sz / 2., 32, 13);
    let sphere_mesh = handle
        .load_model_from_mesh(thread, unsafe { msh.make_weak() })
        .unwrap();
    model_list.list.insert("sphere".into(), sphere_mesh);
    init_level(entity_count);
    let trs = raylib::models::Mesh::gen_mesh_torus(thread, sz / 2., sz * 2., 32, 32);
    let torus_mesh = handle
        .load_model_from_mesh(thread, unsafe { trs.make_weak() })
        .unwrap();
    model_list.list.insert("torus".into(), torus_mesh);
    let cyl = raylib::models::Mesh::gen_mesh_cylinder(thread, sz / 4., sz, 32);
    let cl_mesh = handle
        .load_model_from_mesh(thread, unsafe { cyl.make_weak() })
        .unwrap();
    model_list.list.insert("cylinder".into(), cl_mesh);
    model_list
}
pub fn get_frame_time() -> f64 {
    get_level().frame_time
}
fn run_destructions() {
    loop {
        let queu = DESTROY_QUEUE.lock().unwrap().clone();
        DESTROY_QUEUE.lock().unwrap().clear();
        if queu.is_empty() {
            break;
        }
        for i in queu {
            destroy_entity_actual(i);
        }
    }
}
static LEVEL_TO_LOAD: Mutex<
    Option<
        Box<dyn Fn(&raylib::RaylibThread, &mut raylib::RaylibHandle) -> ModelList + Send + Sync>,
    >,
> = Mutex::new(None);

pub fn level_loop(thread: &raylib::RaylibThread, handle: &mut raylib::RaylibHandle) {
    let mut model_list = LEVEL_TO_LOAD.lock().unwrap().as_ref().unwrap()(thread, handle);
    let mut cam = Camera3D::perspective(
        crate::math::Vector3::new(-10.0, 0., 0.0).as_rl_vec(),
        Vector3::new(1.0, 0., 0.).as_rl_vec(),
        crate::math::Vector3::new(0.0, 0.0, 1.0).as_rl_vec(),
        90.0,
    );
    let mut ui = ui::UI::new(0, 0,1000,1600);
    let mut player_data = PlayerData { camera: cam };
    loop {
        let font = handle.get_font_default();
        let should_continue = LEVEL_SHOULD_CONTINUE.lock().unwrap();
        if !*should_continue {
            break;
        }
        drop(should_continue);
        if handle.window_should_close() {
            *GAME_SHOULD_CONTINUE.lock().unwrap() = false;
            break;
        }
        let dt = handle.get_frame_time() as f64;
        run_game_systems(&mut player_data, thread, handle,dt,&mut ui);
        *physics::SAFE_TO_TAKE.lock().unwrap() = false;
        let j = std::thread::spawn(move || physics::update(dt));
        //physics::update(dt);
        let mut draw = handle.begin_drawing(thread);
        draw.clear_background(color::Color::new(0, 0, 20, 255));
        renderer::render(thread, &mut draw, &mut model_list, &mut cam, &font);
        let _ = j.join();
        run_destructions();
    }
    unsafe {
        crate::level::LEVEL = None;
    }
}
pub fn main_loop(
    level_to_load: Box<
        dyn Fn(&raylib::RaylibThread, &mut raylib::RaylibHandle) -> ModelList
            + Send
            + Sync
            + 'static,
    >,
) {
    *LEVEL_TO_LOAD.lock().unwrap() = Some(level_to_load);
    let (mut handle, thread) = raylib::init()
        .title("hello window")
        .size(1600, 1000)
        .msaa_4x()
        .log_level(TraceLogLevel::LOG_ERROR)
        .build();
    handle.set_target_fps(60);
   // handle.disable_cursor();
    loop {
        let should_continue = GAME_SHOULD_CONTINUE.lock().unwrap();
        if !*should_continue {
            break;
        }
        drop(should_continue);
        level_loop(&thread, &mut handle);
    }
}
pub fn add_child_entity(parent: Entity, child: Entity) {
    assert!(parent != child);
}
pub fn remove_child_entity(parent: Entity, child: Entity) {
    if let Some(mut children) = get_children_mut(parent) {
        let mut idx = 0;
        let mut hit = false;
        for i in 0..children.children.len() {
            if children.children[i] == child {
                idx = i;
                hit = true;
                break;
            }
        }
        if hit {
            children.children.remove(idx);
            remove_parent_comp(child);
        }
    }
}

pub fn child_add_model(parent: Entity, child: Entity, comp: ModelComp) {
    let mut msh = get_model_mut(parent).unwrap();
    for mut i in comp.models {
        i.parent = Some(child);
        i.offset = if let Some(t) = get_transform_mut(child) {
            let mut s = i.offset;
            s.rotation *= t.trans.rotation;
            s.translation += t.trans.translation;
            s
        } else {
            i.offset
        };
        msh.models.push(i);
    }
}
pub fn child_add_physics(parent: Entity, child: Entity, mut comp: Collision) {
    let mut phys = get_physics_mut(parent).unwrap();
    for i in &mut phys.collisions {
        if let Some(k) = i.entity_ref {
            if k == child {
                i.col = comp.col;
                i.offset = comp.offset;
                return;
            }
        }
    }
    comp.entity_ref = Some(child);
    phys.collisions.push(comp);
}
pub fn child_remove_model(parent: Entity, child: Entity) {
    if let Some(mut models) = get_model_mut(parent) {
        loop {
            let mut found = false;
            let mut idx = 0;
            for i in 0..models.models.len() {
                if let Some(id) = models.models[i].parent {
                    if id == child {
                        found = true;
                        idx = i;
                    }
                }
            }
            if found {
                models.models.remove(idx);
            } else {
                break;
            }
        }
    }
}
pub fn child_remove_physics(parent: Entity, child: Entity) {
    if let Some(mut phys) = get_physics_mut(parent) {
        loop {
            let mut found = false;
            let mut idx = 0;
            for i in 0..phys.collisions.len() {
                if let Some(id) = phys.collisions[i].entity_ref {
                    if id == child {
                        found = true;
                        idx = i;
                    }
                }
            }
            if found {
                phys.collisions.remove(idx);
            } else {
                break;
            }
        }
    }
}

pub fn add_child_object(parent: Entity, child: Entity) {
    if get_children_comp(parent).is_none() {
        add_children_comp(
            parent,
            ChildrenComp {
                children: Vec::new(),
            },
        );
    }
    let mut ccmp = get_children_mut(parent).unwrap();
    if ccmp.children.contains(&child) {
        return;
    }
    ccmp.children.push(child);
    let ctrans = if let Some(trans) = get_transform_comp(child) {
        trans.trans
    } else {
        Transform::default()
    };
    if let Some(mshs) = get_model_comp(child).map(|i| i.clone()) {
        if mshs.models.len() != 0 {
            let cmp = get_model_comp(parent);
            if cmp.is_none() {
                drop(cmp);
                add_model_comp(parent, ModelComp::empty());
            } else {
                drop(cmp);
            }
            let mut models = get_model_mut(parent).unwrap();
            for i in &mshs.models {
                let mut md = i.clone();
                md.offset.translation += ctrans.translation;
                md.offset.rotation *= ctrans.rotation;
                md.parent = Some(child);
                models.models.push(md);
            }
        }
        remove_model_comp(child);
    }
    if let Some(phys) = get_physics_comp(child).map(|i| i.clone()) {
        if phys.collisions.len() != 0 {
            let cmp = get_physics_comp(parent);
            if cmp.is_none() {
                drop(cmp);
                add_physics_comp(parent, PhysicsComp::new());
            } else {
                drop(cmp);
            }
            let mut comps = get_physics_mut(parent).unwrap();
            for i in &phys.collisions {
                let mut ps = i.clone();
                ps.offset.translation += ctrans.translation;
                ps.offset.rotation += ctrans.rotation;
                ps.entity_ref = Some(child);
                comps.collisions.push(ps);
            }
        }
        remove_physics_comp(child);
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagComp {
    pub tags: Vec<String>,
}
crate::gen_comp_functions!(
    TagComp,
    tag_comps,
    add_tag_comp,
    remove_tag_comp,
    get_tag_comp,
    get_tag_mut
);
impl TagComp {
    pub fn matches_single(&self, v: &str) -> bool {
        for i in &self.tags {
            if i == v {
                return true;
            }
        }
        false
    }
    pub fn matches_set<T: AsRef<str>>(&self, set: &[T]) -> bool {
        for i in set {
            if !self.matches_single(i.as_ref()) {
                return false;
            }
        }
        true
    }
}
pub fn get_entities() -> Vec<Entity> {
    let mut out = Vec::new();
    let lv = get_level();
    let ents = lv.existing_entities.read().unwrap();
    let generations = lv.component_indexes.read().unwrap();
    for i in 0..ents.len() {
        if ents[i] {
            let et = Entity {
                idx: i as u32,
                generation: generations[i],
            };
            out.push(et);
        }
    }
    return out;
}
pub fn entities_with_tagset<T: AsRef<str>>(set: &[T]) -> Vec<Entity> {
    let mut out = Vec::new();
    let entities = get_entities();
    for i in entities {
        if let Some(tg) = get_tag_comp(i) {
            if tg.matches_set(set) {
                out.push(i)
            }
        }
    }
    out
}
pub fn entities_with_tag(tag: &str) -> Vec<Entity> {
    let mut out = Vec::new();
    let entities = get_entities();
    for i in entities {
        if let Some(tg) = get_tag_comp(i) {
            if tg.matches_single(tag) {
                out.push(i);
            }
        }
    }
    out
}
pub fn set_tags(entity: Entity, tags: Vec<String>) {
    if let Some(mut et) = get_tag_mut(entity) {
        et.tags = tags;
    }
}
pub fn add_tag(entity: Entity, tag: &str) {
    if let Some(mut et) = get_tag_mut(entity) {
        if !et.matches_single(tag) {
            et.tags.push(tag.to_string());
        }
    }
}
pub fn add_tags(entity: Entity, tags: Vec<String>) {
    for i in tags {
        add_tag(entity, i.as_ref());
    }
}
