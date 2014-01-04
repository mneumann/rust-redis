# rust-redis [![Build Status][travis-image]][travis-link]

[travis-image]: https://travis-ci.org/mneumann/rust-redis.png?branch=master
[travis-link]: https://travis-ci.org/mneumann/rust-redis

A [Rust][rust-home] client library for [Redis][redis-home].

[rust-home]: http://www.rust-lang.org
[redis-home]: http://redis.io

## Quickstart

```rust
extern mod redis = "redis#0.1";

fn main() {
  let mut redis = redis::Redis::new("127.0.0.1:6379");
  redis.set("key", "123");

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

## Performance

I did some early performance benchmarks on rust-0.9-pre (as of 04-01-2013)
against redis-benchmark.

    redis-benchmark -n 100000 -c 1 -t set # ~19000 reqs/sec
    redis-benchmark -n 100000 -c 2 -t set # ~27000 reqs/sec
    redis-benchmark -n 100000 -c 4 -t set # ~36000 reqs/sec
    redis-benchmark -n 100000 -c 8 -t set # ~47000 reqs/sec

    ./bin/bench 1 100000 # ~24000 reqs/sec
    ./bin/bench 2  50000 # ~24000 reqs/sec
    ... # ~24000 reqs/sec

At this simple benchmark, rust-redis consistently shows about 24000 requests
per second regardless of concurrency. I think this is because the way
scheduling works. Using native threads would probably lead to the same
performance as redis-benchmark.

## License

rust-redis is under the MIT license, see LICENSE-MIT for details.
