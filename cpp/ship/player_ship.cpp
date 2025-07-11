#include "ship.hpp"
#include "../physics/physics.hpp"
#include "../particles.hpp"
namespace Torpedo{

void PlayerShip::on_tick(){	
	ship.parent = get_as_ref(this);
	Vector2 imp = GetMouseDelta();
	ship.rotation_input = QuaternionFromEuler(get_input_axis(KEY_Q, KEY_E)*0.01,imp.y*0.001, -imp.x*0.001);
	ship.movement_input = Vec3{
		get_input_axis(KEY_S, KEY_W),
		get_input_axis(KEY_D, KEY_A),
		get_input_axis(KEY_LEFT_SHIFT, KEY_Z),
	};
	if(IsKeyPressed(KEY_Z)){
		ship.stablized_velocity = !ship.stablized_velocity;
	}
	if(IsKeyPressed(KEY_SPACE)){
		Vector3 r = get_right_vector();
		r*= 0.2;
		Vec3 lv= Vec3::from(r);
		Vec3 rv = Vec3::from(r*-1.0);
		ship.weapons.fire_projectile(get_location()+get_forward_vector()+lv, get_forward_vector(),get_rotation());
		ship.weapons.fire_projectile(get_location()+get_forward_vector()+rv, get_forward_vector(),get_rotation());
	}
	if(IsKeyPressed(KEY_C)){
		/*Vector3 r = get_right_vector();
		r*= 0.2;
		Vec3 lv= Vec3::from(r);
		Vec3 rv = Vec3::from(r*-1.0);
		ship.weapons.fire_projectile(get_location()+get_forward_vector()+lv, get_forward_vector(),get_rotation());
		ship.weapons.fire_projectile(get_location()+get_forward_vector()+rv, get_forward_vector(),get_rotation());*/
		Vec3 r = get_right_vector()*0.23;
		Vec3 start =get_location()+get_forward_vector()+r;
		Vec3 end =get_location()+get_forward_vector()*1000.0+r;	
		spawn_repeating(1.0,[start,end](){draw_call_3d([start,end](){	
			DrawCapsule(start, end, 0.01, 12, 12, RED);
		});});
		auto c = line_trace( start,end,{id});
		if(c){
			apply_damage(get_as_ref(this),*c,get_forward_vector(), 10);
		}
		start =get_location()+get_forward_vector()-r*2.0;
		end =get_location()+get_forward_vector()*1000.0-r*2.0;	
		spawn_repeating(1.0,[start,end](){draw_call_3d([start,end](){	
			DrawCapsule(start, end, 0.01, 12, 12, RED);
		});});
		c = line_trace( start,end,{id});
		if(c){
			apply_damage(get_as_ref(this),*c,get_forward_vector(), 10);
		}
	}
	ship.update();	
	Vector2 center; 
	center.x = (float)GetScreenWidth()/2.0;
	center.y =(float)GetScreenHeight()/2.0;
	EntityRef min;
	bool hit_min = false;
	double min_dist = Vector2LengthSqr(center);
	bool check = IsKeyPressed(KEY_M);
	std::vector<EntityRef> with_tag = get_all_entities_with_at_least_one_tag((Tag[]){tag_ship},1);
	for(auto a : with_tag	){
		Vec3 v = a.get()->get_location()-get_location();
		Vector3 target =get_forward_vector();
		Camera3D cam;
		cam.up = get_up_vector();
		cam.target = target;
		cam.position = {0,0,0};
		cam.projection =CAMERA_PERSPECTIVE;
		cam.fovy = 120;
		Vector2 p = GetWorldToScreen(v,cam);
		double d = Vector3Distance(v, get_location());
		if(d<1){d = 1;}
		if(d>5000){
			continue;
		}	
		if(a.index == this->id){
			continue;
		}
		if(Vector3DotProduct(Vector3Normalize(v-get_location()),target)<0.0){
				continue;
		}
		draw_call([p,d](){
			double r = 500/d;
			if(r<10){
				r = 10;
			}
			DrawCircleLinesV(p, r,{255,0,0,255});
		});
		if(check){
		Aligned * align_ptr = a.downcast<Aligned>();
		if(align_ptr){
			if(align_ptr->get_alignment() == Alignment::PlayerAligned){
				continue;
			}
			double dist = Vector2Distance(center, p);
			if(dist<min_dist){
				min_dist = dist;
				min = a;
				hit_min = true;
			}
		}
		}
	}
	if(check){
		ship.weapons.fire_missile(get_location()+get_forward_vector(), get_forward_vector(), get_rotation(), min, hit_min);
	}	
}
PlayerShip::PlayerShip(){
	ship = ShipComp{};
}
Alignment PlayerShip::get_alignment(){
	return Alignment::PlayerAligned;
}
EntityRef create_player_ship(Vec3 pos, Quat rot){
	EntityRef out = create_entity<PlayerShip>();
	out.get()->add_tag(tag_ship);
	out.get()->add_tag(tag_movable);
	out.get()->add_tag(tag_pressurized);
	set_player_entity(out);
	out.get()->get_physics().trans.trans.translation = pos;
	out.get()->get_physics().trans.trans.rotation = rot;
	out.get()->get_physics().destroy_on_impact = false;
	out.get()->get_physics().is_valid = true;
	out.get()->get_physics().colliders = std::vector<Collider>{Collider{Trans::create(),  BoundingBox{Vec3{-1.91526,-0.309, -0.309}/2.0, Vec3{1.0067,0.309, 0.309}/2.0}}};
	MeshPart msh;
	msh.color = Color{0,0,0,0};
	msh.offset= Trans::create();
	//-0.257, 0, -0.0615
	msh.offset.translation = Vec3{-0.540*0.75,0,-0.130*0.75};
	msh.offset.scale = Vec3{1,1,1};
	msh.string = "ship";
	out.get()->get_mesh().meshes = std::unordered_map<std::string, MeshPart>{{"mesh", msh}};
	return out;
}
void PlayerShip::on_damage(Vec3 incoming_direction, double damage){
	destroy_entity(get_as_ref(this));
}
}

