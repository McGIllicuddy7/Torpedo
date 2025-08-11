#include "level.h"
#include "physics.h"
#include "renderer.h"
Runtime runtime;
Arena * frame_arena(){
    return runtime.level->frame_arena;
}
Arena * static_arena(){
    return runtime.static_arena;
}
void apply_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage){
}
void process_systems(){
    for(size_t i =0; i<runtime.level->systems.length; i++){
        runtime.level->systems.items[i].update();
    }    
}
void process_events(){
    for(size_t i =0; i<runtime.level->events.length; i++){
    }
    runtime.level->events.length =0;
}
void tick(){ 
    process_systems();
    physics_prepare_update();
    update_physics();
    physics_finish_update();
    process_events();
    game_render(&runtime.level->cam); 
    arena_reset(frame_arena());
}

PhysicsComp *get_physics_comps(){
    return (PhysicsComp*)runtime.level->components[PHYSICS_COMPS_IDX];
}
MeshComp * get_mesh_comps(){
    return (MeshComp*)runtime.level->components[MESH_COMPS_IDX];
}
Tag * get_tags_ptr(){
    return runtime.level->tags;
}
OwnedComps* get_owned_comps_ptr(){
    return runtime.level->owned_comps;
}
bool entity_eq(EntityRef a, EntityRef b){
    return (a.index == b.index && a.generation == b.generation);
}
bool entity_is_valid(EntityRef e){
    if(e.generation == 0){
        return false;
    }
    if(e.index>=ENTITY_COUNT){
        return false;
    }
    return e.generation == runtime.level->generations[e.index];
}
EntityRef entity_null(){
    return (EntityRef){0,0};
}
void * fralloc(size_t count){
    return arena_alloc(runtime.level->frame_arena, count);
}
void * stalloc(size_t count){
    return arena_alloc(runtime.static_arena, count);
}
void register_system(System s){
    v_append(runtime.level->systems, s);
}
void draw_call(DrawCall dc){
    v_append(runtime.level->draw_calls, dc);
}
void draw_call_3d(DrawCall3D dc){
    v_append(runtime.level->draw3d_calls, dc);
}
void draw_sphere(Vec3 pos, double r, Color col){
    DrawCall3D out;
    out.color = col;
    out.draw_call_type = draw_call_sphere;
    out.draw_call_sphere_info.r = r;
    out.draw_call_sphere_info.pos = Vec3_to_Vector3(pos);
    draw_call_3d(out);
}
void draw_cube(Vec3 pos, double w,double h, double d, Color col){
    DrawCall3D out;
    out.color = col;
    out.draw_call_type = draw_call_cube;
    out.draw_call_cube_info.d = d;
    out.draw_call_cube_info.w = w;
    out.draw_call_cube_info.h = h;
    out.draw_call_cube_info.pos = Vec3_to_Vector3(pos);
    draw_call_3d(out);
}
void draw_line(Vec3 start, Vec3 end,Color col){
    DrawCall3D out;
    out.color=  col;
    out.draw_call_type= draw_call_line;
    out.draw_call_line_info.start = Vec3_to_Vector3(start);
    out.draw_call_line_info.end = Vec3_to_Vector3(end);
    draw_call_3d(out);
}
void draw_text(const char * text, int x, int y, int height, Color col){
    DrawCall out;
    out.color = col;
    out.draw_call_type = draw_call_text;
    out.draw_call_text_info.text = new_string(frame_arena(), text).items;
    out.draw_call_text_info.x = x;
    out.draw_call_text_info.y = y;
    out.draw_call_text_info.height = height;
    draw_call(out);
}
void draw_rect(int x, int y, int w, int h, Color col){
    DrawCall out;
    out.color = col;
    out.draw_call_type = draw_call_rect;
    out.draw_call_rect_info.x = x;
    out.draw_call_rect_info.y = y;
    out.draw_call_rect_info.width =w;
    out.draw_call_rect_info.height = h;
    draw_call(out);
}
void draw_circle(int x, int y, float r, Color col){
    DrawCall out;
    out.color = col;
    out.draw_call_type = draw_call_circle;
    out.draw_call_circ_info.x  =x;
    out.draw_call_circ_info.y = y;
    out.draw_call_circ_info.r = r;
    draw_call(out);
}
EntityRef create_entity(){}
void destroy_entity(EntityRef target){
    if(!entity_is_valid(target)){
        return;
    }
    get_tags_ptr()[target.index] =(Tag)0;
    runtime.level->owned_comps[target.index] = (OwnedComps)0;
    for(size_t i =0; i<COMPONENT_COUNT; i++){
        void (*dest)(void *, u32) = runtime.level->handlers[i].destructor;
        if(dest){
            dest(runtime.level->components[i], target.index);
        }
    }
}
bool has_component(EntityRef e,OwnedComps cmp){
    return runtime.level->owned_comps[e.index] ==cmp;
}
void add_component(EntityRef e, OwnedComps cmp){
    *((u64*)&runtime.level->owned_comps[e.index] )|= (u64)cmp;
}
void add_tag(EntityRef e, Tag tg){
    *(u32*)&runtime.level->tags[e.index] |= tg;
}
void remove_component(EntityRef e, OwnedComps cmp){
    *((u64*)&runtime.level->owned_comps[e.index] )^= (u64)cmp;

}
void remove_tag(EntityRef e, OwnedComps cmp){}
bool has_tag(EntityRef e, Tag tag){}
EntityRefVec get_all_entities_with_tag(Tag tag){}
EntityRefVec get_all_entities_with_component(OwnedComps cmp){}
