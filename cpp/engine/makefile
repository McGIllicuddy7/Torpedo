 files= physics/col.cpp physics/physics.cpp level.cpp renderer/renderer.cpp particles.cpp
 flags = -std=c++20 -g3 -fsanitize=address
 libs =-lraylib -L /opt/homebrew/raylib/lib/ 
make: $(files)
	g++ -c $(files) $(flags) $(libs)
	ar rcs engine.a *.o
	rm -rf *.o
