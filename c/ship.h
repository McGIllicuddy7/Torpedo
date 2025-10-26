#pragma once 
#include "base.h"
#include "level.h"
#define SHIP_COMPS_IDX 2
#define comp_ship 0b100

typedef enum{
	Rocket,
	Ballistic,
}MovementMode;
typedef enum{
	InputHuman,
	InputMoveTo,
}InputMode;
typedef enum{
	Player, 
	Enemy,
}Team;
typedef struct{
	bool is_ai;
	InputMode input_mode;
	MovementMode mode;
	bool is_proj;
	Vec3 input;
	Vec3 rot_input;
	Vec3 desired_velocity;
	Vec3 desired_pos;
	Quat desired_or;
}InputData;
typedef struct {
	InputData input;
	EntityRef target;
	Team team;
}ShipComp;
void ship_update();
ShipComp * get_ship_comp(EntityRef ref);
ShipComp * get_ship_comps();
void ship_handle_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage);

EntityRef create_ship(Vec3 location, Vec3 angle, bool player);
bool has_view_to(EntityRef r, EntityRef k);