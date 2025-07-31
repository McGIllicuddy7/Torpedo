#include <string.h>
#include <tgmath.h>
#include "physics.h"


typedef struct {
    Vec3 v0;
    Vec3 v1;
}Vec3Pair;
typedef struct {
    Vec3 points [8];
}VertexSet;
typedef struct{
    Vec3 items[13];
}Vec3_13;
typedef struct{
    Vec3 items[6];
}Vec3_6;
VertexSet get_vertices(BoundingBox a, Trans offset, Trans a_trans){
    VertexSet verts = {{    
        (Vec3){1., 1., 1.},
        (Vec3){1., -1., 1.},
        (Vec3){1., 1., 1.},
        (Vec3){1., -1., 1.0},
        (Vec3){1., 1., -1.},
        (Vec3){1., -1., -1.},
        (Vec3){-1., 1., -1.},
        (Vec3){-1., -1., -1.0},   
    }};
    double dx = a.max.x - a.min.x;
    double dy = a.max.y - a.min.y;
    double dz = a.max.z - a.min.z;
    for(size_t i=0; i<8;i++) {
        double x = verts.points[i].x * dx / 2.;
        double y = verts.points[i].y * dy / 2.;
        double z = verts.points[i].z * dz / 2.;
        verts.points[i].x = x;
        verts.points[i].y = y; 
        verts.points[i].z = z;
    }
    for(size_t i =0; i<8; i++){
        Vec3 tmp = verts.points[i];
        tmp = Vec3_add(tmp,a_trans.translation);
        tmp = Vec3_add(tmp, 
            Vec3_from_Vector3(
                    Vector3Transform(
                       Vec3_to_Vector3(offset.translation),
                       QuaternionToMatrix(
                            QuaternionMultiply(
                                Vec4_to_Vector4(a_trans.rotation), 
                                Vec4_to_Vector4(offset.rotation)
                            )
                       )
                )
        ));
        verts.points[i] = tmp;
    }
    return verts;
}
bool Vec3_13_contains(Vec3_13 a, Vec3 v,size_t count){
    size_t idx = 0;
    while(idx <count){
        if (a.items[idx].x == v.x && a.items[idx].y == v.y && a.items[idx].z == v.z ){
            return true;
        }
        idx += 1;
    }
    return false;
}

Vec3_13 internal_get_normals(){
        Vec3_13 norms;
        for(size_t i = 0; i<13; i++){
            norms.items[i].x = 0;
            norms.items[i].y = 0;
            norms.items[i].z = 0;
        }
        int count = 0;
        int x = -1;
        int y = -1;
        int z = -1;
        while (x < 2) {
            while (y < 2 ){
                while (z < 2 ){
                    if (x == 0 && y == 0 && z == 0 ){
                        z += 1;
                        continue;
                    }
                    Vec3 v;
                    v.x =x;
                    v.y = y;
                    v.z = z;
                    Vec3 tmp = {-v.x, -v.y, -v.z};
 
                    if( Vec3_13_contains(norms, tmp,count)) {
                        z += 1;
                        continue;
                    }
                    norms.items[count] = v;
                    count += 1;
                    z += 1;
                }
                z = -1;
                y += 1;
            }
            y = -1;
            x += 1;
        }

        size_t idx = 0;
        while (idx < 13){
            double l = norms.items[idx].x * norms.items[idx].x
                + norms.items[idx].y * norms.items[idx].y
                + norms.items[idx].z * norms.items[idx].z; 
            norms.items[idx].x /= l;
            norms.items[idx].y /= l;
            norms.items[idx].z /= l;
            idx += 1;
        }
    return norms;
}

Vec3_13 get_normals(Trans a_trans, Trans a_off){
    const Vec3_13 base_normals = internal_get_normals();
    Vec3_13 normals = base_normals;
    Matrix rot = QuaternionToMatrix(QuaternionMultiply(Vec4_to_Vector4(a_trans.rotation),Vec4_to_Vector4(a_off.rotation)));
    for(size_t i =0; i<13; i++){
        normals.items[i] = Vec3_from_Vector3(Vector3Transform(Vec3_to_Vector3(normals.items[i]),rot));
    }
    return normals;
}
Vec3_6 get_normals_basic(Trans a_trans, Trans a_off){
    Vec3_6 normals = {{
        (Vec3){1.0, 0., 0.},
        (Vec3){-1., 0., 0.},
        (Vec3){0., 1., 0.},
        (Vec3){0., -1., 0.},
        (Vec3){0., 0., 1.},
        (Vec3){0., 0., -1.0},
    }
    };
    Quat v1 = a_trans.rotation;
    Quat v2 = a_off.rotation;
    Quat result = { v1.x*v2.x, v1.y*v2.y, v1.z*v2.z, v1.w*v2.w };
    Matrix rot = QuaternionToMatrix(Vec4_to_Vector4(result));
    for (size_t i= 0; i<6; i++) {
        normals.items[i] = Vec3_from_Vector3(Vector3Transform(Vec3_to_Vector3(normals.items[i]), rot));
    }
    for (size_t i= 0; i<6; i++) {
        normals.items[i]= Vec3_normalize(normals.items[i]);
    }
    return normals;
}
OptCol check_collision(
    BoundingBox a, 
    Trans a_off, 
    TransformComp a_trans, 
    BoundingBox b, 
    Trans b_off,
    TransformComp b_trans){
    {
        Vector3 a_loc = Vector3Add(Vector3Transform(Vec3_to_Vector3(a_off.translation), QuaternionToMatrix(Vec4_to_Vector4(a_trans.trans.rotation))),Vec3_to_Vector3(a_trans.trans.translation));
        Vector3 b_loc = Vector3Add(Vector3Transform(Vec3_to_Vector3(b_off.translation), QuaternionToMatrix(Vec4_to_Vector4(b_trans.trans.rotation))),Vec3_to_Vector3(b_trans.trans.translation));
        BoundingBox ta = a;
        BoundingBox tb = b;
        ta.min = Vector3Add(ta.min, a_loc);
        ta.max = Vector3Add(ta.max, a_loc);
        tb.min = Vector3Add(tb.min, b_loc);
        tb.max = Vector3Add(tb.max, b_loc);
        if(!CheckCollisionBoxes(ta, tb)){
            return (OptCol){0};
        }
    }
    VertexSet a_verts = get_vertices(a, a_off, a_trans.trans);
    VertexSet b_verts = get_vertices(b, b_off, b_trans.trans);
    Vec3_13 a_norms = get_normals(a_trans.trans, a_off);
    Vec3_13 b_norms = get_normals(b_trans.trans, b_off);
    Vec3 norms[26] = {0};
    size_t idx =0;
    for(size_t i =0; i<13; i++){
        norms[idx] = a_norms.items[i];
        idx++;
    }
    for(size_t i =0; i<13; i++){
        norms[idx] = b_norms.items[i];
        idx++;
    }
    Vec3 col_norm = (Vec3){0., 0., 0.};
    double col_depth = 1000000.0;
    for(size_t i =0; i<26; i++){
        double a_max = -1000000.0;
        double a_min = -a_max;
        double b_max = a_max;
        double b_min = -b_max;
        for(size_t j =0; j<13; j++){
            double a_dot = Vec3_dot_product(a_verts.points[j], norms[i]);
            if (a_dot > a_max ){
                a_max = a_dot;
            }
            if (a_dot < a_min ){
                a_min = a_dot;
            }
        }
        for(size_t j =0; j<13; j++){
            double b_dot = Vec3_dot_product(b_verts.points[j], norms[i]);
            if (b_dot > b_max ){
                b_max = b_dot;
            }
            if (b_dot < b_min ){
                b_min = b_dot;
            }
        }
        if (a_min > b_max + 0.001 || b_min > a_max + 0.001 ){
            return (OptCol){0};
        }
    }
    idx =0;
    Vec3_6 a_norms_basic = get_normals_basic(a_trans.trans, a_off);
    Vec3_6 b_norms_basic = get_normals_basic(b_trans.trans, b_off);
    Vec3 trans[12] = {0};
    for(size_t i =0; i<6; i++){
        trans[idx] = a_norms_basic.items[i];
        idx++;
    }
    for(size_t i =0; i<6; i++){
        trans[idx] = b_norms_basic.items[i];
        idx++;
    }
    for (size_t id = 0; id<12; id++) {
        Vec3 i = trans[id];
        double a_max = -1000000.0;
        double a_min = -a_max;
        double b_max = a_max;
        double b_min = -b_max;
        for (size_t jd = 0; jd<8; jd++){
            Vec3 j = a_verts.points[jd];
            double a_dot =Vec3_dot_product(j, i);
            if( a_dot > a_max ){
                a_max = a_dot;
            }
            if (a_dot < a_min ){
                a_min = a_dot;
            }
        }
        for (size_t jd =0; jd<8; jd++){
            Vec3 j = b_verts.points[jd];
            double b_dot = Vec3_dot_product(j,i);
            if (b_dot > b_max ){
                b_max = b_dot;
            }
            if (b_dot < b_min ){
                b_min = b_dot;
            }
        }
        double da = abs(b_min - a_max);
        double db = abs(a_min - b_max);
        double del =  da > db? db : da ;
        if(del < col_depth) {
            col_depth = del;
            col_norm = i;
        }
    }
    Col out;
    out.norm = Vec3_scale(Vec3_normalize(col_norm),-1);
    out.depth = col_depth;
    return (OptCol){.is_valid = true, .col = out};
}

Vec3Pair collision_response(
    double m1,
    Vec3 v1,
    double m2,
    Vec3 v2,
    Vec3 normal){
    assert(Vec3_len(normal)>0.0);
    Vec3 n_0 = normal;
    normal = Vec3_normalize(normal);
    Vec3 center_momentum = Vec3_add(Vec3_scale(v1 ,m1) ,Vec3_scale( v2 , m2));
    Vec3 momentum_1 = Vec3_sub(v1 , Vec3_scale(center_momentum,1./(m1+m2)));
    Vec3 momentum_2 = Vec3_sub(v2 , Vec3_scale(center_momentum,1./(m1+m2)));
    momentum_1 = Vec3_from_Vector3(Vector3Reflect(Vec3_to_Vector3(momentum_1), Vec3_to_Vector3(normal)));
    momentum_2 = Vec3_from_Vector3(Vector3Reflect(Vec3_to_Vector3(momentum_2), Vec3_to_Vector3(normal)));
    Vec3 out1 = Vec3_add(momentum_1, Vec3_scale(center_momentum,1./(m1+m2) ));
    Vec3 out2 = Vec3_add(momentum_2 , Vec3_scale(center_momentum,1./(m1+m2)));
    Vec3Pair out; 
    out.v0 = out1;
    out.v1 = out2;
    return out;
}
static inline double min(double a, double b){
    if(a<b) return a;
    return b;
}
static inline double max(double a, double b){
    if(a<b) return b;
    return a;
}
double check_collision_aabb(Vec3 start, Vec3 direction, BoundingBox box){
    double tmin = 0.0, tmax = INFINITY; 
    double t1x = (box.min.x -start.x) * 1.0/direction.x;
    double t2x = (box.max.x - start.x) * 1.0/direction.x;
    tmin = max(tmin, min(min(t1x, t2x),tmax));
    tmax = min(tmax, max(max(t1x, t2x),tmin));
    double t1y = (box.min.y -start.y) * 1.0/direction.y;
    double  t2y = (box.max.y - start.y) * 1.0/direction.y;
    tmin = max(tmin, min(min(t1y, t2y),tmax));
    tmax = min(tmax, max(max(t1y, t2y),tmin)); 
    double t1z = (box.min.z -start.z) * 1.0/direction.z;
    double t2z = (box.max.z - start.z) * 1.0/direction.z;
    tmin = max(tmin, min(min(t1z, t2z),tmax));
    tmax = min(tmax, max(max(t1z, t2z),tmin)); 
    if(tmin>tmax){
        return -1;
    } 
    Vec3 p = Vec3_add(start,Vec3_scale(direction,tmin));
    if(p.x>box.max.x || p.x<box.min.x || p.y>box.max.y|| p.y<box.min.y || p.z>box.max.z || p.z<box.min.z){
        return -1;
    }
    return tmin; 
}
double check_collision_line_box(Vec3 start, Vec3 end, Trans trans,Trans offset, BoundingBox box){ 
    Matrix mat = QuaternionToMatrix(Vec4_to_Vector4(trans.rotation));
    Matrix inv = MatrixInvert(mat);
    Vec3 s = Vec3_from_Vector3(Vector3Transform(Vec3_to_Vector3(Vec3_sub(start,trans.translation)),inv));
     Vec3 e= Vec3_from_Vector3(Vector3Transform(Vec3_to_Vector3(Vec3_sub(end,trans.translation)),inv));

    e = Vec3_add(e,trans.translation);
    s = Vec3_add(s,trans.translation);
    box.min = Vector3Add(box.min,Vec3_to_Vector3(trans.translation));
    box.max =  Vector3Add(box.max,Vec3_to_Vector3(trans.translation));
    Vec3 dir =Vec3_normalize(Vec3_sub(e,s));
    double h = check_collision_aabb(s,dir, box);
    return h;
}


