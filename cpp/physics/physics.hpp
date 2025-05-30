#pragma once
#include "../utils.hpp"
std::optional<Torpedo::Col>check_collision(
    BoundingBox a,
    Torpedo::Trans a_off,
    Torpedo::TransformComp a_trans,
    BoundingBox b,
    Torpedo::Trans b_off,
    Torpedo::TransformComp b_trans
);
void physics_prepare_update();
void update_physics();
void physics_finish_update();
