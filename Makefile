build:
	rustpkg build redis
	rustpkg build -O bench
	rustpkg build examples/simple

install:
	rustpkg install redis
	rustpkg install bench
	rustpkg install examples/simple

clean:
	rustpkg clean

explicit:
	rustc --lib -O --out-dir=. src/redis/lib.rs
	rustc -O -L . -o bench src/bench/main.rs 
	rustc -O -L . -o simple src/examples/simple/main.rs 
