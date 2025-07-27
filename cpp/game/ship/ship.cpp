#include "ship.hpp"
namespace Torpedo{
void ShipComp::private_update_non_homing(){
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
	   Vec3 delta2 = desired_position - parent.get()->get_location();
	    if(Vector3LengthSqr(delta2)<0.1){
		    parent.get()->get_physics().trans.trans.translation = desired_position;
	        if(Vector3LengthSqr(parent.get()->get_physics().velocity) <= 0.1){
		   parent.get()->get_physics().velocity = (Vec3){0, 0,0 };
		}
	    }else{
		Vec3 dn = Vec3::from(Vector3Normalize(delta2));
		Vec3 v = parent.get()->get_physics().velocity;
		Vec3 dv = Vec3::from(Vector3Normalize(v));
		double d =  Vector3DotProduct(dn,dv);
		Vec3 vout = (dn -Vec3{dv.x*d, dv.y*d, dv.z*d})*(1-d);
		vout += dn*d;
		parent.get()->get_physics().velocity += Vector3Normalize(vout)*accel_value*1./60.;
	    }
    } else{	
	if(stablized_velocity && Vector3Length(movement_input)<0.001){
	    Vec3 vec = parent.get()->get_physics().velocity;
	    if(Vector3Length(vec)<0.001){
		vec = Vec3{0,0,0};
	    } else{
		parent.get()->get_physics().velocity -= Vec3::from(Vector3Normalize(vec)*accel_value*1./60.0);
	    }
	}else{
	    parent.get()->get_physics().velocity+= to_global_vector(
		Vec3::from(Vector3(movement_input)*accel_value*1./60.0),
		parent.get()->get_forward_vector(), 
		parent.get()->get_right_vector(), 
		parent.get()->get_up_vector()
	    );
	}
    }

}
void ShipComp::private_update_homing(){
    Entity * t = target.get();
    Vec3 to_target = t->get_location()-parent.get()->get_location();
    use_desired_position = true;
    desired_position = t->get_location();
    Vec3 dp = desired_position; 
    parent.get()->get_physics().trans.trans.rotation= QuaternionFromMatrix(MatrixLookAt(parent.get()->get_location(), t->get_location(),parent.get()->get_velocity()));
    private_update_non_homing(); 
}
void ShipComp::update(){	 
    if(use_target){
	private_update_homing();
    }else{
	private_update_non_homing();
    }
}
void ShipComp::on_damage(Vec3 direction, double amount){
    health -= amount;
    char buff[100];
    snprintf(buff, 99, "took %f damage, health:%f",amount, health);
    log(buff, 2.0);
    if(health<=0.0){
	spawn_explosion((parent.get()->get_location()-Vec3::from(Vector3Normalize(parent.get()->get_velocity()))), 100.0);
	destroy_entity(parent);
    }
}
void ShipComp::serialize(Serializer*ser) const{
    ser->serialize_trivial(*this);
}
ShipComp ShipComp::deserialize(Deserializer*des){
    return des->deserialize_trivial<ShipComp>();
}


}
