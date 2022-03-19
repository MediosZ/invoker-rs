all: run

run:
	gcc -o ./target/load ./src/load.c -ldl
	./target/load
