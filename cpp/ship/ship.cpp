#include "ship.hpp"
namespace Torpedo{
void ship_comp_update(ShipComp* comp){
	
	Matrix base =  QuaternionToMatrix(comp->parent.get()->get_physics().trans.trans.rotation);
	Matrix rot_matrix= QuaternionToMatrix(comp->rotation_input)*base;
    	comp->parent.get()->get_physics().trans.trans.rotation =  Quat::from(QuaternionFromMatrix(rot_matrix));
    	comp->parent.get()->get_physics().velocity+= comp->movement_input;
}


}
