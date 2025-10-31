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
			return Skirmish;
		}else{
			s->target = (EntityRef){0,0};
		}
	}
	return Patrol;
}
AiState handle_skirmish(EntityRef ref, ShipComp * s,AiState state){
	AiInfo * info = &s->ai_info;
	ship_fire_machine_gun(ref, s);
	Vec3 loc = ent_get_location(ref);
	if(!entity_is_valid(s->target)){
		return Patrol;
	}
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
	AiInfo * info = &s->ai_info;
	ship_fire_machine_gun(ref, s);
	Vec3 loc = ent_get_location(ref);
	if(!entity_is_valid(s->target)){
		return Patrol;
	}
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
	draw_sphere(s->ai_info.move_to_point, 0.1, GREEN);
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
		s->input.input = next_impulse(pos, s->ai_info.move_to_point, vel, (Vec3){0,0,0}, s->acc*8);
	}
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