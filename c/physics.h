#pragma once
#include "base.h"
#include "level.h"
typedef struct{
	Col col;
	bool is_valid;
}OptCol;
REFLECT_STRUCT(OptCol, REFLECT_FIELD(OptCol, Col, col), REFLECT_FIELD(OptCol, bool, is_valid))
typedef struct{
	EntityRef ref;
	bool is_valid;
}OptEntityRef;
REFLECT_STRUCT(OptEntityRef, REFLECT_FIELD(OptEntityRef, EntityRef, ref), REFLECT_FIELD(OptEntityRef, bool,is_valid))
OptCol check_collision(BoundingBox a, Trans a_off, TransformComp a_trans, BoundingBox b, Trans b_off, TransformComp b_trans);
void physics_prepare_update();
void update_physics();
void physics_finish_update();
OptEntityRef line_trace(Vec3 start, Vec3 end, uint32_t  to_ignore[], size_t to_ignore_count);
