#pragma once 
#include "utils.hpp"
namespace Torpedo{
    class Entity{ 
        public:
        uint32_t id;
        virtual ~Entity();
        virtual void on_tick();
        virtual void apply_damage(const char * comp, double damage);
        virtual vector<unsigned char> serialize();
        virtual void set_velocity(Vec3 vel);
        virtual Vec3 get_velocity();
        virtual PhysicsComp& get_physics();
        virtual MeshComp& get_mesh();
        static unique_ptr<Entity> deserialze(std::string_view name,vector<unsigned char> bytes);
	virtual Vec3 get_forward_vector();
	virtual Vec3 get_right_vector();
	virtual Vec3 get_up_vector();
	virtual Vec3 get_location();
	virtual Quat get_rotation();
    }; 
    class Level{
        public:
	Entity * player;
	std::vector<uint32_t> destroy_queue;
        unordered_map<string, Model> models;
        std::vector<std::unique_ptr<Entity>> entities;
        std::vector<uint32_t> generations;
        std::vector<MeshComp> meshes;
        std::vector<PhysicsComp> physics;
    };
    class Runtime{
        public:
        unique_ptr<Level> level;
    };
extern Runtime runtime;
    class EntityRef{
public:
        uint32_t index;
        uint32_t generation;
public:
        static inline EntityRef create(uint32_t index, uint32_t generation){
            EntityRef out;
            out.index = index;
            out.generation = generation;
            return out;
        }
        inline bool is_valid(){
            return runtime.level->entities[index] && runtime.level->generations[index] == generation;
        }
        inline Entity& operator->(){
            assert(is_valid());
            return *runtime.level->entities[index];
        } 
        inline Entity & operator*(){
            assert(is_valid());
            return *runtime.level->entities[index];
        }
        inline Entity * get(){
            assert(is_valid());
            return runtime.level->entities[index].get();
         }
        template<typename T> T* downcast(){
            Entity * e= get(); 
            return dynamic_cast<T>(e);
        } 
    };

void mainloop(const char * level);
void setup();
Level & get_level();
void load_level(const char* path);
    template<typename T, typename...Args>EntityRef create_entity(Args...args){
        for(size_t i =0; i<runtime.level->entities.size(); i++){
            if(!runtime.level->entities[i]){
                runtime.level->entities[i] = std::make_unique<T>(args...);
                runtime.level->generations[i]+=1;
                runtime.level->entities[i]->id = i;  
                return EntityRef::create(i, runtime.level->generations[i]);
            }
        }
        runtime.level->entities.push_back(std::make_unique<T>(args...));
        runtime.level->generations.push_back(0);
        runtime.level->physics.push_back(PhysicsComp{});
        runtime.level->meshes.push_back(MeshComp{});
        runtime.level->entities[runtime.level->entities.size()-1]->id = runtime.level->entities.size()-1;
        return EntityRef::create(runtime.level->entities.size()-1, 0);
    }
void destroy_entity(EntityRef ref);

EntityRef create_cube(Vec3 location, Vec3 scale, Vec3 velocity, Color color, Vec3 angular= Vec3{0,0,0});
void set_player_entity(EntityRef ref);
inline EntityRef get_as_ref(Entity * ptr){
	return EntityRef::create(ptr->id, get_level().generations[ptr->id]);
}
}





