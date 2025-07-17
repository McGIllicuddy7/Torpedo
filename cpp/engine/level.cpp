#include "level.hpp"
#include "renderer/renderer.hpp"
#include "physics/physics.hpp"
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
        if(runtime.level->entities[i]){
            runtime.level->entities[i]->on_tick();
        }
    }
}
void process_events(){
    for(int i =0; i<runtime.level->event_queue.size(); i++){
	Event e = runtime.level->event_queue[i];
	if(e.event_type==EventType::ApplyDamage){
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
//	printf("%f,%f,%f\n", cam->up.x, cam->up.y, cam->up.z);
	}
	else {
    		UpdateCamera(cam, CAMERA_FREE);
	}
}
void handle_loading_and_saving(){
    if(get_level().should_save){
	Serializer s;
	s.serialize_interface(runtime.level.get());
	s.write_to_file(get_level().save_name.c_str());
    }
    if(get_level().should_load){
	FILE *f = fopen(get_level().load_name.c_str(),"r");
	if(!f){
	    goto done;
	}
	fclose(f);
	Deserializer d = Deserializer::from_file(get_level().load_name.c_str());	
	runtime.level.reset(new Level(d.deserialize<Level>()));	
    }
done:
    get_level().should_load = false;
    get_level().should_save = false;
    get_level().load_name = "";
    get_level().save_name=  ""; 
}
void mainloop(std::function<void()> func){
    InitWindow(GetScreenWidth(), GetScreenHeight(), "brid-get");
    DisableCursor();
    setup();
    load_level_fn(func); 
    Camera *cam= &get_level().cam;
    cam->up = {0,0,1};
    cam->target = {-1, 0,0};
    cam->fovy = 100;
    cam->position = {20,0,0};
    cam->projection = CAMERA_PERSPECTIVE; 

    SetTargetFPS(60);
    RenderTexture2D post_texture = LoadRenderTexture(GetScreenWidth(),GetScreenHeight());
    Shader post_shader = LoadShader("./shaders/vertex.glsl", "./shaders/postfrag.glsl");
    while(!WindowShouldClose()){
	cam = &get_level().cam;
	runtime.level->draw_calls.clear();
	runtime.level->draw_calls_3d.clear();
	runtime.level->event_queue.clear();
	update();
        physics_prepare_update();
        update_physics();
	cam_update(cam);
        renderer_update(cam,post_texture, post_shader);
        physics_finish_update();
	process_events();
	run_destructors();
	handle_loading_and_saving();
    }  
    CloseWindow();
    runtime.level = 0;
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
    destroy_entity(get_as_ref(this));    
}

void Entity::set_velocity(Vec3 vel){
}

Vec3 Entity::get_velocity(){
    return Vec3{0,0,0};
}
void Entity::serialize(Serializer * ser) const{
    ser->serialize("Entity");
    ser->serialize(id);
    ser->serialize(tags);
}
Entity Entity::deserialize(Deserializer* des){
    Entity out;
    des->deserialize<std::string>();
    out.id = des->deserialize<uint32_t>();
    out.tags = des->deserialize<Tag>();
    return out;
}
Entity * Entity::interface_deserialize(Deserializer&des){
    return new Entity (Entity::deserialize(&des));
}

void setup(){
    runtime.level = std::make_unique<Level>(Level{});
    runtime.level->textures["../assets/ship_texture.png"] =LoadTexture("../assets/ship_texture.png");
    runtime.level->player = 0;
    Shader shader = LoadShader("shaders/vertex.glsl", "shaders/frag.glsl");
    get_level().models["cube"] = path_load_model("cube",std::vector<std::string>{}, runtime.level->textures,shader);
    get_level().models["cylinder"] = path_load_model("cylinder",std::vector<std::string>{}, runtime.level->textures,shader);
    get_level().models[string("ship")] =path_load_model("ship", std::vector<std::string>{"../assets/ship_texture.png"}, runtime.level->textures,shader);// LoadModel("../assets/ship.glb");
    get_level().mesh_textures[string("ship")]= std::vector<std::string>{std::string("../assets/ship_texture.png")}; 
    runtime.level->shader = shader;

}
void run_destructors(){
	if(!runtime.level->destroy_queue.size()){
		return;
	}	
	size_t index = runtime.level->destroy_queue.size();
	if(index == 0){
	    return;
	}
	do {
		index -=1;
		size_t idx = runtime.level->destroy_queue[index];
		if(!runtime.level->entities[idx]){	
		    continue;
		}
		Entity *ptr = runtime.level->entities[idx];
		if(!ptr){ 
		    continue;
		}
		delete ptr;
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

void load_level_fn(std::function<void()>func){ 
    func();

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

void draw_call_3d(std::function<void()>to_call){
    try {runtime.level->draw_calls_3d.push_back(to_call);} catch(std::exception e) {
	fputs("exception in draw call caught\n",stderr);
    }
}
std::vector<EntityRef> get_all_entities_with_tag(Tag tag){
    std::vector<EntityRef> out = {};
    for(uint32_t i =0; i<runtime.level->entities.size(); i++){
	Entity * e = runtime.level->entities[i];
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
	Entity * e = runtime.level->entities[i];
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
	Entity * e = runtime.level->entities[i];
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
    event.event_type = EventType::ApplyDamage;
    event.apply_damage.damage = amount;
    event.apply_damage.direction = direction;
    runtime.level->event_queue.push_back(event);
}
void Level::serialize(Serializer * ser)const{
    ser->serialize((std::string)"Level");
//    printf("ser partial:%zu\n", ser->get_current_idx());
    if(player){
 //           printf("ser player idx:%ld\n", (long)player->id);
	ser->serialize<long>((long)player->id);
    } else{
	ser->serialize<long>((long)(-1));
//        printf("ser player idx:%ld\n", -1l);
    } 

//    printf("ser stage 1:%zu\n", ser->get_current_idx());
 //   fflush(stdout);

    ser->serialize(textures.size());
    for(const auto &i:textures){
	ser->serialize(i.first);
    }
  //  printf("ser stage 2:%zu\n", ser->get_current_idx()); 
   // fflush(stdout);
    ser->serialize(mesh_textures.size());
    for(auto &i: mesh_textures){
	ser->serialize(i.first);
	ser->serialize_array(i.second.data(), i.second.size());
    }
   // printf("ser stage 3:%zu\n", ser->get_current_idx()); 
    //fflush(stdout);
    ser->serialize(models.size()); 
    //printf("model_count:%zu\n", models.size());
    for(const auto&i : models){
	ser->serialize(i.first);
    }
   //  printf("ser stage 4:%zu\n", ser->get_current_idx()); 
    //fflush(stdout);
    ser->serialize_interface_array(entities.data(), entities.size());
     //printf("ser stage 5:%zu\n", ser->get_current_idx()); 
    //fflush(stdout);
    ser->serialize_array(generations.data(), generations.size());
     //printf("ser stage 6:%zu\n", ser->get_current_idx()); 
    //fflush(stdout);
    ser->serialize_array(meshes.data(), meshes.size());
     //printf("ser stage 7:%zu\n", ser->get_current_idx()); 
    //fflush(stdout);
    ser->serialize_array(physics.data(), physics.size());
    ser->serialize(cam);
   // fflush(stdout);
}
Level Level::deserialize(Deserializer* des){
    printf("deserializing\n");
    Level out;
    des->deserialize<std::string>();
    std::string t = des->deserialize<std::string>();
    //printf("des partial:%zu, s:%s\n",des->get_current_idx(), t.c_str());
    out.shader = LoadShader("shaders/vertex.glsl", "shaders/frag.glsl");
    long player_idx = des->deserialize<long>();
    //printf("des player idx:%ld\n", player_idx);
     //printf("des stage 1:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    size_t texture_size = des->deserialize<size_t>(); 
    for(size_t i =0; i<texture_size; i++){
	std::string name = des->deserialize<std::string>();
	Texture tex = LoadTexture(name.c_str());
	 out.textures.insert({name, tex});
    }  
    //printf("des stage 2:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    size_t mesh_texture_size = des->deserialize<size_t>(); 
    for(size_t i =0; i<mesh_texture_size; i++){
	std::string name = des->deserialize<std::string>();
	std::vector<std::string> values = des->deserialize_array<std::string>();
	out.mesh_textures.insert({name, values});
    }
    //printf("des stage 3:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    size_t model_size = des->deserialize<size_t>();
    //printf("model_count:%zu\n", model_size);
    for(size_t i =0; i<model_size; i++){
	std::string name = des->deserialize<std::string>();
	Model m = path_load_model(name, out.mesh_textures[name], out.textures,out.shader);
	out.models.insert({name, m});
    }
   // printf("des stage 4:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    out.entities = des->deserialize_interface_array<Entity>(); 
    if(player_idx>=0&& player_idx<out.entities.size()){
	out.player = out.entities[player_idx];
    }else{
	out.player =0;
    }
    //printf("des stage 5:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    out.generations=  des->deserialize_array<uint32_t>();
    //printf("des stage 6:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    out.meshes = des->deserialize_array<MeshComp>();
    //printf("des stage 7:%zu\n", des->get_current_idx()); 
    //fflush(stdout);
    out.physics = des->deserialize_array<PhysicsComp>();
    //printf("des stage 8:%zu\n", des->get_current_idx()); 
    out.cam = des->deserialize<Camera3D>();
    //fflush(stdout);
    return out;
}
Level * Level::interface_deserialize(Deserializer&des){
    return new Level(Level::deserialize(&des));
}
Level::~Level(){
    for(auto & i:models){
	    UnloadModel(i.second);
    }
    for(auto &i:textures){
	    UnloadTexture(i.second);
    }
    UnloadShader(shader);
}
void load_level(const char* path){
    get_level().should_load = true;
    get_level().load_name = path;
}
void save_level(const char* path){
    get_level().should_save = true;
    get_level().save_name = path;
}
Model path_load_model(const std::string&mod, const std::vector<std::string>& textures, unordered_map<string,Texture> & loaded_textures, Shader shader){
    Model out;
    if(mod == "../assets/cube.glb"|| mod == "cube"){
	out = LoadModelFromMesh(GenMeshCube(0.5, 0.5, 0.5)); 
    }else if(mod == "../assets/cylinder.glb"||mod == "cylinder"){
	out =LoadModelFromMesh(GenMeshCube(0.2,0.02, 0.02));	
    } else{
	string name = std::string("../assets/")+mod+".glb";
	printf("loading %s\n", name.c_str());
	out = LoadModel(name.c_str());
    }
    for(size_t i =0; i<textures.size(); i++){
	if(!loaded_textures.contains(textures[i])){
	    loaded_textures[textures[i]] = LoadTexture(textures[i].c_str());
	}
	out.materials[0].maps->texture=loaded_textures[textures[i]];
	printf("%s\n", textures[i].c_str());
    }
    out.materials[0].shader = shader;
    return out;
}

}
