#include "ship.h"
//returns the next acceleration
Vec3 path_integral(Vec3 pos, Vec3 desiredPos, Vec3 vel,float acc){
	
}
void ai_update(EntityRef ship, ShipComp * s){
	if(entity_is_valid(s->target)){
		if(has_view_to(ship, s->target)){
			todo();
		}
	}else{
		todo();
	}

}
