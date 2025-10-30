#pragma once
#include "base.h"
//#define ENTITY_COUNT 32000
#define ENTITY_COUNT 32000
#define COMPONENT_COUNT 64
#define PHYSICS_COMPS_IDX 0
#define MESH_COMPS_IDX 1
enable_hash_type(String, Model);
typedef struct Log{
	struct Log * next;
	char data[256];
	double remaining_time;
}Log;
REFLECT_STRUCT(Log, REFLECT_PTR(Log, Log, next,1), REFLECT_FIELD(Log,char, data  ), REFLECT_FIELD(Log, double, remaining_time))
typedef enum{
		draw_call_text,
		draw_call_circle,
		draw_call_rect,
} DrawCallType;

enable_vec_type(Log);
typedef struct {
	DrawCallType draw_call_type;
	union{
		struct {
			const char * text;
			int x;
			int y;
			int height;
		}draw_call_text_info;
		struct {	
			int x;
			int y;
			int height;
			int width;	
		}draw_call_rect_info;
		struct {
			float r;
			int x;
			int y;
		}draw_call_circ_info;
	};
	Color color;
}DrawCall;
enable_vec_type(DrawCall);
typedef enum{
		draw_call_cube,
		draw_call_sphere,
		draw_call_line,
} DrawCall3DType;

typedef struct {
	DrawCall3DType draw_call_type;
	union{
		struct {
			Vector3 pos;
			float w;
			float h;
			float d;
		}draw_call_cube_info;
		struct{
			Vector3 pos;
			float r;
		} draw_call_sphere_info;
		struct {
			Vector3 start;
			Vector3 end;
		}draw_call_line_info;
	};
	Color color;
}DrawCall3D;
enable_vec_type(DrawCall3D);
typedef enum: uint32_t{
	tag_alive = 0b1,
	tag_movable = 0b10,
}Tag;
typedef enum: uint64_t {
	comp_model= 0b1, 
	comp_physics = 0b10,
} OwnedComps;
typedef struct{
	uint32_t index;
	uint32_t generation;
} EntityRef;
REFLECT_VALUE Type uint32_tINFO= UNSIGNED_intINFO;
REFLECT_STRUCT(EntityRef, REFLECT_FIELD(EntityRef, uint32_t,index ), REFLECT_FIELD(EntityRef, uint32_t,generation ))
enable_vec_type(EntityRef);
typedef enum{
	ApplyDamage = 0b1,
}EventType;
typedef struct{
	Vec3 direction;
        Vec3 point;
        double damage;
} ApplyDamageInfo;
REFLECT_STRUCT(ApplyDamageInfo, REFLECT_FIELD(ApplyDamageInfo, Vec3, direction), REFLECT_FIELD(ApplyDamageInfo, Vec3, point),  REFLECT_FIELD(ApplyDamageInfo,double,damage))
typedef struct{
	EntityRef target;
	EntityRef source;
	EventType event_type;
	union{
		ApplyDamageInfo apply_damage;
	};
}Event;
REFLECT_STRUCT(Event,REFLECT_FIELD(Event, EntityRef, target), REFLECT_FIELD(Event, EntityRef, source), 
	       REFLECT_FIELD(Event, int, source), REFLECT_FIELD(Event, ApplyDamageInfo,apply_damage)
	       )


typedef struct {
	void (*update)();
} System;
typedef struct{
	bool (*handle_event)(Event ev);
}EventHandler;
enable_vec_type(Event);
enable_vec_type(System);
enable_vec_type(EventHandler);
typedef struct{
	void (*destructor)(void*, u32 idx);
	void (*serialize)(Stream * stream, void*);
	void (*deserialize)(Allocator al,Stream * stream, void*);
} ComponentHandler;
typedef struct {
	EntityRef player_entity;
	Camera3D cam;
	Trans cam_player_offset;
	u32* generations;	
	Tag* tags;
	OwnedComps * owned_comps;	
	StringModelHashTable *models;
	Shader shader;
	bool should_save;
	bool should_load;
	const char * save_name;
	void ** components;
	const char *load_name;
	Arena * frame_arena;
	EventVec events;
	LogVec logs;
	DrawCallVec draw_calls;
	DrawCall3DVec draw3d_calls;
	SystemVec systems;
	EventHandlerVec hooks;
	ComponentHandler handlers[COMPONENT_COUNT];
	size_t actual_comp_count;
	void (*damage_handler)(EntityRef source, EntityRef target,  Vec3 direction, double damage);
	EntityRefVec destroy_queue;
}Level;

typedef struct{
	Arena * static_arena;
	Arena * level_arena;
	void (*on_startup)(void*);
	void *startup_data;
	Level * level;
}Runtime;
PhysicsComp *get_physics_comps();
MeshComp * get_mesh_comps();
extern Runtime runtime;
extern ComponentHandler physics_handler;
extern ComponentHandler mesh_handler;

void apply_damage(EntityRef source, EntityRef target, Vec3 normal,double damage);
EntityRef create_entity();
void destroy_entity(EntityRef target);
bool entity_eq(EntityRef a, EntityRef b);
bool entity_is_valid(EntityRef e);
bool has_component(EntityRef e,OwnedComps cmp);
void add_component(EntityRef e, OwnedComps cmp);
void add_tag(EntityRef e, Tag tg);
void remove_component(EntityRef e, OwnedComps cmp);
void remove_tag(EntityRef e, Tag tg);
bool has_tag(EntityRef e, Tag tag);
EntityRefVec get_all_entities_with_tag(Tag tag);
EntityRefVec get_all_entities_with_component(OwnedComps cmp);
EntityRef entity_null();
Tag * get_tag_ptr();
OwnedComps * get_owned_comps_ptr();
void * fralloc(size_t count);
void * stalloc(size_t count);
Arena * frame_arena();
Arena* static_arena();
void register_system(System s);
void draw_call(DrawCall dc);
void draw_call_3d(DrawCall3D dc);
void draw_sphere(Vec3 pos, double r, Color col);
void draw_cube(Vec3 pos, double w,double h, double d, Color col);
void draw_line(Vec3 start, Vec3 end,Color col);
void draw_text(const char * text, int x, int y, int height, Color col);
void draw_rect(int x, int y, int w, int h, Color col);
void draw_circle(int x, int y, float r, Color col);
PhysicsComp * get_physics_comp(EntityRef ref);
MeshComp * get_mesh_comp(EntityRef ref);
Level * get_level();
EntityRef create_debug_cube(Vec3 pos);
Level *create_level();
void save_level(const char * path);
void load_level(const char * path);
Vec3 ent_get_location(EntityRef ref);
Quat ent_get_orientation(EntityRef ref);
Vec3 ent_get_forward_vector(EntityRef ref);
Vec3 ent_get_left_vector(EntityRef ref);
Vec3 ent_get_up_vector(EntityRef ref);
Vec3 ent_get_velocity(EntityRef ref);
EntityRef entity_ref_from_index(int idx);
