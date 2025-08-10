#define CTILS_IMPLEMENTATION
#include "utils.h" 
#include "level.h"
extern void tick();
void model_unload(Model * model){
    UnloadModel(*model);
}
void setup(){
    InitWindow(1000,750, ":3");
    runtime.level = malloc(sizeof(Level));
    Level * level = runtime.level;
    level->cam.up = (Vector3){0,0,1};
    level->cam.target = (Vector3){1,0,0};
    level->cam.fovy = 90.0;
    level->cam.position = (Vector3){0,0,0};
    level->cam.projection = CAMERA_PERSPECTIVE;
    level->tags = make(0, Tag);
    level->alive = make(0,bool);
    level->meshes = make(0,MeshComp);
    level->physics = make(0,PhysicsComp);
    level->shader = LoadShader("vertex.glsl", "frag.glsl");
    level->models = StringModelHashTable_create(4096,hash_string, string_equals, unmake_string, model_unload);
    level->should_load = false;
    level->should_save = false;
    level->load_name =0;
    level->save_name = 0;
    StringModelHashTable_insert(level->models, new_string(0, "cube"),LoadModelFromMesh(GenMeshCube(1., 1., 1.)));
}
void main_loop(){
    while(!WindowShouldClose()){
        tick();
    }
}
int main(){
    setup();
    main_loop();
    CloseWindow();
}
