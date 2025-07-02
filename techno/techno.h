#pragma once
#include <stdlib.h>
#include <stdbool.h>
#include <assert.h>
#include <_static_assert.h>
typedef enum{
	DrawCallHeader, DrawCallText, DrawCallPanel, DrawCallButton,DrawCallImage
} draw_call_type_t;

typedef struct draw_call_t{
	struct draw_call_t * next;
	const char * string;
	int x;
	int y;
	int height;
	int width;
	char r;
	char g;
	char b;
	char a;
	draw_call_type_t to_call;
} draw_call_t;
void techno_begin();
draw_call_t* techno_end();

void techno_color(int r, int g, int b, int a);
bool techno_panel_begin(int x, int y, int width, int height);
void techno_panel_end();
bool techno_button(const char * text,const char * name);
void techno_text(const char * text);
void techno_title(const char * text);
void techno_scroll_box_begin(int depth);
void techno_scroll_box_end();

