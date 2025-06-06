#include "renderer.hpp"
#include "../level.hpp"
using namespace Torpedo;
static void draw_mesh_comp(const MeshComp& cmp, const Trans &trans, BoundingBox b){
    for(const auto&i:cmp.meshes){
        Model md = get_level().models[i.second.string];
        auto old = md.transform;
        md.transform = QuaternionToMatrix(trans.rotation); 
        auto loc = trans.translation+i.second.offset.translation;
        DrawModel(md, loc,1.0,i.second.color);//        printf("%f,%f,%f\n", loc.x, loc.y, loc.z);
        md.transform = old;
  //      b.min += trans.translation;
 //       b.max += trans.translation;
//        DrawBoundingBox(b, GREEN);

    }
}
void renderer_update(Camera *cam){
    UpdateCamera(cam, CAMERA_FREE);
    BeginDrawing();
    ClearBackground(BLACK); 
    DrawFPS(1100, 80);
    rlSetClipPlanes(0.005, 5000000);
    BeginMode3D(*cam);
    for(size_t i =0; i<get_level().meshes.size(); i++){
        if(get_level().meshes[i].meshes.empty()|| !get_level().entities[i]){
            continue;
        }
        draw_mesh_comp(get_level().meshes[i], get_level().physics[i].trans.trans,get_level().physics[i].colliders[0].bb);
    }
    EndMode3D();
    EndDrawing();
}
