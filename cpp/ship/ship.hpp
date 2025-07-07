#pragma once 
#include "../utils.hpp"
#include "../level.hpp"
namespace Torpedo{
struct WeaponsComp{
};
struct ShipComp{
	bool use_desired_rotation = false;	
	bool use_desired_position = false;
	bool use_target=false;
	int32_t fuel = 1000;
	EntityRef parent =EntityRef{0,0};
	Quat desired_rotation = Quat{0,0,0,1};
	Vec3 desired_position = Vec3{0,0,0};
	Quat rotation_input = Quat{0,0,0,0};
	Vec3 movement_input = Vec3{0,0,0};
	WeaponsComp weapons = WeaponsComp{};
	EntityRef target = {0,0};
	void private_update_homing();
	void private_update_non_homing();
	void update();
	void apply_damage(Vec3 direction, double amount);
};


class PlayerShip:public Entity{
	ShipComp ship;
	public:
	PlayerShip();
	virtual void on_tick();
	EntityRef create();
};
EntityRef create_player_ship(Vec3 pos, Quat rot);
};
