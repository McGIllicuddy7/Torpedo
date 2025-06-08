#include "physics.hpp"
#include <string.h>
#include "../utils.hpp"
using namespace Torpedo;
array<Vec3, 8> get_vertices(BoundingBox a,Trans offset, Trans a_trans) {
    std::array<Vec3, 8> verts = {
        Vec3{1., 1., 1.},
        Vec3{1., -1., 1.},
        Vec3{1., 1., 1.},
        Vec3{1., -1., 1.0},
        Vec3{1., 1., -1.},
        Vec3{1., -1., -1.},
        Vec3{-1., 1., -1.},
        Vec3{-1., -1., -1.0},
    };
    double dx = a.max.x - a.min.x;
    double dy = a.max.y - a.min.y;
    double dz = a.max.z - a.min.z;
    for(auto &i: verts) {
        double x = i.x * dx / 2.;
        double y = i.y * dy / 2.;
        double z = i.z * dz / 2.;
        i.x = x;
        i.y = y; 
        i.z = z;
    }
    for(auto &i:verts){
        Vec3 tmp = i;
        tmp += a_trans.translation;
        tmp += Vector3Transform(offset
            .translation,QuaternionToMatrix(a_trans.rotation * offset.rotation));
        i = tmp;
    }
    return verts;
}
template <size_t COUNT>constexpr bool vec_contains(array<Vec3, COUNT>a, Vec3 v,size_t count){
    size_t idx = 0;
    while(idx <count){
        if (a[idx].x == v.x && a[idx].y == v.y && a[idx].z == v.z ){
            return true;
        }
        idx += 1;
    }
    return false;
}
constexpr array<Vec3, 13> internal_get_normals(){
        std::array<Vec3, 13> norms;
        for(size_t i = 0; i<13; i++){
            norms[i].x = 0;
            norms[i].y = 0;
            norms[i].z = 0;
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
 
                    if( vec_contains(norms, tmp,count)) {
                        z += 1;
                        continue;
                    }
                    norms[count] = v;
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
        while (idx < norms.size() ){
            double l = norms[idx].x * norms[idx].x
                + norms[idx].y * norms[idx].y
                + norms[idx].z * norms[idx].z; 
            norms[idx].x /= l;
            norms[idx].y /= l;
            norms[idx].z /= l;
            idx += 1;
        }
    return norms;
}
array<Vec3,13> get_normals(Trans a_trans, Trans a_off)  {
    array<Vec3, 13> normals = internal_get_normals();
    Matrix rot = QuaternionToMatrix(a_trans.rotation * a_off.rotation);
    for (auto & i:normals ){
        i = Vector3Transform(i,rot);
//        printf("%f,%f, %f\n", i.x, i.y, i.z);
    }
   return normals;
}
array<Vec3, 6>get_normals_basic(Trans a_trans, Trans a_off){
    array<Vec3, 6> normals = {
        Vec3{1.0, 0., 0.},
        Vec3{-1., 0., 0.},
        Vec3{0., 1., 0.},
        Vec3{0., -1., 0.},
        Vec3{0., 0., 1.},
        Vec3{0., 0., -1.0},
    };
    Quat v1 = a_trans.rotation;
    Quat v2 = a_off.rotation;
    Quat result = { v1.x*v2.x, v1.y*v2.y, v1.z*v2.z, v1.w*v2.w };
    Matrix rot = QuaternionToMatrix(result);
    for (auto &i:normals) {
        i = Vector3Transform(i, rot);
    }
    for (auto & i: normals) {
        i = Vector3Normalize(i);
    }
    return normals;
}

std::optional<Col>check_collision(
    BoundingBox a,
    Trans a_off,
    TransformComp a_trans,
    BoundingBox b,
    Trans b_off,
    TransformComp b_trans
) {
    {
        auto a_lock = Vector3Transform( a_off
            .translation
            ,QuaternionToMatrix(a_trans.trans.rotation))
            + a_trans.trans.translation;
        auto b_lock= Vector3Transform( b_off
            .translation
            ,QuaternionToMatrix(b_trans.trans.rotation))
            + b_trans.trans.translation;
        auto ta = a;
        auto tb = b;
        BoundingBox a = BoundingBox {
            ta.min + a_lock,
            ta.max + a_lock,
        };
        BoundingBox b = BoundingBox {
            tb.min + b_lock,
            tb.max + b_lock,
        };
        if (!CheckCollisionBoxes(a, b)) {
            return optional<Col>{};
        }
    }
    auto  a_verts = get_vertices(a, a_off, Trans::from(a_trans.trans));
    auto b_verts = get_vertices(b, b_off, Trans::from(b_trans.trans));
    auto a_norms = get_normals(Trans::from(a_trans.trans), a_off);
    auto b_norms = get_normals(Trans::from(b_trans.trans), b_off);
    array<Vec3, 26> norms;
    size_t idx = 0;
    for(auto i: a_norms) {
        norms[idx] = i;
        idx += 1;
    }
    for(auto i: b_norms) {
        norms[idx] = i;
        idx += 1;
    }
    Vec3 col_norm = Vec3{0., 0., 0.};
    double col_depth = 1000000.0;
    for(auto i:norms) {
        double a_max = -1000000.0;
        double a_min = -a_max;
        double b_max = a_max;
        double b_min = -b_max;
        for(auto j:a_verts) {
            double a_dot = Vector3DotProduct(j, i);
            if (a_dot > a_max ){
                a_max = a_dot;
            }
            if (a_dot < a_min ){
                a_min = a_dot;
            }
        }
        for (auto j:b_verts){
            double b_dot = Vector3DotProduct(j,i);
            if (b_dot > b_max ){
                b_max = b_dot;
            }
            if( b_dot < b_min ){
                b_min = b_dot;
            }
        }
        if (a_min > b_max + 0.001 || b_min > a_max + 0.001 ){
            return optional<Col>{};
        }
    }
    idx = 0;
    auto a_norms_basic = get_normals_basic(Trans::from(a_trans.trans), a_off);
    auto b_norms_basic = get_normals_basic(Trans::from(b_trans.trans), b_off);
    array<Vec3, 12> trans;
    for( auto i:a_norms_basic) {
        trans[idx] = i;
        idx += 1;
    }
    for(auto i:b_norms_basic) {
        trans[idx] = i;
        idx += 1;
    }
    for (auto i:trans) {
        double a_max = -1000000.0;
        double a_min = -a_max;
        double b_max = a_max;
        double b_min = -b_max;
        for (auto j:a_verts ){
            double a_dot =Vector3DotProduct(j, i);
            if( a_dot > a_max ){
                a_max = a_dot;
            }
            if (a_dot < a_min ){
                a_min = a_dot;
            }
        }
        for (auto j : b_verts ){
            double b_dot = Vector3DotProduct(j,i);
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
    out.norm = Vector3Negate(Vector3Normalize(col_norm));
    out.depth = col_depth;
    return out;
}
std::array<Vec3, 2> collision_response(
    double m1,
    Vec3 v1,
    double m2,
    Vec3 v2,
    Vec3 normal) {
    assert(Vector3Length(normal)>0.0);
    auto n_0 = normal;
    normal = Vec3::from(Vector3Normalize(normal));
    auto center_momentum = v1 * m1 + v2 * m2;
    auto momentum_1 = v1 - center_momentum/(m1+m2);
    auto momentum_2 = v2 - center_momentum/(m1+m2);
    momentum_1 = Vector3Reflect(momentum_1, normal);
    momentum_2 = Vector3Reflect(momentum_2, normal);
    Vec3 out1 = Vec3::from(momentum_1+ center_momentum/(m1+m2) );
    Vec3 out2 = Vec3::from(momentum_2 + center_momentum/(m1+m2));
    std::array<Vec3, 2> out;
    out[0] = out1;
    out[1] = out2;
    return out;
}
std::array<Vec3, 2> angular_collision_response(
    double m1,
    Vec3 v1,
    Vec3 p1,
    double m2,
    Vec3 v2,
    Vec3 p2) {
    std::array<Vec3, 2> out;
    return out;
}
