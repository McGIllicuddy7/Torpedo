use raylib::{color::Color, ffi::MouseButton};

use crate::draw_call::{self, draw_box, draw_rounded_box, draw_text};
pub type Void = ();
#[derive(Debug)]
pub struct UI{
    pub selected_item:i32,
    pub frames:Vec<Frame>,
    pub x:i32, 
    pub y:i32, 
    pub w:i32, 
    pub h:i32,
    pub ever_selected:bool,
} 

#[derive(Debug)]
pub struct Frame{
    pub is_vertical:bool, 
    pub bx:i32,
    pub by:i32, 
    pub bheight:i32, 
    pub bwidth:i32, 
    pub next_v:i32,
    pub offset:i32,
}
impl Frame{
    //x y w h
    fn add_child(&mut self,extent:i32)->Option<(i32, i32, i32, i32)>{
        if self.is_vertical{
            let out = (self.bx+self.offset, self.next_v+self.offset, self.bwidth -self.offset, extent);
            println!("v:{},{}", self.bheight, self.bwidth);
            self.next_v+= extent+self.offset;
            if extent<self.offset || self.next_v>self.by+self.bheight{
                return None;
            }
            Some(out)
        } else{
           let out =(self.next_v+self.offset, self.by+self.offset,  extent, self.bheight-self.offset);
           println!("h:{},{}", self.bheight,self.bwidth);
            self.next_v+= extent;
           if extent<self.offset || self.next_v>self.by+self.bheight{
               return None;
            }
            Some(out)
        }
    }
}
impl UI{
    pub fn new(x:i32, y:i32, h:i32, w:i32)->Self{
       let out = Self { selected_item: -1, frames: Vec::new(), x, y, w,h, ever_selected:false };
       out
    }
    pub fn new_frame(&mut self, vertical:bool, extent:i32, offset:i32)->Option<Void>{
        if let Some(prev) = self.frames.get_mut(0){
            let (x,y,w,h) = prev.add_child(extent)?;
            let next_v = if vertical{
                y
            } else{
                x
            };
            let f = Frame { is_vertical:vertical, bx:x, by:y, bheight:h, bwidth:w, next_v, offset };
            self.frames.push(f);
        } else{
            self.frames.push(Frame { is_vertical: false, bx: self.x, by: self.y, bheight: self.h, bwidth: self.w, next_v: self.x, offset: 10 });
            return self.new_frame(vertical, extent, offset);
        }
        Some(())
    }
    pub fn new_frame_v(&mut self,extent:i32, offset:i32)->Option<Void>{
        self.new_frame(true, extent, offset)
    }
    pub fn new_frame_h(&mut self, extent:i32, offset:i32)->Option<Void>{
        self.new_frame(false, extent, offset)
    }
    pub fn end_frame(&mut self)->Void{
        self.frames.pop();
    }
    pub fn end_drawing(&mut self)->Void{
        self.frames.clear();
        let mouse_down = unsafe{
            raylib::ffi::IsMouseButtonDown(MouseButton::MOUSE_BUTTON_LEFT as i32)
        };
        if !mouse_down{
            self.selected_item = -1;
        }
        if !self.ever_selected{
            self.selected_item = -1;
        }
    }
    pub fn current_frame(&mut self)->&mut Frame{
        let l = self.frames.len();
        &mut self.frames[l-1]
    }
    pub fn new_botton(&mut self, extent:i32, id:u16, color:Color)->bool{
        let (x, y, w, h) = self.current_frame().add_child(extent).unwrap(); 
        let colliding =           unsafe{
            let pos = raylib::ffi::GetMousePosition();
            let vx = pos.x as i32;
            let vy = pos.y as i32;
            if vx>= x && vy>= y && vx< x+w && vy<y+h{
                true
            } else{
                false
            }
        } ;
        let mouse_down = unsafe{
            raylib::ffi::IsMouseButtonDown(MouseButton::MOUSE_BUTTON_LEFT as i32)
        };
        let out = if self.selected_item == id as i32{
            if !mouse_down && colliding{
                true
            } else{
                false
            }
        } else{
            false
        };
        if mouse_down && colliding{
            self.selected_item = id as i32;
            self.ever_selected = true;
        }
        draw_box(x, y, h,w , selected_color(color, mouse_down && colliding));
        return out;
    }
    pub fn new_botton_text(&mut self, extent:i32, id:u16, color:Color, text:String, text_color:Color)->bool{
        let (x, y, w, h) = self.current_frame().add_child(extent).unwrap(); 
        let colliding =           unsafe{
            let pos = raylib::ffi::GetMousePosition();
            let vx = pos.x as i32;
            let vy = pos.y as i32;
            if vx>= x && vy>= y && vx< x+w && vy<y+h{
                true
            } else{
                false
            }
        } ;
        let mouse_down = unsafe{
            raylib::ffi::IsMouseButtonDown(MouseButton::MOUSE_BUTTON_LEFT as i32)
        };
        let out = if self.selected_item == id as i32{
            if !mouse_down && colliding{
                true
            } else{
                false
            }
        } else{
            false
        };
        if mouse_down && colliding{
            self.selected_item = id as i32;
            self.ever_selected = true;
        }
        draw_box(x, y, h,w , selected_color(color, mouse_down && colliding));
        draw_text_inside_box(x, y, h, w,text,text_color);
        return out;
    }
    pub fn new_text_box(&mut self,text:String, extent:i32, text_color:Color, bg:Color){
        let (x,y,w,h) = self.current_frame().add_child(extent).unwrap();
        draw_box(x, y, h,w, bg);
        draw_text_inside_box(x, y, h,w, text, text_color);
    }
    pub fn new_text_rounded(&mut self,text:String, extent:i32, text_color:Color, bg:Color){
        let (x,y,w,h) = self.current_frame().add_child(extent).unwrap();
        draw_rounded_box(x, y, h,w, bg,0.5, 100, );
        draw_text_inside_box(x, y, h,w, text, text_color);
    }
}
pub fn show_mouse(){
    unsafe {
        raylib::ffi::ShowCursor();
        raylib::ffi::EnableCursor();
    }
}
pub fn hide_mouse(){
    unsafe {
        raylib::ffi::HideCursor();
        raylib::ffi::DisableCursor();
    }
}
fn selected_color(color:Color, selected:bool)->Color{
    if !selected{
        return color;
    }
    let clen = color.r as i32+color.g as i32+color.b as i32;
    if clen>=128{
        let out = Color{r:color.r/2, g:color.g/2, b:color.b/2, a:color.a};
        out
    } else{
        let out = Color{r:color.r*2+10, g:color.g*2+10, b:color.b*2+10, a:color.a};
        out
    }
}
pub fn draw_text_inside_box(x:i32, y:i32, height:i32, width:i32, text:String, color:Color){
    draw_call::draw_text_bounded(x, y, height, width, text, color);
    
}