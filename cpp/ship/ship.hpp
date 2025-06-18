#pragma once 
#include "../utils.hpp"
#include "../level.hpp"
namespace Torpedo{
typedef struct {
	bool use_desired_rotation;
	bool use_desired_position;
	int32_t fuel;
	EntityRef parent;	
	Quat desired_rotation;
	Vec3 desired_position;
	Quat rotation_input;
	Vec3 movement_input;
}ShipComp;
void ship_comp_update(ShipComp * comp);

class PlayerShip:public Entity{
	ShipComp ship;
	public:
	PlayerShip();
	virtual void on_tick();
	EntityRef create();
};
EntityRef create_player_ship(Vec3 pos, Quat rot);
};
