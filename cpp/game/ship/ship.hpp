#pragma once 
#include "../../engine/utils.hpp"
#include "../../engine/level.hpp"
#include "../../engine/particles.hpp"
namespace Torpedo{
enum class Alignment{
	PlayerAligned, EnemyAligned
};
struct WeaponsComp{
	void fire_projectile(Vec3 start, Vec3 direction,Quat rot);
	void fire_missile(Vec3 start, Vec3 direction, Quat rot, EntityRef target,bool homing);
};

struct ShipComp{
	bool use_desired_rotation = false;	
	bool use_desired_position = false;
	bool use_target=false;
	bool stablized_velocity = true;
	int32_t fuel = 1000;
	float accel_value= 0.05;
	EntityRef parent =EntityRef{0,0};
	Quat desired_rotation = Quat{0,0,0,1};
	Vec3 desired_position = Vec3{0,0,0};
	Quat rotation_input = Quat{0,0,0,0};
	Vec3 movement_input = Vec3{0,0,0};
	WeaponsComp weapons = WeaponsComp{};
	EntityRef target = {0,0};
	double health = 100.0;
	void private_update_homing();
	void private_update_non_homing();
	void update();
	void on_damage(Vec3 direction, double amount);
	void serialize(Serializer*ser) const;	
	static ShipComp deserialize(Deserializer*des);
};

class Aligned{
public:
	virtual Alignment get_alignment() = 0;
};
class PlayerShip:public Entity,public Aligned{
	ShipComp ship;
	public:
	PlayerShip();
	virtual void on_tick();
	virtual Alignment get_alignment();
	EntityRef create();
	virtual void on_damage(Vec3 incoming_direction, double damage);
	virtual void serialize(Serializer* ser) const;
	static PlayerShip deserialize(Deserializer* des);
	static Entity * interface_deserialize(Deserializer&des);
};
Register(PlayerShip, Entity);
class Projectile:public Entity{	
public:
	double remaining_time;
	bool pending_kill;
	Projectile();
	virtual ~Projectile();
	virtual void on_tick();
	virtual void on_damage(Vec3 incoming_direction, double damage);
	virtual void serialize(Serializer* ser) const;
	static Projectile deserialize(Deserializer* des);
	static Entity * interface_deserialize(Deserializer&des);
};
Register(Projectile, Entity);
class Missile:public Entity{
public:
	ShipComp ship;
	double remaining_time;
	EntityRef target;
	bool homing;
	Missile();
	virtual ~Missile();
	virtual void on_tick();
	virtual void on_damage(Vec3 incoming_direction, double damage);
	virtual void serialize(Serializer* ser) const;
	static Missile deserialize(Deserializer* des);
	static Entity * interface_deserialize(Deserializer&des);

};
Register(Missile,Entity);
class NPCShip: public Entity, public Aligned{
	ShipComp ship;
public:
	Alignment align;
	NPCShip();
	virtual Alignment get_alignment();
	virtual void on_tick();
	virtual void on_damage(Vec3 incoming_direction, double damage);
	virtual void serialize(Serializer* ser) const;
	static NPCShip deserialize(Deserializer* des);
	static Entity * interface_deserialize(Deserializer&des);
};
Register(NPCShip, Entity);
EntityRef create_player_ship(Vec3 pos, Quat rot);
EntityRef create_npc_ship(Vec3 pos, Quat rot, Alignment align);
void spawn_explosion(Vec3 pos, double size);
};
