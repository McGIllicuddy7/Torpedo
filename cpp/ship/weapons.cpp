#include "ship.hpp"
namespace Torpedo{
	void WeaponsComp::fire_projectile(Vec3 start, Vec3 direction, Quat rot){
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
		phys.velocity = direction*30.0;
		e.get()->get_physics()= phys; 
}	
void WeaponsComp::fire_missile(Vec3 start, Vec3 direction, Quat rot,EntityRef target, bool homing){
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
		phys.velocity = direction*100;
		e.get()->get_physics()= phys;
}
Projectile::Projectile(){
	remaining_time = 10.0;
	pending_kill =false;
};
void Projectile::on_tick(){
	if(pending_kill){
		printf("error %ul is pending kill\n", id);
	}
	assert(!pending_kill);
	remaining_time -= 1.0/60.0;
	if(remaining_time<0.0){
		destroy_entity(EntityRef{id, runtime.level->generations[id]});
		pending_kill = true;
	}
}
Projectile::~Projectile(){
}
void Projectile::on_damage(Vec3 incoming_direction, double damage){
	destroy_entity(get_as_ref(this));
}
void Projectile::serialize(Serializer * ser)const {
	ser->serialize("Projectile");
	ser->serialize(id);
	ser->serialize(tags);
	ser->serialize(remaining_time);
	ser->serialize(pending_kill);
}
Projectile Projectile::deserialize(Deserializer* des){
	Projectile out;
	des->deserialize<std::string>();
	out.id = des->deserialize<uint32_t>();
	out.tags = des->deserialize<Tag>();
	out.remaining_time = des->deserialize<double>();
	out.pending_kill = des->deserialize<bool>();
	return out;
}
Entity * Projectile::interface_deserialize(Deserializer&des){
	return new Projectile(Projectile::deserialize(&des));
}
Missile::Missile(){
	remaining_time = 60.0;
	ship = ShipComp{};
	ship.accel_value = 0.1;
}
Missile::~Missile(){
}
void Missile::on_tick(){
remaining_time -= 1.0/60.0;
	if(remaining_time<0.0){
		destroy_entity(EntityRef{id, runtime.level->generations[id]});
	}
	ship.parent = get_as_ref(this);
	ship.target = target;
	if(homing){
		if(target.get())ship.use_target = homing; else ship.use_target = false;
	}else{
		ship.use_target = false;
	}
	
	ship.update();
}
void Missile::on_damage(Vec3 incoming_direction, double damage){
	destroy_entity(get_as_ref(this));
}
void Missile::serialize(Serializer*ser) const{
	ser->serialize("Missile");
	ser->serialize(id);
	ser->serialize(tags);
	ser->serialize(ship);
	ser->serialize(remaining_time);
	ser->serialize(target);
	ser->serialize(homing);
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
	return out;
}
Entity * Missile::interface_deserialize(Deserializer&des){
	return new Missile(Missile::deserialize(&des));
}
}
