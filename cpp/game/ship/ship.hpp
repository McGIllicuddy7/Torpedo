#pragma once 
#include "../../engine/utils.hpp"
#include "../../engine/level.hpp"
#include "../../engine/particles.hpp"
constexpr double  UNITS_TO_KM = 1.;
constexpr double KM_TO_UNITS  = 1./UNITS_TO_KM;
template<typename T, void(*to_call)(T&)> struct Timer{
	double remaining_time;
	T capture;
	bool is_valid;
	void update(double dt){
		if(!is_valid){
			return;
		}
		remaining_time	 -= dt;
		if(remaining_time<0){
			to_call(capture);
			is_valid = false;
		}
	
	}
};
namespace Torpedo{ 
enum class Alignment{
	PlayerAligned, EnemyAligned
};
struct WeaponsComp{
	void fire_projectile(Vec3 start, Vec3 direction,Vec3 base_vel,Quat rot, bool spawned_by_player = false);
	void fire_missile(Vec3 start, Vec3 direction, Vec3 base_vel,Quat rot, EntityRef target,bool homing, bool spawned_by_player = false);
};
struct AIComp{
	Alignment Align;
	EntityRef parent;	
	static void test(EntityRef&);
	Timer<EntityRef, AIComp::test> timer;
};
struct ShipComp{
	bool use_desired_rotation = false;	
	bool use_desired_position = false;
	bool use_target=false;
	bool stablized_velocity = true;
	int32_t fuel = 1000;
	float accel_value= 0.05*KM_TO_UNITS;
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
	bool spawned_by_player;
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
	bool spawned_by_player;
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
