
use std::{any::{type_name, Any}, collections::HashMap, error::Error, ffi::c_void, fmt::{Debug, Display}, ops::{Deref, DerefMut}, sync::{ LazyLock, Mutex}};

use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};


pub trait DynSer{
    fn name(&self)->String{
        std::any::type_name::<Self>().to_owned()
    }
    fn serialize_to_bytes(&self)->Vec<u8>;
}

pub struct Obj<T:?Sized +'static>{
    pub v:Box<T>,
}
impl<T:?Sized+'static> From<Box<T>> for Obj<T>{
    fn from(value: Box<T>) -> Self {
        Self { v: value }
    }
}
impl<T:?Sized+'static> Deref for Obj<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.v.as_ref()
    }
}
impl<T:?Sized+'static> DerefMut for Obj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.v.as_mut()
    }
}
impl<T:?Sized+'static> AsRef<T> for Obj<T>{
    fn as_ref(&self) -> &T {
        self.v.as_ref()
    }
}
impl<T:?Sized+'static> AsMut<T> for Obj<T> {
    fn as_mut(&mut self) -> &mut T {
        self.v.as_mut()
    }
}
impl <T:?Sized+'static+Clone> Clone for Obj<T>{
    fn clone(&self) -> Self {
        Self { v: self.v.clone() }
    }
}
impl <T:'static> Obj<T> {
    pub fn new(v:T)->Self{
        Self { v: Box::new(v) }
    }

}
impl <T:'static+?Sized> Obj<T>{
    pub fn from_box(v:Box<T>)->Self{
        Self { v }
    }
}
impl <T:Debug+'static> Debug for Obj<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Obj").field("v", &self.v).finish()
    }
} 
impl <T:Display+'static> Display for Obj<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}
#[derive(Serialize, Deserialize)]
struct SerializeablObj{
    pub name:String, 
    pub data:Vec<u8>,
}
impl<T:?Sized+'static+DynSer> Serialize for Obj<T>{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let obj = SerializeablObj{name:self.v.name(), data:self.v.serialize_to_bytes()};
        obj.serialize(serializer)
    }
}
pub struct DeserializeTable<T:?Sized>{
    pub internal:Mutex<HashMap<String, fn(Vec<u8>)->Box<T>>>,
}
pub struct VoidPtr{
    ptr:*const c_void,
}
unsafe impl Send for VoidPtr{}
unsafe impl Sync for VoidPtr{}
impl <T:?Sized+'static+DynSer> Obj<T>{

    pub fn get_table()->&'static DeserializeTable<T>{
        //safety, points to a static deserializable table, will never be freed.
        unsafe{
            static TABLES:LazyLock<Mutex<HashMap<String,VoidPtr>>> = LazyLock::new(||{Mutex::new(HashMap::new())});
            let mut tables =TABLES.lock().unwrap();
            if !tables.contains_key(type_name::<T>()){
                let table = Box::new(DeserializeTable::<T>{internal:Mutex::new(HashMap::new())});
                let ptr = Box::leak(table) as *const DeserializeTable<T> as *const c_void;
                tables.insert(type_name::<T>().to_string(), VoidPtr{ptr});
                
            }
            return (tables[type_name::<T>()].ptr as *const DeserializeTable<T>).as_ref().unwrap()
        }

        
    }
}
pub fn register_deserializer<T:for<'a> Deserialize<'a>+DynSer+'static, U:DynSer+'static+?Sized>() where Box<T>:Into<Box<U>>{
    fn deserialize<T:for<'a> Deserialize<'a>, U:DynSer+'static+?Sized>(bytes:Vec<u8>)->Box<U> where Box<T>:Into<Box<U>>{
          let v:Box<T> =Box::new(serde_json::from_slice(&bytes).unwrap());
          let out:Box<U>= v.into();
          out
    }
    let deserializer = Obj::<U>::get_table();
    let mut lock = deserializer.internal.
    lock().unwrap();
    let func = deserialize::<T,U>;
    lock.insert(type_name::<T>().to_string(), func);
}
impl <'de,T:?Sized+'static + DynSer> Deserialize<'de> for Obj<T>{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let functions = Obj::<T>::get_table().internal.lock().unwrap();
        let obj = SerializeablObj::deserialize(deserializer)?;
        let v = (functions[&obj.name])(obj.data);
        return Ok(Self{v})

    }
}