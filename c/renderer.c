#include "renderer.h"
#include "base.h"
#include "utils.h"
extern Arena * arena_create_sized(size_t count);
void process_3d_draw_calls(){
    for(size_t i =0; i<runtime.level->draw3d_calls.length; i++){
        DrawCall3D d = runtime.level->draw3d_calls.items[i];
        if(d.draw_call_type ==draw_call_line){
            DrawLine3D(d.draw_call_line_info.start, d.draw_call_line_info.end, d.color);
        }else if(d.draw_call_type == draw_call_cube){
            DrawCube(d.draw_call_cube_info.pos, d.draw_call_cube_info.w, d.draw_call_cube_info.h, d.draw_call_cube_info.d, d.color);
        }else if(d.draw_call_type == draw_call_sphere){
            DrawSphere(d.draw_call_sphere_info.pos, d.draw_call_sphere_info.r, d.color);
        }
    }
    runtime.level->draw3d_calls.length =0;
    runtime.level->draw3d_calls.capacity =0;
    runtime.level->draw3d_calls.items =0;
}
void process_draw_calls(){
    for(size_t i =0; i<runtime.level->draw_calls.length; i++){
        DrawCall d = runtime.level->draw_calls.items[i];
        if(d.draw_call_type ==draw_call_rect){
            DrawRectangle(d.draw_call_rect_info.x, d.draw_call_rect_info.y, d.draw_call_rect_info.width, d.draw_call_rect_info.height,d.color);
        }else if(d.draw_call_type == draw_call_text){
            DrawText(d.draw_call_text_info.text, d.draw_call_text_info.x, d.draw_call_text_info.y, d.draw_call_text_info.height, d.color);
        }
        else if(d.draw_call_type == draw_call_circle){
            DrawCircle(d.draw_call_circ_info.x, d.draw_call_circ_info.y, d.draw_call_circ_info.r, d.color);
        }
    }
    runtime.level->draw_calls.length =0;
    runtime.level->draw_calls.capacity =0;
    runtime.level->draw_calls.items =0;
}
static void draw_mesh_comp( Arena * arena,MeshComp cmp, Trans trans, BoundingBox b){
    for(size_t i =0 ; i<cmp.mesh_count; i++){ 
        Model *md = StringModelHashTable_find(runtime.level->models, new_string(arena,cmp.meshes[i].string));
        Matrix old = md->transform;
        md->transform = QuaternionToMatrix(Vec4_to_Vector4(trans.rotation)); 
        Shader s = md->materials[0].shader;
        md->materials[0].shader = get_level()->shader;
        Vec3 loc = Vec3_add(trans.translation,to_global_vector(cmp.meshes[i].offset.translation, get_forward_vector(trans), get_left_vector(trans), get_up_vector(trans)));
        DrawModel(*md, Vec3_to_Vector3(loc),1.0,cmp.meshes[i].color);//        printf("%f,%f,%f\n", loc.x, loc.y, loc.z);
        md->transform = old;
        md->materials[0].shader = s;
        b.min = Vector3Add( b.min,Vec3_to_Vector3(trans.translation));
        b.max = Vector3Add( b.max,Vec3_to_Vector3(trans.translation));
        //DrawBoundingBox(b, GREEN);
    }
}
void game_render(Camera * cam){
    if(!entity_is_valid(runtime.level->player_entity)){
        UpdateCamera(cam, CAMERA_FREE);
    } else{
        Transform base = Trans_to_Transform(get_physics_comp(runtime.level->player_entity)->trans.trans);
        auto m =QuaternionToMatrix(base.rotation);
        Transform offset = Trans_to_Transform(runtime.level->cam_player_offset);
        cam->position = Vector3Add(base.translation, Vector3Transform(offset.translation, m));
        Matrix mat = MatrixMultiply(QuaternionToMatrix(offset.rotation),QuaternionToMatrix(base.rotation));
        cam->target = Vector3Add(Vector3Transform((Vector3){1,0,0},mat), cam->position);
        cam->up = Vector3Transform((Vector3){0,0,1},mat);
    }
    BeginDrawing(); 
    ClearBackground((Color){16, 16, 32, 255});
    BeginShaderMode(runtime.level->shader);
    BeginMode3D(*cam);
    rlSetClipPlanes(0.0001, 10000.0);
   // rlDisableBackfaceCulling();

//    rlSetClipPlanes(0.01, 5000000);
    Arena *arena = arena_create_sized(4096*1024);
 
    for(size_t i =0; i<ENTITY_COUNT; i++){
        if(get_mesh_comps()[i].mesh_count == 0|| !runtime.level->tags[i] || !(get_level()->owned_comps[i] & comp_model)){
            continue;
        }
        draw_mesh_comp(arena,get_mesh_comps()[i],get_physics_comps()[i].trans.trans,get_physics_comps()[i].colliders[0].bb);
    } 
    process_3d_draw_calls();
    arena_destroy(arena);
    EndMode3D();
    EndShaderMode();
    process_draw_calls();
    DrawFPS(900, 20);
    EndDrawing();

}
