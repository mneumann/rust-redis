RUSTC=rustc

bench: redis.rs bench.rs Makefile
	${RUSTC} -O bench.rs
