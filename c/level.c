#include "level.h"
#include "physics.h"
Runtime runtime;
void apply_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage){}
void tick(){
    physics_prepare_update();
    update_physics();
    physics_finish_update();
}
