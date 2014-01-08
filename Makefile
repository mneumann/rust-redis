compile:
	rustpkg build redis
	rustpkg build -O bench
	rustpkg build examples/simple
	rustpkg build examples/server

install:
	rustpkg install redis
	rustpkg install bench
	rustpkg install examples/simple
	rustpkg install examples/server

clean:
	rustpkg clean
