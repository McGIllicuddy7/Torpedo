#define LOLTH_IMPLEMENTATION
#include "lolth.h"
#define CTILS_IMPLEMENTATION
#include "utils.h" 
#include "ship.h"
#include "level.h"
#ifdef PROFILE
//#include <gperftools/profiler.h>
#include </opt/homebrew/include/gperftools/profiler.h>
#endif
#include <pthread.h>
extern void tick();
extern void* physics_loop(void*);
void draw_update(){
    //draw_cube((Vec3){0,0,0}, 1,1,1, WHITE);
}

void setup(){
    srand(time(0));
    Level * level = create_level();
    register_system((System){ship_update});
    register_system((System){projectile_update});
    level->components[SHIP_COMPS_IDX] = arena_alloc(0, ENTITY_COUNT*sizeof(ShipComp));
    level->components[PROJECTILE_COMPS_IDX] = arena_alloc(0, ENTITY_COUNT*sizeof(ProjectileComp));
    StringModelHashTable_insert(level->models, new_string(0, "ship"),LoadModel("assets/ship.glb"));
    int delt = 0;
    level->damage_handler = ship_handle_damage;
    level->handlers[MESH_COMPS_IDX] = mesh_handler;
    level->handlers[PHYSICS_COMPS_IDX] = physics_handler;
    level->handlers[SHIP_COMPS_IDX] = ship_handler;
    level->handlers[PROJECTILE_COMPS_IDX] = projectile_handler;
    level->actual_comp_count = 1;
    double scale = 10.0;
    for(int z = -delt; z<delt+1; z++){
        for(int y =-delt; y<delt+1;y++){
            for(int x= -delt; x<delt+1; x++){
                if(x == 0 && y == 0&& z == 0)continue;
                EntityRef et = create_debug_cube((Vec3){(double)x*scale,(double)y*scale,(double)z*scale});
                PhysicsComp * phys = get_physics_comp(et);
                phys->velocity = Vec3_scale(Vec3_normalize(phys->trans.trans.translation),-1.0);
            }
        }
    }
    EntityRef s = create_ship((Vec3){50, 0,0,}, (Vec3){1,0,0}, true);	
    EntityRef e = create_ship((Vec3){-50,0,0}, (Vec3){1,0,0}, false);
}
void tear_down(){
    StringModelHashTable_unmake(runtime.level->models);
    UnloadShader(runtime.level->shader);
    arena_destroy(runtime.static_arena);
}
void main_loop(){
    srand(time(0));
    pthread_t phys_thread;
    pthread_create(&phys_thread, 0, physics_loop, 0);
    while(!WindowShouldClose()){
        tick();
    }
}
int main(){
    #ifdef PROFILE
    ProfilerStart("output_inside.prof"); 
    #endif
    setup();
    main_loop();
    tear_down();
    CloseWindow();
    #ifdef PROFILE
    ProfilerStop();
    #endif
}
