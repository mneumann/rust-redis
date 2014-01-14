# rust-redis [![Build Status][travis-image]][travis-link]

[travis-image]: https://travis-ci.org/mneumann/rust-redis.png?branch=master
[travis-link]: https://travis-ci.org/mneumann/rust-redis

A [Rust][rust-home] client/server library for [Redis][redis-home].

[rust-home]: http://www.rust-lang.org
[redis-home]: http://redis.io

## Quickstart

```rust
extern mod redis = "redis#0.1";

fn main() {
  let mut redis = redis::Client::new("127.0.0.1:6379");

  redis.set_int("counter", 1);
  redis.set("key", "Hello");

  redis.incr("counter");

  let counter = redis.get_int("counter").unwrap();
  println!("counter = {}", counter);

  let key = redis.get_str("key").unwrap();
  println!("key = {}", key);

  match redis.get("key") {
    redis::Nil => {
      println!("Key not found")
    }
    redis::Data(ref s) => {
      println!("{:?}", std::str::from_utf8(*s))
    }
    _ => { fail!() }
  }
}
```

For a simple redis server implementation which supports GET/SET commands see
examples/server/main.rs.

## Performance

I did some early performance benchmarks with rust-0.9 and compared it against
redis-benchmark.

    redis-benchmark -n 100000 -c 1 -t set # ~18500 reqs/sec
    redis-benchmark -n 100000 -c 2 -t set # ~27000 reqs/sec
    redis-benchmark -n 100000 -c 4 -t set # ~36000 reqs/sec
    redis-benchmark -n 100000 -c 8 -t set # ~44000 reqs/sec

    ./bin/bench 1 100000 # ~19500 reqs/sec
    ./bin/bench 2 100000 # ~19100 reqs/sec
    ./bin/bench 4 100000 # ~18800 reqs/sec
    ./bin/bench 8 100000 # ~18000 reqs/sec

At this simple benchmark, rust-redis consistently shows about 18000 requests
per second regardless of concurrency. I think this is because the way
scheduling works. Using native threads would probably lead to the same
performance as redis-benchmark.

## License

rust-redis is under the MIT license, see LICENSE-MIT for details.
