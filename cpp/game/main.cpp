#include "../engine/level.hpp"
#include "ship/ship.hpp"
using namespace Torpedo;
void load_test_level1(){
    #define MULT 
    int dims =4;
      EntityRef player =create_player_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)}, Quat{0,0,0,1});
    EntityRef enemy = create_npc_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)},Quat{0,0,0,-1},Alignment::EnemyAligned);
    #ifdef MULT
 
    int count =5;
    for(int x = -count; x<count+1; x++){
        for(int y = -count; y<count+1; y++){
            for(int z = -count; z<count+1; z++){
                Vec3 point = Vec3{(double)x,(double)y,(double)z}*4;
                Vec3 v;
                v.x = x == 0 ? 0 : (x> 0 ? -1 : 1);
                v.y = y == 0 ? 0 : (y> 0 ? -1 : 1);
                v.z = z == 0 ? 0 : (z> 0 ? -1 : 1);
                Vec3 ang;
                ang.x = (rand()%1000)/1000.0*2-1;
                ang.y= (rand()%1000)/1000.0*2-1;
                ang.z = (rand()%1000)/1000.0*2-1;
//                ang *= 0.0;
                EntityRef a = create_cube(point,Vec3{0.5, 0.5, 0.5}, Vec3{0,0,0}, WHITE, ang);
            }
        }
    }
    #endif
    #ifndef MULT
    double s = rand()%1000/1000.0*2*M_PI;
    Vec3 p1 = Vec3{-1, sin(s), cos(s)};
    Vec3 p2 = Vec3{-1, cos(s), -sin(s)};
    Vec3 v1 = {0,-sin(s), -cos(s)};
    Vec3 v2 = {0,-cos(s), sin(s)};
    double scale = 5.0;
    double speed = 0.5;
    create_cube(p1*scale, Vec3{1,1,1}, v1*speed, RED);
    create_cube(p2*scale, Vec3{1,1,1}, v2*speed, BLUE);
    #endif
}
void  load_test_level2(){
    int dims =25;
    EntityRef player =create_player_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)}, Quat{0,0,0,1});
    EntityRef enemy = create_npc_ship(Vec3{(double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2), (double)(rand()%dims-dims/2)},Quat{0,0,0,-1},Alignment::EnemyAligned); 
}
int main(int argc, const char** argv){
    Torpedo::mainloop(load_test_level1);
}
