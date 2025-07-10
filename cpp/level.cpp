#include "level.hpp"
#include "renderer/renderer.hpp"
#include "physics/physics.hpp"
#include "shaders.hpp"
#include "ship/ship.hpp"
#include <stdio.h>
namespace Torpedo{
Runtime runtime;
void update();
void run_destructors();
void set_player_entity(EntityRef ref){
	runtime.level->player = ref.get();
}

void update(){
    for(int i =0; i<runtime.level->entities.size(); i++){
        if(runtime.level->entities[i].get()){
            runtime.level->entities[i]->on_tick();
        }
    }
}
void process_events(){
    for(int i =0; i<runtime.level->event_queue.size(); i++){
	Event e = runtime.level->event_queue[i];
	if(e.EventType ==EventType::ApplyDamage){
		EntityRef ent = EntityRef{.index = e.target_idx, .generation = e.target_generation};
		if(!ent.get()) continue;
		ent.get()->on_damage(e.apply_damage.direction, e.apply_damage.damage);
	}
    } 
}
void cam_update(Camera * cam){
	if (runtime.level->player){
		cam->position = runtime.level->player->get_physics().trans.trans.translation;
		cam->target = runtime.level->player->get_forward_vector()+cam->position;
		cam->up = runtime.level->player->get_up_vector();
//		printf("%f,%f,%f\n", cam->up.x, cam->up.y, cam->up.z);
	}
	else {
    		UpdateCamera(cam, CAMERA_FREE);
	}
}
void mainloop(const char * startup_level){
    InitWindow(GetScreenWidth(), GetScreenHeight(), "brid-get");
    DisableCursor();
    Camera cam;
    cam.up = {0,0,1};
    cam.target = {-1, 0,0};
    cam.fovy = 120;
    cam.position = {20,0,0};
    cam.projection = CAMERA_PERSPECTIVE; 
    load_level(startup_level);
    setup();
    SetTargetFPS(60);
    RenderTexture2D post_texture = LoadRenderTexture(GetScreenWidth(),GetScreenHeight());
    Shader post_shader = LoadShader("./shaders/vertex.glsl", "./shaders/postfrag.glsl");
    while(!WindowShouldClose()){
	runtime.level->draw_calls.clear();
	runtime.level->event_queue.clear();
	update();
        physics_prepare_update();
        update_physics();
	cam_update(&cam);
        renderer_update(&cam,post_texture, post_shader);
        physics_finish_update();
	process_events();
	run_destructors();
    }  
    CloseWindow();
}

Entity::~Entity(){

}
MeshComp & Entity::get_mesh(){
    return get_level().meshes[id];
}
PhysicsComp& Entity::get_physics(){
    return get_level().physics[id];
}

void Entity::on_tick(){

}

bool Entity::has_tag(Tag tag)const{
    return tag & tags;
}
void Entity::add_tag(Tag tag){
    tags |= tag;
}
void Entity::remove_tag(Tag tag){
    tags &= ~tag;
}
void Entity::on_damage(Vec3 incoming_direction,double damage){
    
}

vector<unsigned char> Entity::serialize(){
    return {};
}

unique_ptr<Entity> Entity::deserialze(std::string_view name,vector<unsigned char> bytes){
    return 0;
} 

void Entity::set_velocity(Vec3 vel){
}

Vec3 Entity::get_velocity(){
    return Vec3{0,0,0};
}

void setup(){

}
void run_destructors(){
	if(!runtime.level->destroy_queue.size()){
		return;
	}	
	size_t index = runtime.level->destroy_queue.size();
	do {
		index -=1;
		size_t idx = runtime.level->destroy_queue[index];
		if(!runtime.level->entities[idx]){	
		    continue;
		}
		Entity *ptr = runtime.level->entities[idx].get();
		if(!ptr){
 
		    continue;
		}
		if(runtime.level->player == ptr){
			runtime.level->player = 0;
		}
		runtime.level->meshes[idx].reset();
		runtime.level->physics[idx].reset();
		runtime.level->physics[idx].is_valid = false;
		runtime.level->entities[idx] = 0;
		assert(runtime.level->entities[idx] == 0);
	}while(index>0);
	runtime.level->destroy_queue.clear();

}


Level & get_level(){
    return *runtime.level;
}

void load_level(const char * path){
    #define MULT
    runtime.level = std::make_unique<Level>(Level{});
    runtime.level->player = 0;
    get_level().models[string("cube")]= LoadModelFromMesh(GenMeshCube(0.5, 0.5, 0.5)); 
    get_level().models[string("cylinder")] = LoadModelFromMesh(GenMeshCube(0.2,0.02, 0.02));
    get_level().models[string("ship")] = LoadModel("../assets/ship.glb");
    Shader shader = LoadShader("shaders/vertex.glsl", "shaders/frag.glsl");
    get_level().models[string("cube")].materials[0].shader = shader;
    get_level().models[string("cube")].materials->maps->color = BLACK;

    get_level().models[string("ship")].materials[0].shader = shader;
    get_level().models[string("ship")].materials[0].maps->texture = LoadTexture("../assets/ship_texture.png");
    int64_t dims = 4;
    EntityRef player =create_player_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)}, Quat{0,0,0,1});
    EntityRef enemy = create_npc_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)},Quat{0,0,0,-1},Alignment::EnemyAligned);
    #ifdef MULT
    int count =2;
    for(int x = -count; x<count+1; x++){
        for(int y = -count; y<count+1; y++){
            for(int z = -count; z<count+1; z++){
                Vec3 point = Vec3{(double)x,(double)y,(double)z}*40;
                Vec3 v;
                v.x = x == 0 ? 0 : (x> 0 ? -1 : 1);
                v.y = y == 0 ? 0 : (y> 0 ? -1 : 1);
                v.z = z == 0 ? 0 : (z> 0 ? -1 : 1);
                Vec3 ang;
                ang.x = (rand()%1000)/1000.0*2-1;
                ang.y= (rand()%1000)/1000.0*2-1;
                ang.z = (rand()%1000)/1000.0*2-1;
                ang *= 0.0;
                EntityRef a = create_cube(point,Vec3{0.5, 0.5, 0.5}, v, WHITE, ang);
            }
        }
    }
    #endif
    #ifndef MULT
    double s = rand()%1000/1000.0*2*M_PI;
    Vec3 p1 = Vec3{-1, sin(s), cos(s)};
    Vec3 p2 = Vec3{-1, cos(s), -sin(s)};
    Vec3 v1 = {0,-sin(s), -cos(s)};
    Vec3 v2 = {0,-cos(s), sin(s)};
    double scale = 5.0;
    double speed = 0.5;
    create_cube(p1*scale, Vec3{1,1,1}, v1*speed, RED);
    create_cube(p2*scale, Vec3{1,1,1}, v2*speed, BLUE);
    #endif

}
EntityRef create_cube(Vec3 location, Vec3 scale, Vec3 velocity, Color color, Vec3 angular){
    MeshPart m;
    m.string = "cube";
    m.offset= Trans::create(); 
    m.color = color;
    EntityRef e = create_entity<Entity>();
    e.get()->add_tag(tag_movable);
    e.get()->get_mesh().meshes["base"] = m;
    PhysicsComp phys = {0};
    phys.mass = 1.0;
    phys.is_valid = true;
    phys.trans.trans = Trans::create();
    phys.trans.trans.translation = location;
    phys.destroy_on_impact = false; 
    phys.angular_velocity = Vec3{angular.x, angular.y, angular.z};
    Collider col;
    col.offset= Trans::create();
    Vec3 mscale = Vec3{-scale.x, -scale.y, -scale.z};
    //col.bb = BoundingBox{Vec3{-1.91526,-0.309, -0.309}/2.0, Vec3{1.0067,0.309, 0.309}/2.0};
    col.bb = BoundingBox{mscale, scale};
    phys.colliders.push_back(col);
    phys.velocity = velocity; 
    e.get()->get_physics()= phys;
    return e;
}
void destroy_entity(EntityRef ref){
	if(!ref.is_valid()){
		return;
	}
	runtime.level->destroy_queue.push_back(ref.index);
}
Vec3 Entity::get_forward_vector() {
	return get_physics().trans.get_forward_vector();
}
Vec3 Entity::get_right_vector(){
	return get_physics().trans.get_right_vector();
}
Vec3 Entity::get_up_vector() {
	return get_physics().trans.get_up_vector();
}
Vec3 Entity::get_location(){
	return get_physics().trans.trans.translation;
}
Quat Entity::get_rotation(){
	return get_physics().trans.trans.rotation;
}
void draw_call(std::function<void()>func ){
    try {runtime.level->draw_calls.push_back(func);} catch(std::exception e) {
	fputs("exception in draw call caught\n",stderr);
    }

}
std::vector<EntityRef> get_all_entities_with_tag(Tag tag){
    std::vector<EntityRef> out = {};
    for(uint32_t i =0; i<runtime.level->entities.size(); i++){
	Entity * e = runtime.level->entities[i].get();
	if(!e){
	    continue;
	} else if(e->has_tag(tag)){
	    out.push_back(EntityRef{.index = i, .generation = runtime.level->generations[i]});
	}
    }
    return out;
}
std::vector<EntityRef> get_all_entities_with_at_least_one_tag(Tag tags[], size_t count){
    std::vector<EntityRef> out = {};
    uint32_t tg = 0;
    for(int i =0; i<count; i++){
	tg |= tags[i];
    }
    for(uint32_t i =0; i<runtime.level->entities.size(); i++){
	Entity * e = runtime.level->entities[i].get();
	if(!e){
	    continue;
	} else if(e->has_tag((Tag)tg)){
	    out.push_back(EntityRef{.index = i, .generation = runtime.level->generations[i]});
	}
    }
    return out;
}
std::vector<EntityRef> get_all_entities_with_tag_set(Tag tags[], size_t count){
    std::vector<EntityRef> out = {}; 
    for(uint32_t i =0; i<runtime.level->entities.size(); i++){
	Entity * e = runtime.level->entities[i].get();
	if(!e){
	    continue;
	}
	else{
	    bool has = true;
	    for(int j =0; j<count; j++){
		if(!e->has_tag(tags[i])){
		    has = false;
		    break;
		}
	    }
	    if(has){
		out.push_back(EntityRef{.index = i, .generation = runtime.level->generations[i]});
	    }
	}
    }
    return out;
}

void apply_damage(EntityRef source, EntityRef target,Vec3 direction, double amount){
    Event event;
    event.cause_idx = source.index;
    event.cause_generation = source.generation;
    event.target_idx = target.index;
    event.target_generation = target.generation;
    event.EventType = EventType::ApplyDamage;
    event.apply_damage.damage = amount;
    event.apply_damage.direction = direction;
    runtime.level->event_queue.push_back(event);
}
}


