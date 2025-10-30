#include "physics.h"
#include <stdatomic.h>
static PhysicsCompVec comps;
static u32Vec indexs;
enable_hash_type(u64, u32Vec);
void* physics_loop(void*);
atomic_int should_process_physics = false;
u64u32VecHashTable *grid = 0;
static double square_size =2.0;
static double min_x = 0.0;
static double min_y = 0.0;
static double min_z = 0.0;
static double max_x = 0.0;
static double max_y = 0.0;
static double max_z = 0.0;
typedef struct {
    Vec3 v0;
    Vec3 v1;
}Vec3Pair;
static inline i64 compute_position(Vec3 v){
    int64_t dx =(max_x -min_x)/square_size;
    int64_t dz = (max_z-min_z)/square_size;
    int64_t dy = (max_y -min_y)/square_size;
    int64_t px = (v.x-min_x)/square_size;
    int64_t py =(v.y-min_y)/square_size;
    int64_t pz =(v.z-min_z)/square_size;
    if(px>dx|| px<0 || py>dy || py<0 || pz>dz || pz<0){
        return -1;
    }
    int64_t x = px+py*dy+pz*dz*dy;
    return x;
}
static bool u64_equals(u64 l, u64 r){
    return l == r;
}
static void u32Vec_destroy(u32Vec * v){
    unmake((*v));
}
static void setup_grid(){	
        grid = u64u32VecHashTable_create(4096*16,(size_t (*)(u64))hash_long,u64_equals, (void(*)(u64*))no_op_void, u32Vec_destroy); 
        for(size_t j = 0; j<comps.length; j++){
            PhysicsComp i = comps.items[j];
            if(!i.can_ever_collide){
                continue;
            }
            Vec3 pos = i.trans.trans.translation;
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

    for(size_t i =0; i<comps.length; i++){
        int64_t p =compute_position(comps.items[i].trans.trans.translation); 

        u32Vec * v = u64u32VecHashTable_find(grid, p);
        if(v){
           v_append((*v), i);
        } else{
            u32Vec vec = make(frame_arena(), u32);
            //v_resize(vec, 1000);
            v_append(vec, i);
            u64u32VecHashTable_insert(grid, p,vec);
        }
    } 
}
Vec3Pair collision_response(double m1, Vec3 v1, double m2, Vec3 v2, Vec3 normal);
Vec3Pair angular_collision_response(double m1, Vec3 v1, double m2, Vec3 v2, Vec3 normal);
void physics_prepare_update(){
    comps.length =0;
    indexs.length =0;
    for(size_t i =0; i<ENTITY_COUNT; i++){
        if(!runtime.level->tags[i]){
            continue;
        }
        PhysicsComp p = get_physics_comps()[i];
        if(has_component((EntityRef){.index = i, .generation = runtime.level->generations[i]}, comp_physics)){
            v_append(indexs, i);
            v_append(comps,p); 
        }else{
			todo()
		}
    }
    should_process_physics = true;
}
OptCol physics_comp_check_collision(PhysicsComp a, PhysicsComp b){
    for(size_t i =0; i<a.collider_count; i++){
        for(size_t j =0; j<b.collider_count; j++){
            OptCol c= check_collision(a.colliders[i].bb, a.colliders[i].offset, a.trans, b.colliders[j].bb, b.colliders[j].offset, b.trans);
            if(c.is_valid){
                return c;
            }
        }
    }
    return (OptCol){0};
}
[[gnu::always_inline]]
static inline void update_pair(size_t i, size_t j, bool * did_hit){
        OptCol col = physics_comp_check_collision(comps.items[i], comps.items[j]);
        if(col.is_valid){
            *did_hit = true;
            comps.items[i].trans.trans.translation =Vec3_add(comps.items[i].trans.trans.translation, Vec3_scale(col.col.norm,(col.col.depth+0.01)));
            uint32_t i_gen = runtime.level->generations[i];
            uint32_t j_gen = runtime.level->generations[j];
            uint32_t ui = indexs.items[i];
            uint32_t uj = indexs.items[j];
            EntityRef iref = (EntityRef){.index = ui, .generation =i_gen};
            EntityRef jref = (EntityRef){.index = uj, .generation = j_gen};
            apply_damage(iref,jref,col.col.norm,11);
            apply_damage(jref, iref, Vec3_scale(col.col.norm,-1), 11);
            if(comps.items[i].destroy_on_impact || comps.items[j].destroy_on_impact){
                return;
            }
            Vec3Pair v = collision_response(comps.items[i].mass, comps.items[i].velocity, comps.items[j].mass, comps.items[j].velocity, Vec3_normalize(col.col.norm));
            //Vec3Pair v2 = angular_collision_response(comps.items[i].mass, comps.items[i].velocity, comps.items[i].trans.trans.translation,comps.items[j].mass, comps.items[j].velocity, comps.items[j].trans.trans.translation);
            comps.items[i].velocity = Vec3_scale(v.v0, 0.42);
            comps.items[j].velocity = Vec3_scale(v.v1,0.42); 
        }
}
u64 update_obj(size_t i, bool * did_hit){
        size_t count = 0;
        Vec3 v = comps.items[i].trans.trans.translation;
        u32Vec reached = make(frame_arena(), u32);
        for(int x = -1; x<2; x++){
            for(int y =-1; y<2; y++){
                for(int z =-1; z<2; z++){
                    Vec3 v0 = Vec3_add(v,Vec3_scale((Vec3){(double)x,(double)y,(double)z},square_size/2.0));
                    int64_t p = compute_position(v0);
                    if(p == -1){
                    continue;
                    }
                    u32Vec * vs = u64u32VecHashTable_find(grid,p);
                    if(vs){  
                        for(size_t idx =0; idx<vs->length; idx++){ 
                            u32 j= vs->items[idx];
                            if(i == j){
                                continue;
                            }
                            update_pair(i,j,did_hit); 
                            count += 1;
                            if(*did_hit){
                                return count;
                            }
                        }
                    } 
                }
            }
        }

    return count;
}
void update_physics(){    
    setup_grid(); 
    size_t count = 0;
    size_t max_count =0;
    for(size_t i =0; i<comps.length; i++){
        if(!comps.items[i].can_ever_collide){
            comps.items[i].trans.trans.translation= Vec3_add(comps.items[i].trans.trans.translation,Vec3_scale(comps.items[i].velocity,1/60.0));
            continue;
        } 
        double dist = Vec3_len(comps.items[i].velocity)*1./60;
        Vec3 p = comps.items[i].trans.trans.translation;
        double delta= 0.5;
        Vec3 delt  = Vec3_scale(Vec3_normalize(comps.items[i].velocity), 1./60.0);
        if(dist == 0.0){ 
            continue;
        }
        int dt = ceil((double)dist/delta);
        if(dt>4)dt =4;
        comps.items[i].trans.trans.rotation = Vec4_from_Vector4(
            QuaternionFromMatrix(
                    MatrixMultiply(
                        QuaternionToMatrix(
                            Vec4_to_Vector4(comps.items[i].trans.trans.rotation)),
                            QuaternionToMatrix(
                                QuaternionFromEuler(comps.items[i].angular_velocity.x, comps.items[i].angular_velocity.y, comps.items[i].angular_velocity.z)
                            )
                    )
            )
        );
        size_t c = 0;
        for (int j =0; j<dt; j++){
            if(j<dt-1){
                comps.items[i].trans.trans.translation = Vec3_add(comps.items[i].trans.trans.translation, delt);
            }else{
                comps.items[i].trans.trans.translation = Vec3_add(p, Vec3_scale(comps.items[i].velocity,1./60.0));
            }
            comps.items[i].trans.trans.rotation = Vec4_from_Vector4(QuaternionNormalize(Vec4_to_Vector4(comps.items[i].trans.trans.rotation)));
            bool did_hit = false;
            c+= update_obj(i, &did_hit); 
            if(did_hit){
                break;
            } 
        }
        if(c>max_count){
                max_count =c;
        }
        count+=c;
    }
}
void physics_finish_update(){
    while(should_process_physics){}
    for(size_t i =0; i<comps.length; i++){
        get_physics_comps()[indexs.items[i]] = comps.items[i];
    }
u64u32VecHashTable_unmake(grid);
    grid =0; 
}

static bool uint32_array_contains(uint32_t check, u32* to_check,size_t to_check_count){
    for(int i =0; i<to_check_count; i++){
        if(to_check[i] == check){
            return true;
        }
    }
    return false;
}
extern double check_collision_line_box(Vec3 start, Vec3 end, Trans trans,Trans offset, BoundingBox box);

OptEntityRef line_trance(Vec3 start, Vec3 end, u32 * to_ignore, size_t to_ignore_count){
    double min = 1000000000.0;
    u32 idx =0;
    bool hit = false;
    for(u64 i =0; i<ENTITY_COUNT; i++){
        if(runtime.level->tags[i]){
            if(!get_physics_comps()[i].is_valid) continue; 
            if(uint32_array_contains(i, to_ignore, to_ignore_count)){
                continue;
            }
            for (u64 j=0; j<get_physics_comps()[i].collider_count; j++){
                double h = check_collision_line_box(start, end, get_physics_comps()[i].trans.trans,get_physics_comps()[i].colliders[j].offset,get_physics_comps()[i].colliders[j].bb);
                if(h>0 && h<min){
                    idx = i;
                    min = h;
                    hit = true;
                }
            }
        }
    }
    if(hit){
        return (OptEntityRef){.is_valid = true, .ref =(EntityRef){.index = idx, .generation = runtime.level->generations[idx]}};
    }
    return (OptEntityRef){0};
}
void* physics_loop(void*v ){
    while(true){
        while(!should_process_physics){
        }
        update_physics();
        should_process_physics = false;
    }
    return 0;
}
EntityRefVec sphere_trace(Arena * arena,Vec3 start, double radius, uint32_t to_ignore[], size_t to_ignore_count){
	EntityRefVec out = make(arena, EntityRef);	
	for(int i=0; i<ENTITY_COUNT;i++){
		if(runtime.level->tags[i]){
			EntityRef e = entity_ref_from_index(i);
			
		}
	}
	return out;
}
