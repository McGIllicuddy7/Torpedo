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
 
    
//    BeginTextureMode(tex);
    BeginDrawing();  
    ClearBackground(Color{1, 1, 64,255});
    static bool stars_init = false;
    static std::vector<Vec3> positions;
    static Model star;
    if(!stars_init){
        star = LoadModelFromMesh(GenMeshSphere(0.5, 3,3)); 
        for(int i =0; i<1000; i++){
            int x= rand()%1000-500;
            int y = rand()%1000-500;
            int z = rand()%1000-500;
            x*= 1;
            y*= 1;
            z*= 1;
            positions.push_back(Vec3{(double)x,(double)y,(double)z});
        }
        stars_init = true;
    }
    BeginMode3D(*cam);
    rlSetClipPlanes(0.01, 5000000);
    for(size_t i =0; i<positions.size(); i++){
        DrawModel(star, positions[i]+cam->position,1.0,WHITE);
    }
    DrawSphere(Vec3{500000,0,0},100000.0, YELLOW);
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
    for(int i =0; i<runtime.level->draw_calls.size(); i++){
        try{runtime.level->draw_calls[i]();} catch(std::exception e){
            assert(false);
        }
    }

 //    EndTextureMode();

  /*  ClearBackground(BLACK);
    BeginShaderMode(postprocess); 
    DrawTextureRec(tex.texture, (Rectangle){ 0, 0, (float)tex.texture.width, (float)-tex.texture.height }, (Vector2){ 0, 0 }, WHITE); 
    EndShaderMode();*/
     DrawFPS(GetScreenWidth()-GetScreenWidth()/5,80);   
    DrawCircle(GetScreenWidth()/2, GetScreenHeight()/2, 5, GREEN);
    EndDrawing();
}
