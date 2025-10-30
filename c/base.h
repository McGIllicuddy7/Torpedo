#pragma once
#if __linux__
#include <raylib.h>
#include <raymath.h>
#include <rlgl.h>
#endif
#ifndef __linux__
#include </usr/local/include/raylib.h>
#include </usr/local/include/raymath.h>
#include </usr/local/include/rlgl.h>
#endif
#include <tgmath.h>
#include <stdint.h>

#include "utils.h"
#include "lolth.h"
REFLECT_STRUCT(Vector3, REFLECT_FIELD(Vector3, float, x), REFLECT_FIELD(Vector3, float, y), REFLECT_FIELD(Vector3, float, z))
REFLECT_STRUCT(BoundingBox, REFLECT_FIELD(BoundingBox,Vector3, min), REFLECT_FIELD(BoundingBox, Vector3, max))
REFLECT_STRUCT(Color, REFLECT_FIELD(Color, char, r),REFLECT_FIELD(Color, char, g),REFLECT_FIELD(Color, char, b),REFLECT_FIELD(Color, char, a))

typedef struct {
	double x;
	double y;
	double z;
}Vec3;
REFLECT_STRUCT(Vec3,
	REFLECT_FIELD(Vec3, double,x),
	REFLECT_FIELD(Vec3, double,y),
	REFLECT_FIELD(Vec3, double,z),
);
static inline Vec3 Vec3_add(Vec3 left, Vec3 right){
	return (Vec3){left.x+right.x, left.y+right.y, left.z+right.z};
}
static inline double  Vec3_len(Vec3 v){
	return sqrt(v.x*v.x+v.y*v.y+v.z*v.z);
}
static inline Vec3 Vec3_scale(Vec3 self, double v){
	return (Vec3){self.x*v, self.y*v, self.z*v};
}
static inline Vec3 Vec3_normalize(Vec3 v){
	return Vec3_scale(v, 1.0/(Vec3_len(v)));
}
static inline Vec3 Vec3_sub(Vec3 left, Vec3 right){
	return (Vec3){left.x+right.x, left.y+right.y, left.z+right.z};
}
static inline double Vec3_dot_product(Vec3 a, Vec3 b){
	return a.x*b.x+a.y*b.y+a.z*b.z;
}
static inline double Vec3_dist(Vec3 a, Vec3 b){
	return Vec3_len(Vec3_sub(a,b));
}

static inline Vector3 Vec3_to_Vector3(Vec3 self){
	return (Vector3){(float)self.x, (float)self.y, (float)self.z};
}
static inline Vec3 Vec3_from_Vector3(Vector3 p){
	return (Vec3){(double)p.x, (double)p.y,(double)p.z};
}
typedef struct{
	double x;
	double y;
	double z;
	double w;
}Vec4;
REFLECT_STRUCT(Vec4,
	REFLECT_FIELD(Vec4, double,x),
	REFLECT_FIELD(Vec4, double,y),
	REFLECT_FIELD(Vec4, double,z),
	REFLECT_FIELD(Vec4, double,w),

);

static inline Vec4 Vec4_add(Vec4 left, Vec4 right){
	return (Vec4){left.x+right.x, left.y+right.y, left.z+right.z, left.w+right.w};
}
static inline Vec4 Vec4_sub(Vec4 left, Vec4 right){
	return (Vec4){left.x+right.x, left.y+right.y, left.z+right.z,left.w+right.w};
}
static inline Vec4 Vec4_scale(Vec4 self, double v){
	return (Vec4){self.x*v, self.y*v, self.z*v,self.w*v};
}
static inline Vec4 Vec4_mul(Vec4 left, Vec4 right){
        return (Vec4){left.x*right.x, left.y*right.y, left.z*right.z, left.w*right.w};
}
static inline Vector4 Vec4_to_Vector4(Vec4 v){
	return (Vector4){(float)v.x, (float)v.y, (float)v.z, (float)v.w};
}
static inline Vec4 Vec4_from_Vector4(Vector4 v){
	return (Vec4){(double)v.x, (double)v.y, (double)v.z, (double)v.w};
}
typedef Vec4 Quat;
typedef struct{
	Vec3 translation;
	Vec3 scale;
	Vec4 rotation;
}Trans;
REFLECT_STRUCT(Trans, REFLECT_FIELD(Trans, Vec3, translation), REFLECT_FIELD(Trans, Vec3, scale), REFLECT_FIELD(Trans, Vec4, rotation))
static inline Trans Trans_create(){
	Trans out;
        out.translation =  (Vec3){0,0,0};
        out.rotation = (Quat){0,0,0,1};
        out.scale = (Vec3){1,1,1};
        return out;
}
static inline Trans Trans_from_Transform(Transform trans){
	Trans out;
	out.translation = Vec3_from_Vector3(trans.translation);
	out.rotation = Vec4_from_Vector4(trans.rotation);
	out.scale = Vec3_from_Vector3(trans.scale);
	return out;
}
static inline Transform Trans_to_Transform(Trans trans){
	Transform out;
	out.translation = Vec3_to_Vector3(trans.translation);
	out.rotation = Vec4_to_Vector4(trans.rotation);
	out.scale = Vec3_to_Vector3(trans.scale);
	return out;
}
static inline Vec3 get_up_vector(Trans trans){
	return Vec3_from_Vector3(Vector3RotateByQuaternion((Vector3){0,0,1}, Vec4_to_Vector4(trans.rotation)));
}
static inline Vec3 get_left_vector(Trans trans){
	return Vec3_from_Vector3(Vector3RotateByQuaternion((Vector3){0,1,0}, Vec4_to_Vector4(trans.rotation)));
}
static inline Vec3 get_forward_vector(Trans trans){
	return Vec3_from_Vector3(Vector3RotateByQuaternion((Vector3){1,0,0}, Vec4_to_Vector4(trans.rotation)));
}
typedef struct {
	Vec3 norm;
	double depth;
}Col;
REFLECT_STRUCT(Col, REFLECT_FIELD(Col, Vec3, norm), REFLECT_FIELD(Col, double, depth))
typedef struct {
	Trans trans;
}TransformComp;
REFLECT_STRUCT(TransformComp, REFLECT_FIELD(TransformComp, Trans, trans))
typedef struct{
	Trans offset;
	BoundingBox bb;
}Collider;
REFLECT_STRUCT(Collider, REFLECT_FIELD(Collider,Trans, offset), REFLECT_FIELD(Collider, BoundingBox, bb));
typedef struct {	
	TransformComp trans;
	Collider colliders[4];
	size_t collider_count;
	Vec3 velocity;
	double mass;
	Vec3 angular_velocity;
	bool destroy_on_impact;
	bool can_ever_collide;
	bool is_valid;
}PhysicsComp;
REFLECT_STRUCT(PhysicsComp, REFLECT_FIELD(PhysicsComp, TransformComp, trans),REFLECT_FIELD(PhysicsComp, Collider, colliders), REFLECT_FIELD(PhysicsComp, size_t, collider_count),REFLECT_FIELD(PhysicsComp, Vec3, velocity), REFLECT_FIELD(PhysicsComp, double, mass), REFLECT_FIELD(PhysicsComp,Vec3, angular_velocity), REFLECT_FIELD(PhysicsComp, bool, destroy_on_impact), 
 REFLECT_FIELD(PhysicsComp, bool, can_ever_collide),
 REFLECT_FIELD(PhysicsComp, bool, is_valid))
enable_vec_type(PhysicsComp);
static inline void PhysicsComp_reset(PhysicsComp*phys){
	phys->is_valid = false;
	phys->trans.trans =Trans_create();
	phys->collider_count = 0;
	phys->velocity = (Vec3){0,0,0};
	phys->angular_velocity = (Vec3){0,0,0};
	phys->mass = 1;
	phys->can_ever_collide = false;
	phys->is_valid = false;
	phys->destroy_on_impact = false;
}
typedef struct{
	const char * string;
	Trans offset;
	Color color;
}MeshPart;
REFLECT_STRUCT(MeshPart,REFLECT_CSTR(MeshPart, string), REFLECT_FIELD(MeshPart, Trans, offset), REFLECT_FIELD(MeshPart, Color, color))
typedef struct {
	size_t mesh_count;
	MeshPart meshes[4];
}MeshComp;
REFLECT_STRUCT(MeshComp, REFLECT_FIELD(MeshComp, size_t, mesh_count), REFLECT_FIELD(MeshComp,MeshPart, meshes))
enable_vec_type(MeshComp);
static inline Vec3 random_vector(){
    double theta = (double)(rand()%100'000)/100'000.0*2.0*PI;
    double phi = (double)(rand()%100'000)/100'000.0*2.0*PI;
    double x = sin(theta)*cos(phi);
    double y = sin(theta)*sin(phi);
    double z = cos(theta);
    return (Vec3){x,y,z};
}
static inline Vec3 to_global_vector(Vec3 input, Vec3 forward, Vec3 right, Vec3 up){
    Vec3 out = {0,0,0};
    out = Vec3_scale(forward,input.x);
    out= Vec3_add(out,Vec3_scale(right,input.y));
    out =Vec3_add(out, Vec3_scale(up,input.z));
    return out;
}
static inline double get_input_axis(int key_negative, int key_positive){
	double out =0;
	if(IsKeyDown(key_negative)){
		out -= 1;
	}
	if(IsKeyDown(key_positive)){
		out += 1;
	}
	return out;
}
static inline Allocator from_arena(Arena * arena){
	Allocator out;
	out.ptr = arena;
	out.alloc = (void*(*)(void*, size_t))arena_alloc;
	out.dealloc= (void(*)(void*, void*))arena_free;
	return out;
}
