#include "ship.hpp"
namespace Torpedo{
void PlayerShip::on_tick(){
	ship.parent = get_as_ref(this);
	Vector2 imp = GetMouseDelta();
	ship.rotation_input = QuaternionFromEuler(0,imp.y*0.001, -imp.x*0.001);
	ship.movement_input = Vec3{
		get_input_axis(KEY_S, KEY_W),
		get_input_axis(KEY_D, KEY_A),
		get_input_axis(KEY_LEFT_SHIFT, KEY_Z),
	}*0.01;
	ship.update();
	auto rot  = get_physics().trans.trans.rotation;	
}
PlayerShip::PlayerShip(){
	ship = ShipComp{};
}
EntityRef create_player_ship(Vec3 pos, Quat rot){
	EntityRef out = create_entity<PlayerShip>();
	set_player_entity(out);
	out.get()->get_physics().trans.trans.translation = pos;
	out.get()->get_physics().trans.trans.rotation = rot;
	out.get()->get_physics().is_valid = true;
	out.get()->get_physics().colliders = std::vector<Collider>{Collider{Trans::create(), BoundingBox{Vector3{-1, -1, -1,}, Vector3{1,1,1}}}};
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
}
