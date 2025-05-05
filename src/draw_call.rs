use std::sync::Mutex;

use raylib::{color::Color, prelude::{RaylibDraw, RaylibDrawHandle}};
#[allow(unused)]
static DRAWCALLS:Mutex<Vec<DrawCall>> = Mutex::new(Vec::new());

pub enum DrawCall{
    Text{x:i32, y:i32, height:i32, text:String, color:Color}, 
    Box{x:i32, y:i32, height:i32, width:i32, color:Color},
    Circle{x:i32, y:i32, radius:i32, color:Color}, 
    RoundedBox{x:i32, y:i32, height:i32, width:i32, roundness:f32, segments:i32, color:Color},
    BoxLines{x:i32, y:i32, width:i32, height:i32, color:Color}, 
    RoundedBoxLines{x:i32, y:i32, height:i32, width:i32, roundness:f32, segments:i32, color:Color}
}
pub fn run_draw_calls(handle:&mut RaylibDrawHandle){
    let mut calls = DRAWCALLS.lock().unwrap();
    for i in calls.iter(){
        match i{
            DrawCall::Text { x, y, height, text, color } => {
                        handle.draw_text(text.as_ref(), *x, *y, *height, *color);
                    }
            DrawCall::Box { x, y, height, width, color } => {
                        handle.draw_rectangle(*x, *y, *width, *height, *color);
                    }
            DrawCall::Circle { x, y, radius, color } => {
                        handle.draw_circle(*x, *y, *radius as f32, *color);
                    }
            DrawCall::RoundedBox { x, y, height, width, roundness, segments, color } => {
                handle.draw_rectangle_rounded(raylib::prelude::Rectangle{x:*x as f32, y:*y as f32, height:*height as f32, width: *width as f32}, *roundness, *segments, *color);
            }
            DrawCall::RoundedBoxLines { x, y, height, width, roundness, segments, color } => {
                handle.draw_rectangle_rounded_lines(raylib::prelude::Rectangle{x:*x as f32, y:*y as f32, height:*height as f32, width: *width as f32}, *roundness, *segments, *color);
            }
            DrawCall::BoxLines { x, y, height, width, color } => {
                handle.draw_rectangle_lines(*x, *y, *width, *height, *color);
            }
       }
    }
    calls.clear();
}
fn push_call(call:DrawCall){
    let mut lck = DRAWCALLS.lock().unwrap();
    lck.push(call);
}
pub fn draw_text(x:i32, y:i32, height:i32, text:String, color:Color){
    push_call(DrawCall::Text { x, y, height, text, color, });
}
pub fn draw_box(x:i32, y:i32, height:i32, width:i32, color:Color){
    push_call(DrawCall::Box { x, y, height, width, color });
}
pub fn draw_circle(x:i32, y:i32, radius:i32, color:Color){
    push_call(DrawCall::Circle { x, y, radius, color })
}
pub fn draw_rounded_box(x:i32, y:i32, height:i32, width:i32, color:Color, roundness:f32, segments:i32){
    push_call(DrawCall::RoundedBox { x, y, height, width, color , roundness, segments});
}
pub fn draw_box_lines(x:i32, y:i32, height:i32, width:i32, color:Color){
    push_call(DrawCall::BoxLines { x, y, height, width, color });
}
pub fn draw_rounded_box_lines(x:i32, y:i32, height:i32, width:i32, color:Color, roundness:f32, segments:i32){
    push_call(DrawCall::RoundedBoxLines { x, y, height, width, color , roundness, segments});
}
