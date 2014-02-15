compile:
	mkdir -p lib bin
	rustc --out-dir lib src/redis/lib.rs
	rustc -L lib -o bin/simple src/examples/simple/main.rs
	rustc -L lib -o bin/server src/examples/server/main.rs
	rustc -O -L lib -o bin/bench src/bench/main.rs
clean:
	rm -rf lib bin
