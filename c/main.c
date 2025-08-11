#define CTILS_IMPLEMENTATION
#include "utils.h" 
#include "level.h"
extern void tick();
void draw_update(){
    draw_cube((Vec3){0,0,0}, 1,1,1, WHITE);
}
void model_unload(Model * model){
    UnloadModel(*model);
}
void setup(){
    InitWindow(1000,750, ":3");
    runtime.static_arena = arena_create();
    runtime.level = (Level*)arena_alloc(runtime.static_arena,(sizeof(Level))); 
    Level * level = runtime.level;
    level->generations = (u32*)arena_alloc(runtime.static_arena, (sizeof(u32))*ENTITY_COUNT);
    level->events = make(frame_arena(), Event);
    level->systems = make(runtime.static_arena, System);
    level->hooks = make(runtime.static_arena, EventHandler);
    level->player_entity= entity_null();
    level->frame_arena = arena_create();
    level->cam.up = (Vector3){0,0,1};
    level->cam.target = (Vector3){1,0,0};
    level->cam.fovy = 90.0;
    level->cam.position = (Vector3){0,0,0};
    level->cam.projection = CAMERA_PERSPECTIVE;
    level->tags = (Tag*)arena_alloc(runtime.static_arena,ENTITY_COUNT*sizeof(Tag));
    level->owned_comps=(OwnedComps*)arena_alloc(runtime.static_arena,ENTITY_COUNT*sizeof(OwnedComps));
    level->components = (void**)arena_alloc(runtime.static_arena,COMPONENT_COUNT*sizeof(void*));
    level->components[MESH_COMPS_IDX] = arena_alloc(runtime.static_arena,ENTITY_COUNT*sizeof(MeshComp));
    level->components[PHYSICS_COMPS_IDX] = arena_alloc(runtime.static_arena,ENTITY_COUNT*sizeof(PhysicsComp));
    level->shader = LoadShader("shaders/vertex.glsl", "shaders/frag.glsl");
    level->models = StringModelHashTable_create(4096,hash_string, string_equals, unmake_string, model_unload);
    level->should_load = false;
    level->should_save = false;
    level->load_name =0;
    level->save_name = 0;
    StringModelHashTable_insert(level->models, new_string(0, "cube"),LoadModelFromMesh(GenMeshCube(1., 1., 1.)));
    register_system((System){draw_update});
}
void tear_down(){
    StringModelHashTable_unmake(runtime.level->models);
    UnloadShader(runtime.level->shader);
    arena_destroy(runtime.static_arena);
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
    tear_down();
}
