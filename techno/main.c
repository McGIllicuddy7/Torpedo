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
		techno_panel_begin(000,200, 400, 200);
		techno_title("GUI TEST");
		techno_text("testing 123");
		if (techno_button("button", "test-button") )exit(0);
		techno_panel_end();
		draw_call_t * draw =techno_end();	
		BeginDrawing();
		ClearBackground(BLACK);
		draw_render_queue(draw);
		EndDrawing();	
	}
	CloseWindow();
}
