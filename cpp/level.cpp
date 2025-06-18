#include "level.hpp"
#include "renderer/renderer.hpp"
#include "physics/physics.hpp"
#include "shaders.hpp"
#include "ship/ship.hpp"
namespace Torpedo{
Runtime runtime;
void update();
void run_destructors();
void set_player_entity(EntityRef ref){
	runtime.level->player = ref.get();
}

void update(){
    for(int i =0; i<runtime.level->entities.size(); i++){
        if(runtime.level->entities[i]){
            runtime.level->entities[i]->on_tick();
        }
    }
}

void cam_update(Camera * cam){
	if (runtime.level->player){
		cam->position = runtime.level->player->get_physics().trans.trans.translation;
		cam->target = runtime.level->player->get_forward_vector();
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
    while(!WindowShouldClose()){
	update();
        physics_prepare_update();
        update_physics();
	cam_update(&cam);
        renderer_update(&cam);
        physics_finish_update();
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

void Entity::apply_damage(const char * comp,double damage){
    
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
	size_t index = runtime.level->destroy_queue.size()-1;
	do {
		size_t idx = runtime.level->destroy_queue[index];
		Entity *ptr = runtime.level->entities[idx].get();
		if(runtime.level->player == ptr){
			runtime.level->player = 0;
		}
		runtime.level->meshes[idx].reset();
		runtime.level->physics[idx].reset();
		runtime.level->physics[idx].is_valid = false;
		runtime.level->entities[idx] = 0;
		index -=1;
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
    Shader shader = LoadShader("shaders/vertex.glsl", "shaders/frag.glsl");
    get_level().models[string("cube")].materials[0].shader = shader;
    EntityRef player =create_player_ship(Vec3{100,0,0}, Quat{0,0,0,1});
    #ifdef MULT
    int count =4;
    for(int x = -count; x<count+1; x++){
        for(int y = -count; y<count+1; y++){
            for(int z = -count; z<count+1; z++){
                Vec3 point = Vec3{(double)x,(double)y,(double)z}*8;
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
    e.get()->get_mesh().meshes["base"] = m;
    PhysicsComp phys = {0};
    phys.mass = 1.0;
    phys.is_valid = true;
    phys.trans.trans = Trans::create();
    phys.trans.trans.translation = location;
    phys.angular_velocity = Quat::from(QuaternionFromEuler(angular.x, angular.y, angular.z));
    Collider col;
    col.offset= Trans::create();
    Vec3 mscale = Vec3{-scale.x, -scale.y, -scale.z};
    col.bb = BoundingBox{mscale/2,scale/2};
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

}


