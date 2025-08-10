#include "level.h"
#include "physics.h"
#include "renderer.h"
Runtime runtime;
void apply_damage(EntityRef source, EntityRef target,  Vec3 direction, double damage){}
void tick(){
    game_render(&runtime.level->cam);
    physics_prepare_update();
    update_physics();
    physics_finish_update();
}
