

use dynamo::{register_deserializer, DynSer, Obj};
use serde_derive::{Deserialize, Serialize};

pub mod dynamo;
#[derive(Serialize, Deserialize)]
pub struct Test{
    pub v:String,
}
impl DynSer for Test{
    fn serialize_to_bytes(&self)->Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}
pub trait TestTrait:DynSer+{
    fn print(&self);
}
impl TestTrait for Test{
    fn print(&self) {
        println!("{:#?}", self.v)
    }
}
impl Into<Box<dyn TestTrait>> for Box<Test>{
    fn into(self) -> Box<dyn TestTrait> {
        self
    }
}
pub fn save_obj(){
    let v:Box<dyn TestTrait> = Box::new(Test{v:"lol".to_string()});
    let obj = Obj::<dyn TestTrait>{v};
    let bytes = serde_json::to_string_pretty(&obj).unwrap();
    let _ = std::fs::write("test.txt", bytes);
}
pub fn load_obj(){
    let p = std::fs::read_to_string("test.txt").unwrap();
    let v2:Obj<dyn TestTrait> = serde_json::from_str(&p).unwrap();
    v2.v.print();
}


fn main() {
    register_deserializer::<Test, dyn TestTrait>();
    let a:Obj<dyn TestTrait> = Obj::from_box(Box::new(Test{v:"hello world".to_string()}));
    save_obj();
}
