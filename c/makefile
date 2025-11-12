 files= main.c physics.c level.c col.c renderer.c ship.c  ai.c
 flags = -std=c99 -lm -O3 -g3
libs =-lraylib -L /opt/homebrew/raylib/lib/ -I /opt/homebrew/include/ -L /opt/homebrew/lib/

make: $(files)
	gcc  $(files) $(flags) $(libs)
	
profile: $(files)
	gcc  $(files) $(flags) $(libs) -D PROFILE -l profiler
	./a.out
	pprof -png output_inside.prof
