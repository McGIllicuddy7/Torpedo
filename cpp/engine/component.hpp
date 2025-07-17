#pragma once 
#include "utils.hpp"
#include <vector>
#include <optional>
namespace Torpedo{
    class IComp{
        public:
        virtual void update();
        virtual ~IComp();
    };

    class ICompList{
        virtual void update();
        virtual ~ICompList();
        virtual void entity_created(size_t index);
        virtual void entity_destroyed(size_t index);
        virtual void comp_created(void * v);
        virtual void  * get_comp(size_t index);
    };
    template <typename T> class ComponentList: ICompList{
        std::vector<std::optional<T>> comps;
        public: 
        virtual void update(){
            for(size_t i =0; i<comps.size(); i++){
                if(comps[i]) comps[i].update();
            }
        }
        virtual ~ComponentList(){

        }
        virtual void entity_created(size_t index){  
            if(index>=comps.size()){
                while(comps.size()<index){
                    comps.push_back(std::optional<T>{});
                }
            }
        }
        virtual void entity_destroyed(size_t index){
                
        }
    };
}
