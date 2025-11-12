#include "ship.h"
#include "physics.h"
extern AiState what_is_to_be_done(EntityRef ref, ShipComp * s,AiState state);
extern AiState handle_patrol(EntityRef ref, ShipComp * s,AiState state);
extern AiState handle_skirmish(EntityRef ref, ShipComp * s,AiState state);
extern AiState handle_direct_attack(EntityRef ref, ShipComp * s,AiState state);
extern AiState handle_search_for(EntityRef ref, ShipComp * s,AiState state);
extern void ai_handle_movement(EntityRef ref, ShipComp * s, AiState state);
extern OptEntityRef can_see_priority_enemy(EntityRef ref ,ShipComp *s);
extern void ship_fire_machine_gun(EntityRef ship, ShipComp*s);
extern bool ai_should_attack(EntityRef ref, ShipComp * s);
Vec3 next_impulse(Vec3 pos, Vec3 end, Vec3 vel,Vec3 des_vel, double acc){
	//A(t) = at+ b
	//V(t) = 1/2at^2+ bt + v_0
	//X(t) = 1/6at^3 + 1/2bt^2 + v_0t + x_0
	//X(0) =  x_0
	//X(T) = x_1 = 1/6at^3 + 1/2bt^2 + v_0t + x_0
	//V(T) =  0 = 1/2aT^2 + bT + v_0
	//T = -b + \sqrt{b^2-2av_0}/a
	//1/6at^3 + 1/2bt^2 + v_0t + x_0-x_1 = 0
	Vec3 a ={0,0,0};
	Vec3 b ={0,0,0};
	double t = Vec3_dist(pos, end)/(acc*acc);
	int hit = 0;
retry:
	for(int i =0; i<512; i++){
		const double x_x = 1./6. * a.x*t*t*t + 0.5 *b.x*t*t + vel.x* t + pos.x-end.x;
		const double x_y = 1./6. * a.y*t*t*t + 0.5 *b.y*t*t + vel.y* t + pos.y-end.y;
		const double x_z = 1./6. * a.z*t*t*t + 0.5 *b.z*t*t + vel.z* t + pos.z-end.z;	
		const double dx_xda = 1./6.*t*t*t;
		const double dx_yda = 1./6.*t*t*t;
		const double dx_zda = 1./6.*t*t*t;
		a.x -= x_x /(dx_xda)*0.1;
		a.y -= x_y /(dx_yda)*0.1;
		a.z -= x_z /(dx_zda)*0.1;
		const double dx_xdb = 0.5*t*t;
		const double dx_ydb = 0.5*t*t;
		const double dx_zdb = 0.5*t*t;
		b.x -= x_x/(dx_xdb)*0.1;
		b.y -= x_y/(dx_ydb)*0.1;
		b.z -= x_z/(dx_zdb)*0.1;	
	}
	for(int i =0; i<1024; i++){
		const double v_x =  0.5 * a.x* t*t +b.x*t + vel.x-des_vel.x; 
		const double v_y =  0.5 * a.y* t*t +b.y*t + vel.y-des_vel.y;
		const double v_z =  0.5 * a.z* t*t +b.z*t + vel.z-des_vel.z;
		const double dv_xdb = t;
		const double dv_ydb = t;
		const double dv_zdb = t;
		const double da_xdb = 1./3. *t;
		const double da_ydb = 1./3. *t;
		const double da_zdb = 1./3. *t;
		const double db_x = v_x/(dv_xdb)*0.1;
		const double db_y = v_y/(dv_ydb)*0.1;
		const double db_z = v_z/(dv_zdb)*0.1;
		b.x+= db_x;
		b.y+= db_y;
		b.z += db_z;
		a.x -= db_x /da_xdb;
		a.y -= db_y/da_ydb;
		a.z-= db_z/da_zdb;	
	}
	double max_acc = Vec3_len(b);
	double ac2 = Vec3_len(Vec3_add(Vec3_scale(a, t),b));
	if(ac2>max_acc){
		max_acc = ac2;
	}
	if(max_acc>acc){
		t*= 1.1;
		goto retry;
	}	
	if(max_acc<acc/1.1&& hit<5){
		hit += 1;
		t /= 1.1;
		goto retry;
	}
	return b;
}


void ai_update(EntityRef ship, ShipComp * s){
	s->ai_info.heart_beat-= GetFrameTime();
	if(s->ai_info.heart_beat<=0.0){
		s->ai_info.heart_beat = 0.0;
	}
	s->ai_info.state = what_is_to_be_done(ship, s, s->ai_info.state);
	ai_handle_movement(ship, s, s->ai_info.state);
	if(s->ai_info.heart_beat<=0.0){
		s->ai_info.heart_beat = 0.8;
	}
}
AiState what_is_to_be_done(EntityRef ref,ShipComp *s, AiState state){
	switch(state){
		case Patrol:{
			return handle_patrol(ref, s, state);
			break;
		}
		case Skirmish:{
			return handle_skirmish(ref, s, state);
			break;
		}
		case SearchFor:{
			return handle_search_for(ref, s, state);
			break;
		}
		case DirectAttack:{
			return handle_direct_attack(ref, s, state);
			break;
		}
	}	
}
AiState handle_patrol(EntityRef ref, ShipComp * s,AiState state){
	AiInfo * info = &s->ai_info;
	Vec3 loc = ent_get_location(ref);	
	if(Vec3_dist(loc, info->move_to_point)<1.0){
		info->move_to_point = Vec3_add(Vec3_scale(random_vector(), 100), info->home_base);
	}	
	if(s->ai_info.heart_beat == 0.0){
		OptEntityRef e = can_see_priority_enemy(ref,s);
		if(e.is_valid){
			s->target = e.ref;	
			s->ai_info.target_dir = ent_get_location(e.ref);
			return Skirmish;
		}else{
			s->target = (EntityRef){0,0};
			s->ai_info.target_dir = ent_get_velocity(ref);
		}
	}
	return Patrol;
}
AiState handle_skirmish(EntityRef ref, ShipComp * s,AiState state){
	if(!entity_is_valid(s->target)){
		s->ai_info.target_dir = ent_get_location(ref);
		return Patrol;
	}
	AiInfo * info = &s->ai_info;
	s->ai_info.target_dir = approximate_target_vector(
		ent_get_location(ref), 
		ent_get_location(s->target),
		ent_get_velocity(ref), 
		ent_get_velocity(s->target), 40.0
	);

	if(ai_should_attack(ref, s)){
		ship_fire_machine_gun(ref, s);
	}
	Vec3 loc = ent_get_location(ref);
		if(Vec3_dist(loc, info->move_to_point)<1.0){
		info->move_to_point = Vec3_add(Vec3_scale(random_vector(), 100), loc);
	}	
	if(s->ai_info.heart_beat == 0.0){
		OptEntityRef e = can_see_priority_enemy(ref,s);
		if(e.is_valid){
			s->target = e.ref;
			if(Vec3_dist(ent_get_location(e.ref), ent_get_location(ref))<100.0){
				return DirectAttack;
			}
			return Skirmish;
		}else{
			s->target = (EntityRef){0,0};
			return Patrol;
		}
	}	
	return Skirmish;
}
AiState handle_direct_attack(EntityRef ref, ShipComp * s,AiState state){
	if(!entity_is_valid(s->target)){
		s->ai_info.target_dir = ent_get_location(ref);
		return Patrol;
	}
	AiInfo * info = &s->ai_info;
	if(ai_should_attack(ref, s)){
		ship_fire_machine_gun(ref, s);
	}
	Vec3 loc = ent_get_location(ref);
	s->ai_info.target_dir = approximate_target_vector(
		ent_get_location(ref), 
		ent_get_location(s->target),
		ent_get_velocity(ref), 
		ent_get_velocity(s->target), 40.0
	);
	info->move_to_point = Vec3_add(ent_get_location(s->target), Vec3_scale(ent_get_forward_vector(s->target), -5.0));
	if(s->ai_info.heart_beat == 0.0){
		OptEntityRef e = can_see_priority_enemy(ref,s);
		if(e.is_valid){
			s->target = e.ref;
			if(Vec3_dist(ent_get_location(e.ref), ent_get_location(ref))<100.0){
				return DirectAttack;
			}
			return Skirmish;
		}else{
			s->target = (EntityRef){0,0};
			return Patrol;
		}
	}	
	return DirectAttack;
}
AiState handle_search_for(EntityRef ref, ShipComp * s,AiState state){
	AiInfo * info = &s->ai_info;
	todo();
	return SearchFor;
}
extern void ai_handle_movement(EntityRef ref, ShipComp * s, AiState state){
	Vec3 vel = ent_get_velocity(ref);
	Vec3 pos = ent_get_location(ref);
	Vec3 future_pos= Vec3_add(pos, Vec3_scale(vel, 5.0));	
	bool hit = line_trace(pos, future_pos, (u32[]){ref.index}, 1).is_valid;
	if(hit || s->ai_info.panic_time>0.0){	
		if(hit){
			if(s->ai_info.panic_time == 0.0){
				Vector3 p = Vec3_to_Vector3(ent_get_left_vector(ref));
				Vector3 d = Vec3_to_Vector3(ent_get_forward_vector(ref));
				double r = ((double)(rand()%1000))*2*PI/(1000);
				Vec3 dir = Vec3_from_Vector3(Vector3RotateByAxisAngle(p, d, r));	
				dir = Vec3_scale(Vec3_normalize(dir), 0.2);
				s->input.input = dir;
			}
			s->ai_info.panic_time += 2;

		}else{
			s->ai_info.panic_time-=GetFrameTime();
			if(s->ai_info.panic_time<0.0){
				s->ai_info.panic_time = 0.0;
			}
		}
	}else{
		s->input.input = next_impulse(pos, s->ai_info.move_to_point, vel, (Vec3){0,0,0}, s->acc*2);
	}
	get_physics_comp(ref)->trans.trans.rotation = rotate_toward_vector_smol(ref, s->ai_info.target_dir);
}

OptEntityRef can_see_priority_enemy(EntityRef ref ,ShipComp *s){
	EntityRefVec entities = sphere_trace(frame_arena(), ent_get_location(ref), 100.0, (uint32_t []){ref.index}, 1);
	for(size_t i=0; i<entities.length; i++){
		EntityRef e = entities.items[i];
		if(has_component(e, comp_ship)){
			OptEntityRef e2 = line_trace(ent_get_location(ref), ent_get_location(e), (uint32_t[]){ref.index, e.index}, 2);
			if(e2.is_valid){
				continue;
			}
			if(entity_eq(ref, e)){
				continue;
			}
			return (OptEntityRef){.is_valid = true, .ref = e};
		}
	}
	return (OptEntityRef){0};
}
bool ai_should_attack(EntityRef ref, ShipComp * s){
	if(!entity_is_valid(s->target)){
		return false;
	}
	Vec3 tloc = ent_get_location(s->target);
	Vec3 sloc = ent_get_location(ref);
	Vec3 dv = Vec3_sub(tloc, sloc);
	Vec3 nv = Vec3_normalize(dv);
	return Vec3_dot_product(ent_get_forward_vector(ref),nv)>0.98;
}
Vector3 QuaternionForwardVector(Quaternion q){
	Vector3  out = {1,0,0};
	Matrix m = QuaternionToMatrix(q);
	return Vector3Transform(out, m);
}
Quat rotate_toward_vector_smol(EntityRef r, Vec3 target){
	Quaternion out = Vec4_to_Vector4(ent_get_orientation(r));
	Quaternion base = out;
	Vector3 t = Vec3_to_Vector3(target);
	t = Vector3Normalize(t);
	float min = 0.0; 
	if(Vec3_len(target)<0.01){
		return Vec4_from_Vector4(out);
	}
	min = Vector3DotProduct(t, QuaternionForwardVector(out));
	for(int x = -3; x<4; x++){
		for(int y = -3; y<4; y++){
			for(int z = -3; z<4; z++){
				float dx=x;
				float dy = y;
				float dz = z;
				dx*= 0.01; dy*= 0.01; dz*= 0.01;
				Quaternion q = QuaternionFromEuler(dx,dy,dz);
				q = QuaternionMultiply(q, base);
				Vector3 v = QuaternionForwardVector(q);
				float f = Vector3DotProduct(v, t);
				if(f>min){
					out = q;
					min = f;
				}
			}
		}
	}
	return Vec4_from_Vector4(out);

}
Vec3 approximate_target_vector(Vec3 pos, Vec3 target_pos, Vec3 vel,Vec3 target_vel, double speed){
	Vec3 rel_vel = Vec3_sub(target_vel, vel);
	Vec3 rel_pos = Vec3_sub(target_pos, pos);
	double impact_time = Vec3_len(rel_pos)/speed;
	Vec3 offset = Vec3_scale(random_vector(), 0.1*impact_time);
	return Vec3_add(Vec3_add(rel_pos, Vec3_scale(rel_vel, impact_time)), offset);
}
