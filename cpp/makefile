files = main.cpp physics/col.cpp physics/physics.cpp level.cpp renderer/renderer.cpp ship/ship.cpp ship/player_ship.cpp ship/weapons.cpp ship/npc_ship.cpp particles.cpp ../cereal/cereal.o
flags = -std=c++20 -g3 -fsanitize=address
libs = -lraylib -L /opt/homebrew/opt/raylib/lib/ 
make: $(files)
	g++ $(files) $(flags) $(libs)
