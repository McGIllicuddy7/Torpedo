#include "renderer.h"
#include "base.h"
static void draw_mesh_comp( Arena * arena,MeshComp cmp, Trans trans, BoundingBox b){
    for(size_t i =0 ; cmp.mesh_count; i++){
        Model *md = StringModelHashTable_find(runtime.level->models, new_string(arena,cmp.meshes[i].string));

        Matrix old = md->transform;
        md->transform = QuaternionToMatrix(Vec4_to_Vector4(trans.rotation)); 
        Vec3 loc = Vec3_add(trans.translation,to_global_vector(cmp.meshes[i].offset.translation, get_forward_vector(trans), get_right_vector(trans), get_up_vector(trans)));
        DrawModel(*md, Vec3_to_Vector3(loc),1.0,cmp.meshes[i].color);//        printf("%f,%f,%f\n", loc.x, loc.y, loc.z);
        md->transform = old;
/*        b.min += trans.translation;
        b.max += trans.translation;
        DrawBoundingBox(b, GREEN);
*/
    }
}
void game_render(Camera * cam){
    BeginDrawing(); ClearBackground((Color){16, 16, 32, 255});
    BeginMode3D(*cam);
    Arena * arena= arena_create();
    for(size_t i =0; i<runtime.level->meshes.length; i++){
        if(runtime.level->meshes.items[i].mesh_count == 0|| !runtime.level->alive.items[i]){
            continue;
        }
        draw_mesh_comp(arena,runtime.level->meshes.items[i], runtime.level->physics.items[i].trans.trans,runtime.level->physics.items[i].colliders[0].bb);
    } 
    arena_destroy(arena);
    EndMode3D();
    EndDrawing();

}
