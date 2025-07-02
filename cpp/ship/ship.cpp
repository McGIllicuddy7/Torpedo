#include "ship.hpp"
namespace Torpedo{
void ShipComp::update(){	
    if(use_desired_rotation){
	Vec3 base = Vec3::from(QuaternionToEuler(parent.get()->get_physics().trans.trans.rotation));	
	Vec3 target = Vec3::from(QuaternionToEuler(desired_rotation));
	Vec3 delta = base-target;
	Quaternion rot;
    	if(Vector3Length(delta)<0.01){
	   rot = desired_rotation;
	} else{
	    Matrix base =  QuaternionToMatrix(parent.get()->get_physics().trans.trans.rotation);
	    delta *= 0.1;
	    Matrix rot_matrix = QuaternionToMatrix(QuaternionFromEuler(delta.x, delta.y, delta.z));
	    parent.get()->get_physics().trans.trans.rotation =  Quat::from(QuaternionFromMatrix(rot_matrix));
	}
	parent.get()->get_physics().trans.trans.rotation = rot;
	Vec3 norm = Vec3::from(Vector3Normalize(delta));

    }else{
	Matrix base =  QuaternionToMatrix(parent.get()->get_physics().trans.trans.rotation);
	Matrix rot_matrix= QuaternionToMatrix(rotation_input)*base;
	parent.get()->get_physics().trans.trans.rotation =  Quat::from(QuaternionFromMatrix(rot_matrix));
    }
    if (use_desired_position){
	   ;
    } else{	
	parent.get()->get_physics().velocity+= to_global_vector(
	    movement_input, 
	    parent.get()->get_forward_vector(), 
	    parent.get()->get_right_vector(), 
	    parent.get()->get_up_vector()
	);
    }
}


}
