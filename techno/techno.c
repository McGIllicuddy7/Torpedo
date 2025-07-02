#define CTILS_IMPLEMENTATION
#include "techno.h"
#include </opt/homebrew/include/raylib.h>
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
	int r;
	int g;
	int b;
	int a;
}techno_color_t;
static techno_panel_t panel_stack[MAX_PANEL_COUNT];
static int panel_stack_ptr = 0;
static draw_call_t* draw_calls = 0;
static draw_call_t * last_draw_call = 0;
static char allocator[ALLOCATOR_SIZE];
static int alloc_ptr = 0;
static techno_color_t current_color;
const char * current_object = 0;
static int cursor_x = 0;
static int cursor_y = 0;
static bool mouse_is_down =0;
static void * alloc_bytes(size_t byte_count){
	if(!byte_count%sizeof(size_t) != 0){
		byte_count += sizeof(size_t)-byte_count%sizeof(size_t);	
	}
	if(alloc_ptr+byte_count>ALLOCATOR_SIZE){
		return 0;
	} else{
		void * out = &allocator[alloc_ptr];
		alloc_ptr += byte_count;
		return out;
	} 
}
static bool push_draw_call(draw_call_t draw_call){
	draw_call_t * dc = alloc_bytes(sizeof(draw_call_t));
	if (!dc){
		return true;
	}
	*dc = draw_call;
	if(last_draw_call){
		last_draw_call->next = dc;
		last_draw_call = dc;
	}else{
		draw_calls = dc;
		last_draw_call = dc;	
	}
	return false;
}
static techno_panel_t * get_current_panel(){
	if(panel_stack_ptr ==0){
		return 0;
	}
	return & panel_stack[panel_stack_ptr-1];
}
void techno_begin(){		
	panel_stack_ptr = 0;
	alloc_ptr = 0;
}
draw_call_t* techno_end(){
	return draw_calls;
}

void techno_color(int r, int g, int b, int a){
	current_color = (techno_color_t){r,g,b,a};	
}
bool techno_panel_begin(int x, int y, int width, int height){
	techno_panel_t pan;
	techno_panel_t * current ;
	if((current = get_current_panel())){
		x+=current->x+current->border;
		y+=current->y+current->border;
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
	if(panel_stack_ptr ==  MAX_PANEL_COUNT){
		return true;
	}
	panel_stack[panel_stack_ptr] = pan;	
	return false;
}
void techno_panel_end(){
	panel_stack_ptr -= 1;
}
bool techno_button(const char * text,const char * name){
}
void techno_text(const char * text){
}
void techno_title(const char * text){
}
void techno_scroll_box_begin(int depth){}
void techno_scroll_box_end(){}

