#include "ship.hpp"
#include "../../engine/physics/physics.hpp"
namespace Torpedo{
	void WeaponsComp::fire_projectile(Vec3 start, Vec3 direction, Vec3 base_vel,Quat rot, bool spawned_by_player){
		MeshPart m;
		m.string = "cylinder";
		m.offset= Trans::create();
		m.color = RED;
		EntityRef e = create_entity<Projectile>();
		e.get()->add_tag(tag_movable);
		e.get()->get_mesh().meshes["base"] = m;
		PhysicsComp phys = {0};
		phys.mass = 1.0;
		phys.is_valid = true;
		phys.trans.trans = Trans::create();
		phys.trans.trans.translation = start;
		phys.angular_velocity = Vec3{0,0,0};
		phys.trans.trans.rotation =rot;
		phys.destroy_on_impact = true;
		Collider col;
		col.offset= Trans::create();
		Vec3 scale = Vec3{0.1, 0.01, 0.01};
		Vec3 mscale = Vec3{-scale.x, -scale.y, -scale.z};
    //col.bb = BoundingBox{Vec3{-1.91526,-0.309, -0.309}/2.0, Vec3{1.0067,0.309, 0.309}/2.0};
		col.bb = BoundingBox{mscale, scale};
		phys.colliders.push_back(col);
		phys.velocity = direction*5*KM_TO_UNITS+base_vel;
		e.get()->get_physics()= phys; 
		e.downcast<Projectile>()->spawned_by_player = spawned_by_player;
		//play_sound("machine-gun.mp3");
}	
void WeaponsComp::fire_missile(Vec3 start, Vec3 direction,Vec3 base_vel, Quat rot,EntityRef target, bool homing,bool spawned_by_player){
		MeshPart m;
		m.string = "cylinder";
		m.offset= Trans::create();
		m.color = RED;
		EntityRef e = create_entity<Missile>();
		Missile * miss= e.downcast<Missile>();
		miss->target = target;
		miss->homing = homing;
		miss->ship.parent = e;
		e.get()->add_tag(tag_movable);
		e.get()->get_mesh().meshes["base"] = m;
		PhysicsComp phys = {0};
		phys.destroy_on_impact = true;
		phys.mass = 1.0;
		phys.is_valid = true;
		phys.trans.trans = Trans::create();
		phys.trans.trans.translation = start;
		phys.angular_velocity = Vec3{0,0,0};
		phys.trans.trans.rotation =rot;
		Collider col;
		col.offset= Trans::create();
		Vec3 scale = Vec3{0.1, 0.05, 0.05};
		Vec3 mscale = Vec3{-scale.x, -scale.y, -scale.z};
    //col.bb = BoundingBox{Vec3{-1.91526,-0.309, -0.309}/2.0, Vec3{1.0067,0.309, 0.309}/2.0};
		col.bb = BoundingBox{mscale, scale};
		phys.colliders.push_back(col);
		phys.velocity = direction*2.0*KM_TO_UNITS+base_vel;
		e.get()->get_physics()= phys;
		e.downcast<Missile>()->spawned_by_player = spawned_by_player;	
}
Projectile::Projectile(){
	remaining_time = 144.0;
	pending_kill =false;
	spawned_by_player = false;
};
void Projectile::on_tick(){
	if(pending_kill){

		printf("error %ul is pending kill\n", id);
	}
	assert(!pending_kill);
	remaining_time -= 1.0/60.0;	
	auto a=line_trace(get_location(), get_location()+Vec3::from(Vector3Normalize(get_velocity())*1./60.0));
	if(a){
		apply_damage(get_as_ref(this), *a, get_forward_vector(),10.0);
		spawn_explosion((get_location()-Vec3::from(Vector3Normalize(get_velocity())*0.1)), 1.0);
		if(spawned_by_player){
			log("bullet impact",2.0);
		}
		destroy_entity(get_as_ref(this));		
	}
	char buff[256];
	snprintf(buff, 255,"remaining time:%d, distance travelled:%d km\n",(int)remaining_time, (int)((144.0-remaining_time)*Vector3Length(get_physics().velocity)*UNITS_TO_KM));
	log(buff, 0.01);
	if(remaining_time<0.0){	
		spawn_explosion((get_location()-Vec3::from(Vector3Normalize(get_velocity())*0.1)), 1.0);
		destroy_entity(EntityRef{id, runtime.level->generations[id]});
		pending_kill = true;
		log("bullet timeout",2.0);
	}

}
Projectile::~Projectile(){
}
void Projectile::on_damage(Vec3 incoming_direction, double damage){	
		remaining_time =0.001;
}
void Projectile::serialize(Serializer * ser)const {
	ser->serialize("Projectile");
	ser->serialize(id);
	ser->serialize(tags);
	ser->serialize(remaining_time);
	ser->serialize(pending_kill);
	ser->serialize(spawned_by_player);
}

Projectile Projectile::deserialize(Deserializer* des){
	Projectile out;
	des->deserialize<std::string>();
	out.id = des->deserialize<uint32_t>();
	out.tags = des->deserialize<Tag>();
	out.remaining_time = des->deserialize<double>();
	out.pending_kill = des->deserialize<bool>();
	out.spawned_by_player = des->deserialize<bool>();
	return out;
}
Entity * Projectile::interface_deserialize(Deserializer&des){
	return new Projectile(Projectile::deserialize(&des));
}
Missile::Missile(){
	remaining_time =100.0;
	ship = ShipComp{};
	ship.accel_value = 0.1;
	spawned_by_player = false;
}
Missile::~Missile(){
}
void Missile::on_tick(){
remaining_time -= 1.0/60.0;	
	
	ship.parent = get_as_ref(this);
	ship.target = target;
	if(homing){
		if(target.get())ship.use_target = homing; else ship.use_target = false;
	}else{
		ship.use_target = false;
	}	
	auto a=line_trace(get_location(), get_location()+Vec3::from(Vector3Normalize(get_velocity())*1./30.0));
	if(a){
		apply_damage(get_as_ref(this), *a, get_forward_vector(),100.0);
		spawn_explosion(get_location()-get_forward_vector(), 30.0);
		if(spawned_by_player){
			log("missile impact", 2.0);
		}
		destroy_entity(get_as_ref(this));
	}
	if(remaining_time<0.0){
		spawn_explosion(get_location(), 30.0);
		destroy_entity(EntityRef{id, runtime.level->generations[id]});
		log("missile timeout", 2.0);
	}
	ship.update();
}
void Missile::on_damage(Vec3 incoming_direction, double damage){	
	remaining_time =0.001;	
}
void Missile::serialize(Serializer*ser) const{
	ser->serialize("Missile");
	ser->serialize(id);
	ser->serialize(tags);
	ser->serialize(ship);
	ser->serialize(remaining_time);
	ser->serialize(target);
	ser->serialize(homing);
	ser->serialize(spawned_by_player);
}
Missile Missile::deserialize(Deserializer* des){
	Missile out;
	des->deserialize<std::string>();
	out.id =des->deserialize<uint32_t>();
	out.tags = des->deserialize<Tag>();
	out.ship = des->deserialize<ShipComp>();
	out.remaining_time = des->deserialize<double>();
	out.target = des->deserialize<EntityRef>();	
	out.homing = des->deserialize<bool>();
	out.spawned_by_player = des->deserialize<bool>();
	return out;
}
Entity * Missile::interface_deserialize(Deserializer&des){
	return new Missile(Missile::deserialize(&des));
}
Texture gen_explosion_texture(){
	
	Image img = LoadImage("./assets/explosion.png");	
	return LoadTextureFromImage(img);
}
void spawn_explosion(Vec3 pos, double size){
	//static Texture texture = LoadTextureFromImage(GenImageColor(821,821, Color{255, 0,0, 128}));//
	//static Texture texture = LoadTexture("../../assets/explosion.png");
	//printf("%d, %d\n", texture.height, texture.width);
	static Texture texture = gen_explosion_texture();	
	Texture tex = texture;
	spawn_repeating(1.0,[tex,pos, size](double time){
		double size2 = size*(1.0-time)*0.10;
		draw_call_3d([tex, pos, size2](){
			DrawBillboardPro(get_level().cam, tex, Rectangle{0,0,280,239},pos, get_level().cam.up,Vector2{(float)size2, (float)size2}, Vector2{(float)(0.5*size2),(float)(0.5*size2)},0.0,WHITE);
		});
		//draw_call([tex](){
			//DrawTexture(tex, 64, 64, WHITE);
		//});
	});

}

}
