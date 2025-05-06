use std::sync::Mutex;

use pprof::flamegraph::color;
use raylib::{color::Color, prelude::{RaylibDraw, RaylibDrawHandle}, text::{Font, WeakFont}};
#[allow(unused)]
static DRAWCALLS:Mutex<Vec<DrawCall>> = Mutex::new(Vec::new());

pub enum DrawCall{
    Text{x:i32, y:i32, height:i32, text:String, color:Color}, 
    TextBounded{x:i32, y:i32, h:i32, w:i32, text:String, color:Color},
    Box{x:i32, y:i32, height:i32, width:i32, color:Color},
    Circle{x:i32, y:i32, radius:i32, color:Color}, 
    RoundedBox{x:i32, y:i32, height:i32, width:i32, roundness:f32, segments:i32, color:Color},
    BoxLines{x:i32, y:i32, width:i32, height:i32, color:Color}, 
    RoundedBoxLines{x:i32, y:i32, height:i32, width:i32, roundness:f32, segments:i32, color:Color},
}
pub fn run_draw_calls(handle:&mut RaylibDrawHandle, font:&WeakFont){
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
            DrawCall::TextBounded { x, y, h, w, text, color }=>{
                render_text_bounded(handle, font,*x, *y, *h, *w, text.as_ref(), *color);
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
pub fn draw_text_bounded(x:i32, y:i32, h:i32, w:i32, text:String, color:Color){
    push_call(DrawCall::TextBounded { x, y, h,w, text, color, });
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



pub fn render_text_bounded(_handle:&mut RaylibDrawHandle, font:&WeakFont, x:i32, y:i32, h:i32, w:i32, text:&str, color:Color){
    unsafe{
        let font_size = 12.0;
        let ptxt = text.to_string() +"\0";
        let bounds = raylib::ffi::MeasureTextEx(**font, ptxt.as_ptr() as *const i8, font_size, 1.0);
        let scaler = h as f32/bounds.y;
        let bx = (w as f32-(bounds.x-font_size*6./15.)*scaler)/2.;
        let by = (h as f32-bounds.y*scaler)/2.;
        let pos = raylib::ffi::Vector2{ x: x as f32+bx, y: y as f32+by};
        raylib::ffi::DrawTextEx(**font, ptxt.as_ptr() as *const i8, pos, font_size*scaler, 1.0, color.into());
    }

}