#pragma once
#include "level.hpp"
#include <functional>
namespace Torpedo{
class RepeaterObject: public Entity{
    double duration;
    std::function<void (double)> to_call;
public:
    RepeaterObject(double dur, std::function<void(double)> callee);
    virtual void on_tick();
};
EntityRef spawn_repeating(double duration, std::function<void()>to_call);
EntityRef spawn_repeating(double duration, std::function<void(double)>to_call);
};
