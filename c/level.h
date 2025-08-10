#pragma once
#include "base.h"
enable_hash_type(String, Model);
typedef struct Log{
	struct Log * next;
	char data[256];
	double remaining_time;
}Log;
typedef enum:uint32_t{
	tag_movable = 0b1,
	tag_on_fire = 0b10,
	tag_ship = 0b100,
	tag_projectile = 0b1000,
	tag_pressurized = 0b10000,
	tag_interactable = 0b100000,	
}Tag;
enable_vec_type(Tag);
enable_vec_type(bool);
typedef struct{
	uint32_t index;
	uint32_t generation;
} EntityRef;
typedef enum{
	ApplyDamage,
}EventType;
typedef struct{
	Vec3 direction;
        Vec3 point;
        double damage;
} ApplyDamageInfo;
typedef struct{
	EntityRef target;
	EntityRef source;
	EventType event_type;
	union{
		ApplyDamageInfo apply_damage;
	};
}Event;
typedef struct {
	Camera3D cam;
	u32Vec generations;	
	TagVec tags;
	boolVec alive;
	PhysicsCompVec physics;
	MeshCompVec meshes;
	StringModelHashTable *models;
	Shader shader;
	bool should_save;
        bool should_load;
        const char * save_name;
	void ** components;
        const char *load_name;
}Level;
typedef struct{
	Level * level;
}Runtime;
extern Runtime runtime;
void apply_damage(EntityRef source, EntityRef target, Vec3 normal,double damage);
EntityRef create_entity();
void destroy_entity(EntityRef target);

