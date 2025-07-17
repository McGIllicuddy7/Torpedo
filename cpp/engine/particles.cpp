#include "particles.hpp"
namespace Torpedo{
    RepeaterObject::RepeaterObject(double dur, std::function<void(double)> callee){
	duration = dur;
	to_call = callee;
    }
    void RepeaterObject::on_tick(){
	duration -= 1.0/60.0;
	to_call(duration);
	if(duration<0.0){
	    destroy_entity(get_as_ref(this));
	}
    }
    EntityRef spawn_repeating(double duration, std::function<void()> to_call){
	auto p = [to_call](double nothing){(void)nothing, to_call();};
	return spawn_repeating(duration,p);
    }
    EntityRef spawn_repeating(double duration, std::function<void(double)> to_call){
	return create_entity<RepeaterObject>(duration, to_call);
    }
}
