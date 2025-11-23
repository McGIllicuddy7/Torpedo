#include "ship.h"
#include "physics.h"
#include "level.h"
#include "base.h"

ShipComp * get_ship_comps(){
	return get_level()->components[SHIP_COMPS_IDX];
}
ProjectileComp * get_projectile_comps(){
	return get_level()->components[PROJECTILE_COMPS_IDX];
}
extern void update_ship(EntityRef ship);
extern void ai_update(EntityRef ship, ShipComp * s);
extern void human_update(EntityRef ship, ShipComp * s);
extern void update_projectile(EntityRef proj);
extern void update_weapons(ShipComp *s);
extern void ship_fire_machine_gun(EntityRef ship, ShipComp*s);
void ship_update(){
	EntityRefVec ships = get_all_entities_with_component(comp_ship);
	for(size_t i =0; i<ships.length; i++){
		update_ship(ships.items[i]);
	}
}
ShipComp * get_ship_comp(EntityRef ref){
	if(!entity_is_valid(ref)){
		return 0;
	}
	return &get_ship_comps()[ref.index];
}
void ship_handle_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage){
	if(damage>100.0){
		destroy_entity(target);
	}
}
void update_ship(EntityRef ship){
	ShipComp * s = get_ship_comp(ship);
	update_weapons(s);
	if(s->input.is_ai){
		ai_update(ship, s);
	}else{
		human_update(ship, s);
		PhysicsComp * phys = get_physics_comp(ship);
		Vec3 vel = phys->velocity;
		//draw_text(string_format(frame_arena(), "%f, %f, %f", vel.x, vel.y, vel.z).items, 900, 10, 30, WHITE);
	}
	if(s->input.input_mode == InputHuman){
		PhysicsComp * phys = get_physics_comp(ship);
		Matrix base = QuaternionToMatrix(Vec4_to_Vector4(phys->trans.trans.rotation));
		Vec3 i = s->input.rot_input;
		Matrix rot_matrix = MatrixMultiply(QuaternionToMatrix(QuaternionFromEuler(i.x, i.y, i.z)), base);
		phys->trans.trans.rotation = Vec4_from_Vector4(QuaternionFromMatrix(rot_matrix));
		Vec3 inp = Vec3_from_Vector3(Vector3Transform(Vec3_to_Vector3(s->input.input), rot_matrix));
		if(Vec3_len(phys->velocity)>0.0){
			float delta = floor(Vec3_dot_product(inp, phys->velocity))*0.8;
			phys->velocity = Vec3_add(
				phys->velocity, 
				Vec3_scale(Vec3_normalize(phys->velocity),delta));
		}

		phys->velocity = Vec3_add(phys->velocity, inp);
		if(Vec3_len(phys->velocity)>10.0){
			phys->velocity = Vec3_scale(Vec3_normalize(phys->velocity),10.0);
		}

	}else if(s->input.input_mode == InputMoveTo){
		todo();
	}else if(s->input.input_mode == InputAi){
		PhysicsComp * phys = get_physics_comp(ship);
		phys->velocity = Vec3_add(phys->velocity, Vec3_scale(s->input.input,2.0));
		if(Vec3_len(phys->velocity)>10.0){
			phys->velocity = Vec3_scale(Vec3_normalize(phys->velocity),10.0);
		}

	}
	else{
		todo();
	}

}

void human_update(EntityRef ship, ShipComp*s){
	Vec3 vel_inp = {0,0,0};
	Vector2 rot_inp = {0,0};
	double acc = s->acc;
	if(IsKeyDown(KEY_W)){
		vel_inp.x += acc;
	}
	if(IsKeyDown(KEY_S)){
		vel_inp.x -= acc;
	}
	if(IsKeyDown(KEY_A)){
		vel_inp.y += acc;
	}
	if(IsKeyDown(KEY_D)){
		vel_inp.y -= acc;
	}
	s->input.rot_input.x = 0;
	if(IsKeyDown(KEY_Q)){
		s->input.rot_input.x -= 0.005;	
	}
	if(IsKeyDown(KEY_E)){
		s->input.rot_input.x += 0.005;
	}
	if(IsKeyPressed(KEY_R)){
		get_level()->should_load= true;
		get_level()->load_name = "test.bin";
	}
	if(IsKeyPressed(KEY_T)){
		get_level()->should_save = true;
		get_level()->save_name = "test.bin";
	}
	if(IsKeyDown(KEY_SPACE)){
		ship_fire_machine_gun(ship, s);
	}
	draw_text(string_format(frame_arena(), "remaining ammo:%d", s->weapon_data.machine_gun_ammo).items, 100, 100, 20, WHITE);
	rot_inp = GetMouseDelta();
	s->input.input = vel_inp;
	s->input.rot_input.y = rot_inp.y*0.001;
	s->input.rot_input.z = -rot_inp.x*0.001;
	human_update_gui(ship, s);

}
EntityRef create_ship(Vec3 location, Vec3 angle, bool player){
    EntityRef out = create_entity();
    add_component(out, comp_physics);
    add_component(out, comp_model);
    add_component(out, comp_ship);
    PhysicsComp * phys = get_physics_comp(out);
    assert(phys);
    phys->is_valid = true;
    phys->mass = 1.0;
    phys->is_valid = true;
    phys->velocity = (Vec3){0,0,0};
    phys->trans.trans  = Trans_create();
    phys->trans.trans.translation = location;
    phys->trans.trans.rotation = Vec4_from_Vector4(QuaternionFromEuler(angle.x, angle.y, angle.z));
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
    mesh->meshes[0].string = "ship";
    mesh->mesh_count =1;
	mesh->lit = true;
	ShipComp * ship = get_ship_comp(out);
	memset(&(ship->input), 0, sizeof(ship->input));
	ship->acc = 0.01/60.0;
	ship->input.is_ai = !player;
	ship->input.mode = Rocket;
	ship->input.input_mode = InputHuman;
	ship->weapon_data.machine_gun_ammo = 1500;
	ship->weapon_data.machine_gun_cooldown  =1./20.;
	ship->weapon_data.machine_gun_remaining_time = 0.0;
	if(player){
		get_level()->player_entity = out;
		get_level()->cam_player_offset.translation.x +=0.405;
		get_level()->cam_player_offset.translation.z +=0.095;	
	}else{
		ship->ai_info.state = Patrol;
		ship->ai_info.home_base = location;
		ship->ai_info.move_to_point = location;
		ship->input.input_mode = InputAi;
	}
    return out;
}
bool has_view_to(EntityRef r, EntityRef k){
	return false;
}
void ship_serialize(Stream * s, void *ptr){
	ShipComp * p= ptr;
	for(int i =0; i<ENTITY_COUNT; i++){
		reflect_serialize(s, REFLECT(ShipComp, p+i));
	}
}
void ship_deserialize(Allocator al, Stream * s, void * ptr){
	ShipComp * p= ptr;
	for(int i =0; i<ENTITY_COUNT; i++){
		reflect_deserialize(al,s, REFLECT(ShipComp, p+i));
	}
}
ComponentHandler ship_handler = {.destructor = 0, .serialize = ship_serialize, .deserialize = ship_deserialize};
EntityRef fire_bullet(Vec3 pos, Vec3 direction, Quat rotation, Vec3 base_vel, EntityRef e){
    EntityRef out = create_entity();
    add_component(out, comp_physics);
    add_component(out, comp_model);
	add_component(out, comp_projectile);
    PhysicsComp * phys = get_physics_comp(out);
    assert(phys);
    phys->is_valid = true;
    phys->mass = 0.001;
    phys->is_valid = true;
    phys->velocity = Vec3_add(Vec3_scale(direction,5.0), base_vel);
	phys->velocity = Vec3_add(phys->velocity, Vec3_scale(random_vector(),0.01));
    phys->trans.trans  = Trans_create();
    phys->trans.trans.translation = pos;
    phys->trans.trans.rotation = rotation;
    phys->can_ever_collide = false;
    Collider col;
    col.bb.min=(Vector3){-0.01,- 0.005, -0.005};
    col.bb.max= (Vector3){0.01,0.005, 0.005};
    phys->colliders[0] = col;
    phys->collider_count = 1;
    phys->angular_velocity = (Vec3){0,0,0};
    phys->can_ever_collide = false;
    phys->destroy_on_impact = true;
    MeshComp * mesh = get_mesh_comp(out);
    mesh->meshes[0].color = BLUE;
    mesh->meshes[0].offset = Trans_create();
    mesh->meshes[0].string = "bullet";
    mesh->mesh_count =1;
	mesh->lit = false;
	ProjectileComp * proj = get_projectile_comp(out);
	proj->lifetime = 60.0;
	proj->parent = e;
    return out;
}
void projectile_serialize(Stream * s, void *ptr){
	ProjectileComp * p= ptr;
	for(int i =0; i<ENTITY_COUNT; i++){
		reflect_serialize(s, REFLECT(ProjectileComp, p+i));
	}
}
void projectile_deserialize(Allocator al, Stream * s, void * ptr){
	ProjectileComp * p= ptr;
	for(int i =0; i<ENTITY_COUNT; i++){
		reflect_deserialize(al,s, REFLECT(ProjectileComp, p+i));
	}
}
ComponentHandler projectile_handler = {.destructor = 0, .serialize =projectile_serialize, .deserialize = projectile_deserialize};
ProjectileComp * get_projectile_comp(EntityRef ref){
	if(!entity_is_valid(ref)){
		return 0;
	}
	return &get_projectile_comps()[ref.index];
}
void projectile_update(){
	EntityRefVec projes= get_all_entities_with_component(comp_projectile);
	for(size_t i =0; i<projes.length; i++){
		update_projectile(projes.items[i]);
	}
}
void update_projectile(EntityRef proj){
	ProjectileComp * p= get_projectile_comp(proj);
	p->lifetime-=GetFrameTime();
	if(p->lifetime<0){
		destroy_entity(proj);
	}else{
		Vec3 base = ent_get_location(proj);
		Vec3 next = Vec3_add(base,Vec3_normalize(ent_get_velocity(proj)));
		OptEntityRef e = line_trace(base, next, (uint32_t[]){proj.index, p->parent.index}, 1.0);
		if(e.is_valid){
			if(!entity_eq(e.ref, p->parent) && Vec3_dist(ent_get_location(proj), ent_get_location(e.ref))<1.0){
				apply_damage(proj, e.ref,ent_get_forward_vector(proj), 150);
				destroy_entity(proj);
			}
		}
	}
}
void update_weapons(ShipComp *s){
	s->weapon_data.machine_gun_remaining_time -= GetFrameTime();
	if(s->weapon_data.machine_gun_remaining_time <0.0){
		s->weapon_data.machine_gun_remaining_time  = 0.0;
	}
}
void ship_fire_machine_gun(EntityRef ship, ShipComp*s){
		if(s->weapon_data.machine_gun_remaining_time == 0.0 && s->weapon_data.machine_gun_ammo>0 ){
			Vec3 pos = Vec3_add(ent_get_location(ship), Vec3_scale(ent_get_forward_vector(ship), 0.5));
			Vec3 delt1 = Vec3_scale(ent_get_left_vector(ship), 0.1);
			Vec3 delt2 = Vec3_scale(ent_get_left_vector(ship), -0.1);
			fire_bullet(Vec3_add(pos,delt1),ent_get_forward_vector(ship), ent_get_orientation(ship), ent_get_velocity(ship),ship);	
			fire_bullet(Vec3_add(pos,delt2),ent_get_forward_vector(ship), ent_get_orientation(ship), ent_get_velocity(ship),ship);
			s->weapon_data.machine_gun_remaining_time = s->weapon_data.machine_gun_cooldown;
			s->weapon_data.machine_gun_ammo -= 1;
		}
}
void human_update_gui(EntityRef ref, ShipComp * ship){
	double trace_rad = 1000.0;
	Vec3 loc = ent_get_location(ref);
	EntityRefVec entities = sphere_trace(frame_arena(), loc, trace_rad, (uint32_t[]){}, 0);
	Vec3 forward = ent_get_forward_vector(ref);
	Vec3 left = ent_get_left_vector(ref);
	double cx = GetScreenWidth()/2.0;
	double cy = (GetScreenHeight()*7.5)/10.0;
	double radius = 150.0;
	draw_circle(cx, cy, radius, DARKGREEN);
	for(int i =0; i<entities.length; i++){
		Vec3 v = ent_get_location(entities.items[i]);
		Vec3 dv = Vec3_sub(v, loc);
		double y= -Vec3_dot_product(forward, dv)*radius/trace_rad;
		double x = -Vec3_dot_product(left, dv)*radius/trace_rad;
		if(has_component(entities.items[i], comp_ship)){
			draw_circle(cx+x, cy+y, 3.0,GREEN);
		}else if(has_component(entities.items[i], comp_projectile)){
			draw_circle(cx+x, cy+y, 0.5,GREEN);
		}		
	}	
}

void i_see_the_tv_glow(Camera3D cam, Vec3 base){
    EntityRefVec ships = get_all_entities_with_component(comp_ship);
	Vec3 dir = Vec3_from_Vector3(cam.target);
	for(int i =0; i<ships.length; i++){
		Vec3 pos = Vec3_sub(ent_get_location(ships.items[i]), base);
		double dist = sqrt(Vec3_len(pos));
		 Vec3 p = Vec3_normalize(pos);
		double d1 =(-Vec3_dot_product(dir,p)+1.5)/2.0;
		if(d1>0.0){
			DrawSphere(Vec3_to_Vector3(pos), dist*d1/5.0, (Color){130,130, 200, 100});
		}
	}
}