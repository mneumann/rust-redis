compile:
	rustpkg build redis
	rustpkg build -O bench
	rustpkg build examples/simple

install:
	rustpkg install redis
	rustpkg install bench
	rustpkg install examples/simple

clean:
	rustpkg clean
