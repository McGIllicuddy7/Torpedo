#include "utils.h"
#include "level.h"
#include "physics.h"
#include "renderer.h"
#include "base.h"
extern Arena * arena_create_sized(size_t count);
Runtime runtime;
extern void draw_update();
void model_unload(Model * model){
    UnloadModel(*model);
}
void finalize_entity(EntityRef target);
Arena * frame_arena(){
    return runtime.level->frame_arena;
}
Arena * static_arena(){
    return runtime.static_arena;
}
void apply_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage){
    if(!entity_is_valid(source) || !entity_is_valid(target)){
        return;
    }
    get_level()->damage_handler(source, target, direction, damage);
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
void finalize_entities(){
	static int hit = false;
	if(runtime.level->destroy_queue.length){
	printf("to destroy count:%zu\n", runtime.level->destroy_queue.length);
	}

	for(size_t i =0; i<runtime.level->destroy_queue.length; i++){
		if(entity_is_valid(runtime.level->destroy_queue.items[i])){
		finalize_entity(runtime.level->destroy_queue.items[i]); printf("destroyed %u\n", runtime.level->destroy_queue.items[i].index);

		}
	}
	if(hit){
		//todo();
	}
	if(runtime.level->destroy_queue.length != 0){
		hit = true;
	}
	runtime.level->destroy_queue.length =0;
}
void tick(){ 
    static int t = 0;
    const int tc  = 0;
    if(t == tc){
        process_systems();
        physics_prepare_update();
    }
    game_render(&runtime.level->cam); 
    if(t == tc){
        physics_finish_update();
        process_events();
        finalize_entities();
    }
    t+= 1;
    if(t>tc){
        t = 0;
    }
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
EntityRef create_entity(){
    for(size_t i =0; i<ENTITY_COUNT; i++){
        if(!runtime.level->tags[i]){
            runtime.level->generations[i]++;
            runtime.level->tags[i] |= tag_alive;
            return (EntityRef){.index =i, .generation = runtime.level->generations[i]};
        }
    }
    return entity_null();
}
void finalize_entity(EntityRef target){
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
void destroy_entity(EntityRef target){
	v_append(get_level()->destroy_queue,target);
}
bool has_component(EntityRef e,OwnedComps cmp){
    return (runtime.level->owned_comps[e.index] &cmp )!= 0;
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
void remove_tag(EntityRef e, Tag tg){
    *(u32*)&runtime.level->tags[e.index] ^= tg; 
}
bool has_tag(EntityRef e, Tag tag){
    if(!entity_is_valid(e)){
        return false;
    }
    Tag t = runtime.level->tags[e.index];
    return t & tag;
}
EntityRefVec get_all_entities_with_tag(Tag tag){
    EntityRefVec out = make(frame_arena(), EntityRef);
    for(size_t i =0; i<ENTITY_COUNT; i++){
        EntityRef e;
        e.index =i;
        e.generation = get_level()->generations[i];
        if(has_tag(e,tag)){
            v_append(out,e);
        }
    }
    return out;
}
EntityRefVec get_all_entities_with_component(OwnedComps cmp){
    EntityRefVec out = make(frame_arena(), EntityRef);
    for(size_t i =0; i<ENTITY_COUNT; i++){
        EntityRef e;
        e.index =i;
        e.generation = get_level()->generations[i];
        if(has_component(e,cmp)){
            v_append(out,e);
        }
    }
    return out;
}
PhysicsComp * get_physics_comp(EntityRef ref){
    if(!entity_is_valid(ref)){
        return 0;
    }
    PhysicsComp * out =&get_physics_comps()[ref.index];
    return out; 
}
MeshComp * get_mesh_comp(EntityRef ref){
    if(!entity_is_valid(ref)){
        return 0;
    }
    MeshComp * out = &get_mesh_comps()[ref.index];
    return out;
}
Level * get_level(){
    return runtime.level;
}
EntityRef create_debug_cube(Vec3 pos){
    EntityRef out = create_entity();
    add_component(out, comp_physics);
    add_component(out, comp_model);
    PhysicsComp * phys = get_physics_comp(out);
    assert(phys);
    phys->is_valid = true;
    phys->mass = 1.0;
    phys->is_valid = true;
    phys->velocity = (Vec3){0,0,0};
    phys->trans.trans  = Trans_create();
    phys->trans.trans.translation = pos;
    Collider col;
    col.bb.min=(Vector3){-0.5,-0.5,-0.5};
    col.bb.max= (Vector3){0.5,0.5,0.5};
    phys->colliders[0] = col;
    phys->collider_count = 1;
    phys->angular_velocity = (Vec3){0,0,0};
    phys->can_ever_collide = true;
    phys->destroy_on_impact = false;
    MeshComp * mesh = get_mesh_comp(out);
    mesh->meshes[0].color = WHITE;
    mesh->meshes[0].offset = Trans_create();
    mesh->meshes[0].string = "cube";
    mesh->mesh_count =1;
    return out;
}

Level * create_level(){
    InitWindow(1920,1080, ":3");
    InitAudioDevice();
    DisableCursor();
    SetTargetFPS(61);
    runtime.static_arena = arena_create();
    runtime.level = (Level*)arena_alloc(runtime.static_arena,(sizeof(Level))); 
    Level * level = runtime.level;
	level->destroy_queue = make(runtime.static_arena, EntityRef);
    level->frame_arena = arena_create_sized(4096*1024);
    level->generations = (u32*)arena_alloc(runtime.static_arena, (sizeof(u32))*ENTITY_COUNT);
    level->events = make(frame_arena(), Event);
    level->systems = make(runtime.static_arena, System);
    level->hooks = make(runtime.static_arena, EventHandler);
    level->player_entity= entity_null();
    level->cam.up = (Vector3){0,0,1};
    level->cam.target = (Vector3){1,0,0};
    level->cam.fovy = 120;
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
    level->damage_handler = 0;
    level->cam_player_offset = Trans_create();
    StringModelHashTable_insert(level->models, new_string(0, "cube"),LoadModelFromMesh(GenMeshCube(1., 1., 1.)));
    register_system((System){draw_update});
    return level;
}