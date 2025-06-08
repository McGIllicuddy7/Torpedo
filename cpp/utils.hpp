#pragma once 
#include <memory>
#include <vector>
#include <array>
#include <assert.h>
#include <optional>
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
#include <string>
#include <unordered_map>
namespace Torpedo{
using std::string;
using std::vector;
using std::array;
using std::unique_ptr;
using std::shared_ptr;
using std::optional;
using std::vector;
using std::unordered_map;
template <typename T> class MutSlice{
    T * start;
    size_t count;
    public:
    MutSlice(std::vector<T>& v){
        start = v.begin();
        count = v.len();
    }
    MutSlice(MutSlice<T>& other){
        start = other.start;
        count = other.count;
    }
    template<size_t COUNT> MutSlice(array<T,COUNT> &v){
        start =v.begin();
        count = COUNT;
    }
    size_t len()const {
        return count;
    }
    const T& operator[](size_t idx) const{
        return start[idx];
    }
    T& operator[](size_t idx){
        return start[idx];
    }
    T* begin(){
        return start;
    }
    T* end(){
        return start+count;
    }
    const T* begin() const{
        return start;
    }
    const T* end() const{
        return start+count;
    }
};

template <typename T> class Slice{
    const T * start;
    size_t count;
    public:
    Slice(const std::vector<T>& v){
        start = v.begin();
        count = v.len();
    }
    Slice(const Slice<T>& other){
        start = other.start;
        count = other.count;
    }
    Slice(const MutSlice<T>& other){
        start = other.start;
        count = other.count;
    }
    template <size_t COUNT>Slice(const std::array<T, COUNT>& v){
        start = v.begin();
        count = COUNT;
    }
    size_t len()const {
        return count;
    }
    const T& operator[](size_t idx) const{
        return start[idx];
    }
    T& operator[](size_t idx){
        return start[idx];
    }
    T* begin(){
        return start;
    }
    T* end(){
        return start+count;
    }
    const T* begin() const{
        return start;
    }
    const T* end() const{
        return start+count;
    }
};
struct Vec3{
    double x;
    double y;
    double z;
    static inline Vec3 from(Vector3 v){
        Vec3 out;
        out.x = v.x;
        out.y = v.y;
        out.z = v.z;
        return out;
    }
    inline Vec3 operator=(const Vector3&v){
        x = v.x;
        y = v.y;
        z = v.z;
        return *this;
    }
    inline Vec3 operator+(const Vec3& other) const{
        return Vec3{x+other.x, y+other.y, z+other.z};
    }
    inline Vec3 operator+=(const Vec3& other){
        *this = *this+other;
        return *this;
    }
    inline Vec3 operator+=(const Vector3& other){
        *this = *this+other;
        return *this;
    }
    inline Vec3 operator-(const Vec3& other) const{
        return Vec3{x-other.x, y-other.y, z-other.z};
    }
    inline Vec3 operator-=(const Vec3& other){
        *this = *this-other;
        return *this;
    }
    inline Vec3 operator-=(const Vector3& other){
        *this = *this-other;
        return *this;
    }
    inline Vec3 operator*(const double v) const{
        return Vec3{x*v, y*v, z*v};
    }
    inline Vec3 operator*=(const double v){
        *this = *this*v;
        return *this;
    }
    inline operator Vector3()const {
        return Vector3{(float)x,(float)y,(float)z};
    }
};
struct Vec4{
    double x;
    double y;
    double z;
    double w;
    static Vec4 from(Vector4 v){
        Vec4 out;
        out.x = v.x;
        out.y = v.y;
        out.z = v.z;
        out.w = v.w;
        return out;
    }
    inline Vec4 operator=(const Vector4& v){
        x = v.x;
        y = v.y;
        z = v.z;
        w = v.w;
        return *this;
    }
    inline Vec4 operator+(const Vec4& other) const{
        return Vec4{x+other.x, y+other.y, z+other.z};
    }
    inline Vec4 operator-(const Vec4& other) const{
        return Vec4{x-other.x, y-other.y, z-other.z,w-other.w};
    }
    inline Vec4 operator*(const double v) const{
        return Vec4{x*v, y*v, z*v, w*v};
    }
    inline Vec4 operator*(const Vec4& other)const{
        return Vec4{x*other.x, y*other.y, z*other.z, w*other.w};
    }
    inline operator Vector4()const{
        return {(float)x,(float)y,(float)z,(float)w};
    }
};
using Quat = Vec4;
struct Trans{
    Vec3 translation;
    Vec3 scale;
    Vec4 rotation;
    inline static Trans create(){
        Trans out;
        out.translation =  Vec3{0,0,0};
        out.rotation = Quat{0,0,0,1};
        out.scale = Vec3{1,1,1};
        return out;
 
    }
    inline static  Trans from(Transform trans){
        Trans out;
        out.translation = Vec3::from(trans.translation);
        out.rotation = Vec4::from(trans.rotation);
        out.scale = Vec3::from(trans.scale);
        return out;
    }
    inline operator Transform()const{
        Transform out;
        out.rotation = rotation;
        out.scale = scale;
        out.translation = translation;
        return out;
    }
};
struct Col{
    Torpedo::Vec3 norm;
    double depth;
};
struct TransformComp{
    Torpedo::Trans trans;
};
struct Collider{
    Trans offset;
    BoundingBox bb;
};
struct PhysicsComp{ 
    bool is_valid;
    TransformComp trans;
    vector<Collider> colliders;
    Torpedo::Vec3 velocity;
    double mass;
    Torpedo::Quat angular_velocity;
    inline void reset(){
        is_valid = false; 
    }
};
struct MeshPart{
    const char * string;
    Trans offset;
    Color color;
};
struct MeshComp{
        unordered_map<string, MeshPart> meshes;
    inline void reset(){
        meshes.clear();
    }
    };
}
