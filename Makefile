build:
	rustpkg build -O bench

install:
	rustpkg install bench

clean:
	rustpkg clean

explicit:
	rustc --lib -O --out-dir=. src/redis/lib.rs
	rustc -O -L . -o bench src/bench/main.rs 
