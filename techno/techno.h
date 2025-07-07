#pragma once
#include <stdlib.h>
#include <stdbool.h>
#include <assert.h>
;
typedef struct{
	char r;
	char g;
	char b;
	char a;
}techno_color_t;
typedef enum{
	DrawCallHeader, DrawCallText, DrawCallPanel, DrawCallButton,DrawCallImage, DrawCallRectangle, DrawCallCircle
} draw_call_type_t;

typedef struct draw_call_t{
	struct draw_call_t * next;
	const char * string;
	int x;
	int y;
	int height;
	int width;
	int text_height;
	techno_color_t text_col;
	techno_color_t border_col;
	techno_color_t background_col;
	techno_color_t panel_col;
	draw_call_type_t to_call;
} draw_call_t;
void techno_begin();
draw_call_t* techno_end();
void techno_border_color(int r, int g, int b, int a);
void techno_text_color(int r, int g, int b, int a);
void techno_background_color(int r, int g, int b, int a);
bool techno_panel_begin(int x, int y, int width, int height);
void techno_panel_end();
bool techno_button(const char * text,const char * name);
void techno_text(const char * text);
void techno_title(const char * text);
void techno_set_text_size(int size);
void techno_progress_bar(double value, double min, double max, int height, bool horizontal);
double techno_slider(double start_value, int height);
void draw_call(draw_call_t * dc);
void draw_render_queue(draw_call_t * start);
