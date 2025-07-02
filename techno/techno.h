#pragma once
#include <stdlib.h>
#include <stdbool.h>
void techno_begin();
void techno_end();

void techno_color(int r, int g, int b, int a);
void techno_panel_begin(int x, int y, int width, int height);
void techno_panel_end();
bool techno_button(const char * text,const char * name);
void techno_text(const char * text);
void techno_title(const char * text);
void techno_scroll_box_begin(int depth);
void techno_scroll_box_end();
void techno_text_box(char buffer, size_t buffer_len, int depth);
