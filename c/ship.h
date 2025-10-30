#pragma once 
#include "base.h"
#include "level.h"
#define SHIP_COMPS_IDX 2
#define comp_ship 0b100
typedef enum{
	Patrol,
	SearchFor,
	DirectAttack, 
	Skirmish,
}AiState;
typedef enum{
	Rocket,
	Ballistic,
}MovementMode;
typedef enum{
	InputHuman,
	InputMoveTo,
	InputAi,
}InputMode;
typedef enum{
	Player, 
	Enemy,
}Team;
typedef struct {	
	Vec3 move_to_point;
	Vec3 home_base;
	AiState state;
}AiInfo;
REFLECT_STRUCT(AiInfo, REFLECT_FIELD(AiInfo, Vec3,move_to_point),
	       REFLECT_FIELD(AiInfo, Vec3, home_base),REFLECT_FIELD(AiInfo, int, state))
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
	AiInfo ai_info;
}InputData;
REFLECT_STRUCT(InputData,REFLECT_FIELD(InputData, bool, is_ai), REFLECT_FIELD(InputData, int, input_mode), REFLECT_FIELD(InputData, int, mode), REFLECT_FIELD(InputData, bool, is_proj), REFLECT_FIELD(InputData, Vec3, input), REFLECT_FIELD(InputData, Vec3, rot_input), REFLECT_FIELD(InputData, Vec3, desired_velocity), REFLECT_FIELD(InputData, Vec3, desired_velocity), REFLECT_FIELD(InputData, Vec3, desired_pos), REFLECT_FIELD(InputData, Vec4, desired_or), REFLECT_FIELD(InputData, AiInfo, ai_info))
typedef struct {
	InputData input;
	EntityRef target;
	AiInfo ai_info;
	Team team;
	double acc;
}ShipComp;
REFLECT_STRUCT(ShipComp, REFLECT_FIELD(ShipComp,InputData, input), REFLECT_FIELD(ShipComp,EntityRef,target), REFLECT_FIELD(ShipComp, int, team), REFLECT_FIELD(ShipComp,double,acc))
void ship_update();
ShipComp * get_ship_comp(EntityRef ref);
ShipComp * get_ship_comps();
void ship_handle_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage);

EntityRef create_ship(Vec3 location, Vec3 angle, bool player);
bool has_view_to(EntityRef r, EntityRef k);
extern ComponentHandler ship_handler;
Vec3 next_impulse(Vec3 pos, Vec3 end, Vec3 vel,Vec3 des_vel, double acc);
