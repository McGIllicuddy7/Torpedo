make: cereal.cpp 
	gcc -c cereal.cpp -std=c++20 -Wall -Wextra -pedantic -g3 -fsanitize=address
test: main.cpp cereal.cpp test.cpp
	clang++ main.cpp cereal.cpp test.cpp -std=c++20 -Wall -Wextra -pedantic -fsanitize=address -g3
