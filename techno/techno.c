#define CTILS_IMPLEMENTATION
#include "techno.h"
#ifdef __MACH__
#include </opt/homebrew/include/raylib.h>
#else 
#include <raylib.h>
#endif
#include <string.h>
#include <stdio.h>
#ifndef TODO
#define TODO() assert(false) 
#endif 
#define MAX_PANEL_COUNT 128
#define ALLOCATOR_SIZE 16000
typedef struct{
	int x;
	int y;
	int width;
	int height;
	int ptr;
	int border;
} techno_panel_t;
typedef struct{
	int height;
	int width;
}dimensions_t;
typedef struct {
	int x;
	int y;
}point_t;
typedef struct {
	int x;
	int y;
	int height;
	int width;
}techno_rect_t;
static int get_text_wrapped_height(const char * string, int pixel_size, int width);
static point_t center_text(const char * string, int pixel_size, int base_width, int in_height);
static dimensions_t get_text_dimensions(const char * text, int pixel_dimensions);
static techno_rect_t request_slab(int height);
static int get_usable_panel_width();

typedef struct {
	techno_panel_t panel_stack[MAX_PANEL_COUNT];
	int panel_stack_ptr;
	draw_call_t* draw_calls;
	draw_call_t * last_draw_call;
	char allocator[ALLOCATOR_SIZE];
	int alloc_ptr;
	techno_color_t text_color;
	techno_color_t background_color;
	techno_color_t border_color;
	techno_color_t panel_color;
	int pixel_size;
	const char * current_object;
	int cursor_x;
	int cursor_y;
	bool mouse_is_down;
	bool mouse_was_down;
	bool selected_object;
}techno_state_t;
static techno_state_t state = {0};
static techno_state_t * current_state = &state;
static void * alloc_bytes(size_t byte_count){
	if(!byte_count%sizeof(size_t) != 0){
		byte_count += sizeof(size_t)-byte_count%sizeof(size_t);	
	}
	if(current_state->alloc_ptr+byte_count>ALLOCATOR_SIZE){
		return 0;
	} else{
		void * out = &current_state->allocator[current_state->alloc_ptr];
		current_state->alloc_ptr += byte_count;
		return out;
	} 
}

static bool push_draw_call(draw_call_t draw_call){
	extern int printf(const char *__restrict__ ,...);
	//printf("wants to draw<%s>\n", draw_call.string);
	draw_call_t * dc = alloc_bytes(sizeof(draw_call_t));
	if (!dc){
		assert(false);
		return true;
	}
	draw_call.next =0;
	*dc = draw_call;
	if(current_state->last_draw_call){	
		current_state->last_draw_call->next = dc;
		current_state->last_draw_call = dc;
	}else{	
		current_state->draw_calls = dc;
		current_state->last_draw_call = dc;	
	}
	return false;
}
static techno_panel_t * get_current_panel(){
	if(current_state->panel_stack_ptr ==0){
		return 0;
	}
	return & current_state->panel_stack[current_state->panel_stack_ptr-1];
}
static techno_rect_t request_slab(int height){
	techno_panel_t * panel  = get_current_panel();
	int width = panel->width-2*panel->border;	
	int x = panel->x+panel->border;
	int y = panel->y+panel->ptr+panel->border;
	panel->ptr += height+panel->border;
	techno_rect_t out;
	out.x = x;
	out.y = y;
	out.height = height;
	out.width =width;
	return out;
}
static int get_usable_panel_width(){
	techno_panel_t* panel = get_current_panel();
	return panel->width-2*panel->border;
}
void techno_begin(){		
	current_state->panel_stack_ptr = 0;
	current_state->alloc_ptr = 0;
	current_state->selected_object = false;
	current_state->cursor_x = GetMouseX();
	current_state->cursor_y = GetMouseY();
	current_state->draw_calls =0;
	current_state->last_draw_call =0;
	current_state->pixel_size = 12;
	current_state->text_color = (techno_color_t){255, 255, 255, 255};
	current_state->background_color = (techno_color_t){128, 128, 128, 255};
	current_state->panel_color = (techno_color_t){32, 32, 32, 255};
	if(IsMouseButtonDown(MOUSE_BUTTON_LEFT)){
		current_state->mouse_is_down = true;
		current_state->mouse_was_down = true;
	} else{
		current_state->mouse_was_down = current_state->mouse_is_down;
		current_state->mouse_is_down = false;
	}
}
draw_call_t* techno_end(){
	if(!current_state->selected_object){
		void * obj = (void*)current_state->current_object;
		free(obj);
		current_state->current_object =0;
	}
	return current_state->draw_calls;
}

void techno_text_color(int r, int g, int b, int a){
	current_state->text_color = (techno_color_t){r,g,b,a};	
}
void techno_background_color(int r, int g, int b, int a){
	current_state->background_color = (techno_color_t){r,g,b,a};	
}
void techno_border_color(int r, int g, int b, int a){
	current_state->border_color = (techno_color_t){r,g,b,a};	
}
bool techno_panel_begin(int x, int y, int width, int height){
	techno_panel_t pan;
	techno_panel_t * current ;
	if((current = get_current_panel())){
		x+=current->x+current->border;
		y+=current->ptr+current->border;
		current->ptr += current->border+height;
		if(width+current->border>current->width){
			width = current->width-current->border;
		}
		if(height+current->border>current->height){
			height = current->height-current->border;
		}
	}
	pan.x = x;
	pan.y = y;
	pan.width = width;
	pan.height = height;
	pan.ptr = 0;
	pan.border = 5;
	if(current_state->panel_stack_ptr ==  MAX_PANEL_COUNT){
		assert(false);
		return true;
	}
	current_state->panel_stack[current_state->panel_stack_ptr] = pan;	
	draw_call_t dc = {0};
	dc.border_col = current_state->border_color;
	dc.text_col = current_state->text_color;
	dc.background_col = current_state->background_color;
	dc.panel_col = current_state->panel_color;
	dc.x = x;
	dc.y = y;
	dc.width = width;
	dc.height = height;
	dc.to_call = DrawCallPanel;
	push_draw_call(dc);
	current_state->panel_stack_ptr++;
	return false;
}
void techno_panel_end(){
	if(current_state->panel_stack_ptr>0) current_state->panel_stack_ptr -= 1;
}
bool techno_button(const char * text,const char * name){
	int width = get_usable_panel_width();
	int height = get_text_wrapped_height(text, current_state->pixel_size, width);
	techno_rect_t rect = request_slab(height);
	int x = current_state->cursor_x;
	int y = current_state->cursor_y;
	int out = 0;
	if(x>= rect.x && y>= rect.y && x<= rect.x+rect.width && y<= rect.y+rect.height){
		if(current_state->mouse_is_down){
			int l = strlen(name);
			char * v = malloc(strlen(name));
			memcpy(v, name, l-1);
			if(current_state->current_object) {
				free((void*)current_state->current_object);
			}
			current_state->current_object = v;
			current_state->selected_object = true;
		
		} else if(current_state->mouse_was_down){
			if(strcmp(current_state->current_object, name)){
				out = 1;
			}
		}
	}
	draw_call_t dc = {0};
	dc.x = rect.x;
	dc.y = rect.y;
	dc.height = rect.height;
	dc.width = rect.width;
	dc.string = text;
	dc.to_call = DrawCallButton;
	dc.text_col = current_state->text_color;
	dc.border_col = current_state->border_color;
	dc.background_col = current_state->background_color;
	push_draw_call(dc);
	return out;
}
void techno_text(const char * text){
	int width = get_usable_panel_width();
	int height = get_text_wrapped_height(text, current_state->pixel_size, width);
	techno_rect_t rect = request_slab(height);
	point_t p = center_text(text, current_state->pixel_size, width, height);
	draw_call_t dc = {0};
	dc.x = rect.x+p.x;
	dc.y = rect.y;
	dc.height = rect.height;
	dc.width = rect.width;
	dc.string = text;
	dc.to_call = DrawCallText;
	dc.text_col = current_state->text_color;
	dc.border_col = current_state->border_color;
	dc.background_col = current_state->background_color;
	dc.text_height = current_state->pixel_size;
	push_draw_call(dc);
}
void techno_title(const char * text){
	int width = get_usable_panel_width();
	int height = get_text_wrapped_height(text, current_state->pixel_size*2, width);
	techno_rect_t rect = request_slab(height);
	point_t p = center_text(text, current_state->pixel_size*2, width, height);
	draw_call_t dc = {0};
	dc.x = rect.x+p.x;
	dc.y = rect.y;
	dc.height = rect.height;
	dc.width = rect.width;
	dc.string = text;
	dc.to_call = DrawCallHeader;
	dc.text_col = current_state->text_color;
	dc.border_col = current_state->border_color;
	dc.background_col = current_state->background_color;
	dc.text_height = current_state->pixel_size*2;
	push_draw_call(dc);
}
void techno_scroll_box_begin(int depth){
	TODO();
}
void techno_scroll_box_end(){
	TODO();
}
static dimensions_t get_text_dimensions(const char * string, int pixel_size){
	int width = 0;
	int current_width =0;
	int height = 0;
	for(int i= 0;string[i] != 0; i++){
		if (string[i] == '\n'){
			height+= pixel_size;
			current_width =0;
		}
		char text[2] = {string[i], 0};
		int w = MeasureText(text, pixel_size);
		current_width += w;
		if(current_width>width){
			width = current_width;
		}
	}
	return (dimensions_t){width, height};
}
static int get_text_wrapped_height(const char * string, int pixel_size, int width){
	int height = 0;	
	int current_width =0;
	for(int i= 0;string[i] != 0; i++){
		if (string[i] == '\n'){
			height+= pixel_size;
			current_width =0;
			continue;
		}
		char text[2] = {string[i], 0};
		int w = MeasureText(text, pixel_size);
		current_width += w;
		if(current_width> width){
			current_width =w;
			height+= pixel_size;
		}	
	}
	return height+pixel_size;
}
static point_t center_text(const char * string, int pixel_size, int width, int in_height){
	int height = 0;	
	int current_width =0;
	int max_width =0;
	int max_char_width =0;
	for(int i= 0;string[i] != 0; i++){
		if (string[i] == '\n'){
			height+= pixel_size;
			current_width =0;
		}
		char text[2] = {string[i], 0};
		int w = MeasureText(text, pixel_size);
		if(w>=max_char_width){
			max_char_width = w;
		}
		current_width += w;
		if(current_width>max_width){
			max_width = current_width;
		}
		if(current_width> width){
			current_width =w;
			height+= pixel_size;
		}	
	}	
	int diff = (width-max_width)/2;
	int h = (in_height-height)/2;
	return (point_t){diff, h};
}
void draw_text(int x, int y, int width, int pixel_size, techno_color_t color,const char * text){	
	char draw_buff[1000] = {0};
	int i =0;
	int current = 0;
	int cw = 0;
	int height =0;
	if(!text){
		return;
	}
	while(text[current]){
		char text2[2] = {text[current],0};
		int w = MeasureText(text2, pixel_size);
		cw += w;
		if(cw>width){
			cw = w;
			height += pixel_size;
			DrawText(draw_buff, x, y+height, pixel_size, *(Color*)&color);		
			memset(draw_buff,0,1000);
			draw_buff[0] = text[current];
			i =1;
		} else{
			draw_buff[i] = text[current];
			i++;
			assert(i<1000);
		}
		current++;
	}
	DrawText(draw_buff, x, y+height, pixel_size, *(Color*)&color);	
}
void draw_call(draw_call_t * dc){
	printf("draw call\n");
	draw_call_type_t dt = dc->to_call;
	switch(dt){
		case DrawCallHeader:{
			draw_text(dc->x, dc->y, dc->width, dc->text_height,dc->text_col,dc->string);
			break;
		}
		case DrawCallText:{
			draw_text(dc->x, dc->y, dc->width, dc->text_height,dc->text_col,dc->string);
			break;
		}
		case DrawCallPanel:{
			printf("drew panel\n");
			DrawRectangleRounded((Rectangle){dc->x, dc->y, dc->width, dc->height}, 
			0.1, 10,*(Color*)&dc->panel_col);
			break;
		}
		case DrawCallButton:{
			point_t p = center_text(dc->string, current_state->pixel_size, dc->width, dc->height);	
			DrawRectangleRounded((Rectangle){dc->x, dc->y, dc->width, dc->height}, 
			0.1, 10,*(Color*)&dc->background_col);
			draw_text(dc->x+p.x, dc->y, dc->width, dc->text_height, dc->text_col, dc->string);
			break;
		}
		case DrawCallImage:{
			TODO();
			break;
		};
		default:TODO();
		break;
	}
}
void draw_render_queue(draw_call_t * start){
	draw_call_t * current = start;
	while(current){
		draw_call(current);
		current = current->next;
	}
}
