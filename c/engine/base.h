#pragma once
#include <tgmath.h>
#include <stdint.h>
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
#include "utils.h"
typedef struct {
	double x;
	double y;
	double z;
}Vec3;
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
        return Vec4{left.x*right.x, left.y*right.y, left.z*right.z, left.w*right.w};
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
typedef struct {
	Vec3 norm;
	double depth;
}Col;
typedef struct {
	Trans trans;
}TransformComp;
typedef struct{
	Trans offset;
	BoundingBox bb;
}Collider;
typedef struct {	
	TransformComp trans;
	Collider* colliders;
	size_t collider_count;
	Vec3 velocity;
	double mass;
	Vec3 angular_velocity;
	bool destroy_on_impact;
	bool can_ever_collide;
	bool is_valid;
}PhysicsComp;
enable_vec_type(PhysicsComp);
static inline void PhysicsComp_reset(PhysicsComp*phys){
	phys->is_valid = false;
	phys->trans.trans =Trans_create();
	if(phys->colliders){
		free(phys->colliders);
	}
	phys->colliders = 0;
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
typedef struct {
	size_t mesh_count;
	MeshPart * meshes;
}MeshComp;
enable_vec_type(MeshComp);
static inline Vec3 random_vector(){
    double theta = (double)(rand()%100'000)/100'000.0*2.0*PI;
    double phi = (double)(rand()%100'000)/100'000.0*2.0*PI;
    double x = sin(theta)*cos(phi);
    double y = sin(theta)*sin(phi);
    double z = cos(theta);
    return Vec3{x,y,z};
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
