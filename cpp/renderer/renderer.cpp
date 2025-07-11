#include "renderer.hpp"
#include "../level.hpp"
using namespace Torpedo;
static void draw_mesh_comp(const MeshComp& cmp, const Trans &trans, BoundingBox b){
    for(const auto&i:cmp.meshes){
        Model md = get_level().models[i.second.string];
        auto old = md.transform;
        md.transform = QuaternionToMatrix(trans.rotation); 
        auto loc = trans.translation+to_global_vector(i.second.offset.translation, trans.get_forward_vector(), trans.get_right_vector(), trans.get_up_vector());
        DrawModel(md, loc,1.0,i.second.color);//        printf("%f,%f,%f\n", loc.x, loc.y, loc.z);
        md.transform = old;
/*        b.min += trans.translation;
        b.max += trans.translation;
        DrawBoundingBox(b, GREEN);
*/
    }
}
void renderer_update(Camera *cam, RenderTexture2D tex,Shader postprocess){
 
   
    BeginTextureMode(tex);
    ClearBackground(BLACK); 
    rlSetClipPlanes(0.001, 5000000);

    for(int i =0; i<runtime.level->draw_calls.size(); i++){
        try{runtime.level->draw_calls[i]();} catch(std::exception e){
            assert(false);
        }
    }
    BeginMode3D(*cam);
    for(size_t i =0; i<get_level().meshes.size(); i++){
        if(get_level().meshes[i].meshes.empty()|| !get_level().entities[i]){
            continue;
        }
        draw_mesh_comp(get_level().meshes[i], get_level().physics[i].trans.trans,get_level().physics[i].colliders[0].bb);
    } 
    for(int i =0; i<runtime.level->draw_calls_3d.size(); i++){
        try{runtime.level->draw_calls_3d[i]();} catch(std::exception e){
            assert(false);
        }
    }
    EndMode3D(); 

     EndTextureMode();
    BeginDrawing();  
    BeginShaderMode(postprocess); 
    DrawTextureRec(tex.texture, (Rectangle){ 0, 0, (float)tex.texture.width, (float)-tex.texture.height }, (Vector2){ 0, 0 }, WHITE); 
    EndShaderMode();
     DrawFPS(GetScreenWidth()-GetScreenWidth()/5,80);   
    DrawCircle(GetScreenWidth()/2, GetScreenHeight()/2, 5, GREEN);
    EndDrawing();
}
