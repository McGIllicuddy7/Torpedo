#include "techno.h"
#ifdef __MACH__
#include </opt/homebrew/include/raylib.h>
#else 
#include <raylib.h>
#endif
#include <stdio.h>
int main(){
	InitWindow(1000, 750, "Brigui");
	while(!WindowShouldClose()){
		
		techno_begin();
		printf("begin frame\n");
		techno_panel_begin(000,200, 400, 200);
		techno_text("hello window");
		techno_text("testing 123");
		techno_button("button", "test-button");
		techno_panel_end();
		draw_call_t * draw =techno_end();	
		BeginDrawing();
		ClearBackground(BLACK);
		draw_render_queue(draw);
		EndDrawing();
		printf("end frame\n");
	}
	CloseWindow();
}
