#include "ship.hpp"
namespace Torpedo{
NPCShip::NPCShip(){
	ship = ShipComp{};
}
void NPCShip::on_tick(){
	ship.parent = get_as_ref(this);
	ship.update();
}
Alignment NPCShip::get_alignment(){
	return align;
}
EntityRef create_npc_ship(Vec3 pos, Quat rot, Alignment align){
	EntityRef out = create_entity<NPCShip>();	
	NPCShip * ptr = out.downcast<NPCShip>();
	ptr->add_tag(tag_ship);
	ptr->add_tag(tag_movable);
	ptr->add_tag(tag_pressurized);
	ptr->align = align;
	out.get()->get_physics().trans.trans.translation = pos;
	out.get()->get_physics().trans.trans.rotation = rot;
	out.get()->get_physics().destroy_on_impact = false;
	out.get()->get_physics().is_valid = true;
	out.get()->get_physics().colliders = std::vector<Collider>{Collider{Trans::create(),  BoundingBox{Vec3{-1.91526,-0.309, -0.309}/2.0, Vec3{1.0067,0.309, 0.309}/2.0}}};
	MeshPart msh;
	msh.color = Color{0,0,0,0};
	msh.offset= Trans::create();
	//-0.257, 0, -0.0615
	msh.offset.translation = Vec3{-0.540*0.75,0,-0.130*0.75};
	msh.offset.scale = Vec3{1,1,1};
	msh.string = "ship";
	out.get()->get_mesh().meshes = std::unordered_map<std::string, MeshPart>{{"mesh", msh}};
	return out;
}
void NPCShip::on_damage(Vec3 incoming_direction, double damage){
	destroy_entity(get_as_ref(this));
}
}
