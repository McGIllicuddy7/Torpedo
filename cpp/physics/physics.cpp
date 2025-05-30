#include "physics.hpp"
#include "../level.hpp"
using namespace Torpedo;
static vector<PhysicsComp> comps;
static vector<uint32_t> indexs;
static vector<Entity *> ptrs;
std::array<Vec3, 2> collision_response(
    double m1,
    Vec3 v1,
    double m2,
    Vec3 v2,
    Vec3 normal); 
void physics_prepare_update(){
    comps.clear();
    ptrs.clear();
    indexs.clear();
    for(size_t i =0; i<runtime.level->entities.size(); i++){
        if(!runtime.level->entities[i]){
            continue;
        }
        auto p = runtime.level->physics[i];
        if(p.is_valid){
            indexs.push_back(i);
            comps.push_back(p);
            ptrs.push_back(runtime.level->entities[i].get());
        }
    }
}
std::optional<Col>physics_comp_check_collision(const PhysicsComp & a, const PhysicsComp& b){
    for(auto i:a.colliders){
        for(auto j:b.colliders){
            std::optional<Col> c = check_collision(i.bb, i.offset, a.trans,j.bb, j.offset, b.trans);
            if(c.has_value()){
                return c;
            }
        }
    }
    return std::optional<Col>{};
}
void update_physics(){
    for(size_t i =0; i<comps.size(); i++){ 
        comps[i].trans.trans.translation += comps[i].velocity*1./60.;
        for(size_t j = i+1; j<comps.size(); j++){
            if(auto col = physics_comp_check_collision(comps[i], comps[j])){
                comps[i].trans.trans.translation += col->norm*col->depth*3.0;
                auto v = collision_response(comps[i].mass, comps[i].velocity, comps[j].mass, comps[j].velocity, Vec3::from(Vector3Normalize(col->norm)));
                auto m_0 = comps[i].velocity * comps[i].mass + comps[j].velocity*comps[j].mass;
                comps[i].velocity = v[0];
                comps[j].velocity = v[1];
                auto m_1 = comps[i].velocity * comps[i].mass + comps[j].velocity*comps[j].mass;
                printf("m_0:{%f,%f,%f}, m_1: {%f, %f, %f}\n",m_0.x, m_0.y, m_0.z, m_1.x, m_1.y, m_1.z); 
           }
        }
    }
}
void physics_finish_update(){
    vector<PhysicsComp>& phys_comps = get_level().physics;
    for(int i =0; i<comps.size(); i++){
        phys_comps[indexs[i]] = comps[i];
    }
}
