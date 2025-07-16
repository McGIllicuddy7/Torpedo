#pragma once 
#include "utils.hpp"
#include <functional>
#include <mutex>

namespace Torpedo{
enum Tag:uint32_t{
    tag_movable = 0b1,
    tag_on_fire = 0b10,
    tag_ship = 0b100,
    tag_projectile = 0b1000,
    tag_pressurized = 0b10000,
    tag_interactable = 0b100000,
};
    class Entity{ 
        public:
        uint32_t tags = 0;
        uint32_t id; 
        virtual ~Entity();
        virtual void on_tick();
        virtual void on_damage(Vec3 incoming_direction, double damage); 
        virtual void set_velocity(Vec3 vel);
        virtual Vec3 get_velocity();
        virtual PhysicsComp& get_physics();
        virtual MeshComp& get_mesh(); 
	virtual Vec3 get_forward_vector();
	virtual Vec3 get_right_vector();
	virtual Vec3 get_up_vector();
	virtual Vec3 get_location();
	virtual Quat get_rotation();
        bool has_tag(Tag tag)const ; 
        void add_tag(Tag tag);
        void remove_tag(Tag tag);
        virtual void serialize(Serializer * ser) const;
        static Entity  deserialize(Deserializer * des);
        static Entity * interface_deserialize(Deserializer&des);
    }; 
    Register(Entity, Entity);
 
    enum class EventType{
        ApplyDamage,
    };
    struct Event{
        uint32_t target_idx;
        uint32_t target_generation; 
        uint32_t cause_idx;
        uint32_t cause_generation;
        EventType event_type;
        struct ApplyDamage{
            Vec3 direction;
            Vec3 point;
            double damage;
        };
        union{
            ApplyDamage apply_damage;
        };
    };
    class Level{
        public:
	Entity * player;
        std::vector<Event> event_queue;
	std::vector<uint32_t> destroy_queue;
        unordered_map<string, Model> models;
        unordered_map<string, std::vector<string>> mesh_textures;
        unordered_map<string, Texture> textures;
        std::vector<Entity*> entities;
        std::vector<uint32_t> generations;
        std::vector<MeshComp> meshes;
        std::vector<PhysicsComp> physics;
        std::vector<std::function<void()>> draw_calls;
        std::vector<std::function<void()>> draw_calls_3d;
        Shader shader;
        ~Level();
        void serialize(Serializer * ser)const;
        static Level deserialize(Deserializer*des);
        static Level *interface_deserialize(Deserializer&des);
        bool should_save = false;
        bool should_load = false;
        std::string save_name;
        std::string load_name;
    };
    class Runtime{
        public:
        unique_ptr<Level> level;
    };
Register(Level, Level);
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
            if(index>=runtime.level->entities.size()){
                return false;
             }
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
            if(!is_valid()){
                return 0;
            }
            return runtime.level->entities[index];
         }
        template<typename T> T* downcast(){
            Entity * e= get(); 
            return dynamic_cast<T*>(e);
        }
    };

void mainloop(std::function<void()> func);
void setup();
Level & get_level();
void load_level(const char* path);
void load_level_fn(std::function<void()>func);
void save_level(const char* path);
template<typename T, typename...Args>EntityRef create_entity(Args...args){
        for(size_t i =0; i<runtime.level->entities.size(); i++){
            if(!runtime.level->entities[i]){
                runtime.level->entities[i] = new T(args...);
                runtime.level->generations[i]+=1;
                runtime.level->entities[i]->id = i;
                return EntityRef::create(i, runtime.level->generations[i]);
            }
        }
        runtime.level->entities.push_back(new T(args...));
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
void draw_call(std::function<void()>to_call);
void draw_call_3d(std::function<void()>to_call);

std::vector<EntityRef> get_all_entities_with_tag(Tag tag);
std::vector<EntityRef> get_all_entities_with_at_least_one_tag(Tag tags[], size_t count);
std::vector<EntityRef> get_all_entities_with_tag_set(Tag tags[], size_t count);
void apply_damage(EntityRef source, EntityRef target,Vec3 direction, double amount);
Model path_load_model(const std::string& mod, const std::vector<std::string>& textures, unordered_map<string, Texture> & loaded_textures, Shader shader);
}





