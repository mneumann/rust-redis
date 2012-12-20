RUSTC=rustc

redis: redis.rs Makefile
	${RUSTC} -O redis.rs
