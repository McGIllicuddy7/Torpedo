files =main.cpp ship/ship.cpp ship/player_ship.cpp ship/weapons.cpp ship/npc_ship.cpp ../../cereal/cereal.o ../engine/engine.a
flags = -std=c++20 -g3  -fsanitize=address
libs = -lraylib -L /opt/homebrew/opt/raylib/lib/ 
make: $(files)
	g++  $(files) $(flags) 	$(libs)
	rm -rf *.o

