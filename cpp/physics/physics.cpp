#include "physics.hpp"
#include "../level.hpp"
using namespace Torpedo;

static vector<PhysicsComp> comps;
static vector<uint32_t> indexs;
static vector<Entity *> ptrs;
static std::unordered_map<uint64_t, vector<uint32_t>> grid;
static double square_size = 1.0;
static double min_x = 0.0;
static double min_y = 0.0;
static double min_z = 0.0;
static double max_x = 0.0;
static double max_y = 0.0;
static double max_z = 0.0;
static int64_t compute_position(Vector3 v){
    int64_t dx =(max_x -min_x)/square_size;
    int64_t dz = (max_z-min_z)/square_size;
    int64_t dy = (max_y -min_y)/square_size;
    int64_t px = (v.x-min_x)/square_size;
    int64_t py =(v.y-min_y)/square_size;
    int64_t pz =(v.z-min_z)/square_size;
//    printf("%lld, %lld, %lld dx:%lld, dy:%lld, dz:%lld\n",px, py, pz,dx,dy,dz);
    if(px>dx|| px<0 || py>dy || py<0 || pz>dz || pz<0){
        return -1;
    }
    int64_t x = px+py*dy+pz*dz*dy;
 //   printf("%lld\n",x);
    return x;
}
static void setup_grid(){
    grid.clear();
    for(const auto &i:comps){
        auto pos = i.trans.trans.translation;
        if(pos.x<min_x){
            min_x = pos.x;
        }
        if(pos.y<min_y){
            min_y = pos.y;
        }
        if(pos.z<min_z){
            min_z = pos.z;
        }
        if(pos.x>max_x){
            max_x = pos.x;
        }
        if(pos.y>max_y){
            max_y = pos.y;
        }
        if(pos.z>max_z){
            max_z = pos.z;
        }
    }
    for(size_t i =0; i<comps.size(); ++i){
        int64_t p =compute_position(comps[i].trans.trans.translation); 
        if(grid.contains(p)){
            grid[p].push_back(i);
        } else{
            vector<uint32_t> v= {(uint32_t)i};
            grid[p] = v;
        }
    }
    size_t count = 0;
    size_t lengths= 0;
    for(auto &i : grid){
        count += 1;
        lengths+=i.second.size();
    }
   // printf("average vec size:%f\n", (double)lengths/(double)count);
}

std::array<Vec3, 2> collision_response(
    double m1,
    Vec3 v1,
    double m2,
    Vec3 v2,
    Vec3 normal); 
std::array<Vec3, 2> angular_collision_response(
    double m1,
    Vec3 v1,
    Vec3 p1,
    double m2,
    Vec3 v2,
    Vec3 p2); 
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
[[gnu::always_inline]]
static void update_pair(size_t i, size_t j){
        if(auto col = physics_comp_check_collision(comps[i], comps[j])){
            comps[i].trans.trans.translation += col->norm*(col->depth+0.01);
            auto v = collision_response(comps[i].mass, comps[i].velocity, comps[j].mass, comps[j].velocity, Vec3::from(Vector3Normalize(col->norm)));
            auto v2 = angular_collision_response(comps[i].mass, comps[i].velocity, comps[i].trans.trans.translation,comps[j].mass, comps[j].velocity, comps[j].trans.trans.translation);
            comps[i].velocity = v[0];
            comps[j].velocity = v[1];
           
            
           }
}
uint64_t update_obj(size_t i){
        size_t count = 0;
        Vec3 v = comps[i].trans.trans.translation;
        for(int x = -1; x<2; x++){
            for(int y =-1; y<2; y++){
                for(int z =-1; z<2; z++){
                    Vec3 v0 = v+Vec3{(double)x,(double)y,(double)z}*square_size;
                    int64_t p = compute_position(v0);
                    if(p<0){
                        continue;
                    }
                    if(grid.contains(p)){
                        for(const auto j: grid[p]){
                            if(i == j){
                                continue;
                            }
                            update_pair(i,j);
                            count += 1;
                        }
                    } 
                }
            }
        }
    return count;
     
 
}
void update_physics(){    
#define GRID

    setup_grid();
 
    size_t count = 0;
    for(size_t i =0; i<comps.size(); i++){ 
        comps[i].trans.trans.translation += comps[i].velocity*1./60.0;
        comps[i].trans.trans.rotation =Quat::from(QuaternionFromMatrix(QuaternionToMatrix(comps[i].trans.trans.rotation)*QuaternionToMatrix(comps[i].angular_velocity)));
        comps[i].trans.trans.rotation = Quat::from(QuaternionNormalize(comps[i].trans.trans.rotation));
        #ifdef GRID
        count += update_obj(i);
        #endif
        #ifndef GRID
        for(size_t j = i+1; j<comps.size(); j++){
            update_pair(i,j);
            count += 1;
        }
        #endif
    }
    double avg = (double)count/(double)comps.size();
//    printf("average compute count:%f\n",avg);
    grid.clear();
}
void physics_finish_update(){
    vector<PhysicsComp>& phys_comps = get_level().physics;
    for(int i =0; i<comps.size(); i++){
        phys_comps[indexs[i]] = comps[i];
    }
}
