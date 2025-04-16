use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vector3{
    pub x:f64, 
    pub y:f64,
    pub z:f64,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vector4{
    pub x:f64, 
    pub y:f64, 
    pub z:f64, 
    pub w:f64,
}
pub type Quaternion = Vector4;
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform{
    pub translation:Vector3, 
    pub rotation:Vector4, 
    pub scale:Vector3,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BoundingBox{
    pub min:Vector3, 
    pub max:Vector3,
}

impl Add for Vector3{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self{x:self.x+rhs.x, y:self.y+rhs.y, z:self.z+rhs.z}
    }
}
impl Sub for Vector3{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{x:self.x-rhs.x, y:self.y-rhs.y, z:self.z-rhs.z}
    }
}
impl Mul<f64> for Vector3{
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self{x:self.x*rhs, y:self.y*rhs, z:self.z*rhs}
    }
}
impl Div<f64> for Vector3{
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self{x:self.x/rhs, y:self.y/rhs, z:self.z/rhs}
    }
}

impl AddAssign for Vector3{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self+rhs;
    }
}
impl SubAssign for Vector3{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self-rhs;
    }
}
impl MulAssign<f64> for Vector3{
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self*rhs;
    }
}
impl DivAssign<f64> for Vector3{
    fn div_assign(&mut self, rhs: f64) {
        *self = *self/rhs;
    }
}
impl Neg for Vector3{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self{x:-self.x, y:-self.y, z:-self.z}
    }
}
impl Vector3{
    pub const fn new(x:f64, y:f64, z:f64)->Self{
        Self{x,y,z}
    }
    pub const fn zero()->Self{
        Self{x:0., y:0., z:0.}
    }
    pub fn length(&self)->f64{
        (self.x*self.x+self.y*self.y+self.z*self.z).sqrt()
    }
    pub const fn dot(&self, rhs:Self)->f64{
        self.x*rhs.x+self.y*rhs.y+self.z*rhs.z
    }
    pub const fn length_sqr(&self)->f64{
        self.dot(*self)
    }
    pub fn normalized(&self)->Self{
        *self/self.length()
    }
    pub fn normalize(&mut self){
        *self = *self/self.length()
    }
    pub fn transform_with(&self,mat:raylib::math::Matrix)->Self{
        let s = self.as_rl_vec();
        Self::from_rl_vec(s.transform_with(mat))
    }
    pub fn transform(&mut self, mat:raylib::math::Matrix){
        let s = self.as_rl_vec();
        *self = Self::from_rl_vec(s.transform_with(mat))
    }
    pub const fn as_rl_vec(&self)->raylib::math::Vector3{
        raylib::math::Vector3{x:self.x as f32, y:self.y as f32, z:self.z as f32}
    }
    pub const fn from_rl_vec(v:raylib::math::Vector3)->Self{
        Self { x: v.x as f64, y: v.y as f64, z: v.z as f64}
    }
    pub fn reflect(&mut self, v:Vector3){
        *self = Self::from_rl_vec(self.as_rl_vec().reflect_from(v.as_rl_vec()))
    }
}

impl Mul<f64> for Vector4{
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self{x:self.x*rhs, y:self.y*rhs, z:self.z*rhs,w:self.w*rhs}
    }
}
impl Div<f64> for Vector4{
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self{x:self.x/rhs, y:self.y/rhs, z:self.z/rhs, w:self.w*rhs}
    }
}

impl MulAssign<f64> for Vector4{
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self*rhs;
    }
}
impl DivAssign<f64> for Vector4{
    fn div_assign(&mut self, rhs: f64) {
        *self = *self/rhs;
    }
}
impl Mul for Vector4{
    type Output= Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self{x:self.x*rhs.x, y:self.y*rhs.y,z:self.z*rhs.z, w:self.w*rhs.w }
    }
}

impl MulAssign for Vector4{

    fn mul_assign(&mut self, rhs: Self) {   
        *self = *self*rhs;
    }
}
impl Add for Vector4{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self{x:self.x+rhs.x, y:self.y+rhs.y, z:self.z+rhs.z, w:self.w+rhs.w}
    }
}
impl Sub for Vector4{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self{x:self.x-rhs.x, y:self.y-rhs.y, z:self.z-rhs.z, w:self.w+rhs.w}
    }
}
impl AddAssign for Vector4{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self+rhs;
    }
}
impl SubAssign for Vector4{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self-rhs;
    }
}
impl Vector4{
    pub fn length(&self)->f64{
        (self.x*self.x+self.y*self.y+self.z*self.z+self.w*self.w).sqrt()
    }
    pub const fn dot(&self, rhs:&Self)->f64{
        self.x*rhs.x+self.y*rhs.y+self.z*rhs.z+self.w*rhs.w
    }
    pub const fn length_sqr(&self)->f64{
        self.dot(self)
    }
    pub fn normalized(&self)->Self{
        *self/self.length()
    }
    pub fn normalize(&mut self){
        *self = *self/self.length()
    }
    pub const fn as_rl_vec(&self)->raylib::math::Vector4{
        raylib::math::Vector4{x:self.x as f32, y:self.y as f32, z:self.z as f32, w:self.w as f32}
    }
    pub const fn from_rl_vec(v:raylib::math::Vector4)->Self{
        Self { x: v.x as f64, y: v.y as f64, z: v.z as f64, w: v.w as f64}
    }
    pub fn to_matrix(&self)->raylib::math::Matrix{
        let t = self.as_rl_vec();
        t.to_matrix()
    }
    pub const fn new(x:f64, y:f64, z:f64, w:f64)->Self{
        Self { x,y,z,w}
    }
    pub const fn zero()->Self{
        Self { x:0.,y:0., z:0., w:0.}
    }
}

impl Default for Transform{
    fn default() -> Self {
        Self { translation: Vector3::zero(), rotation:Quaternion::new(0., 0., 0., 1.), scale: Vector3::new(1., 1., 1.) }
    }
}