extern mod extra;
extern mod green;
use redis::{Redis};
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::buffered::BufferedStream;

mod redis;

fn bench_set(n: uint) {
  let addr = from_str::<SocketAddr>("127.0.0.1:6379").unwrap();
  let tcp_stream = TcpStream::connect(addr).unwrap();
  let mut reader = BufferedStream::new(tcp_stream);
  let mut redis = Redis::new(&mut reader);

  for _ in range(0, n) {
    redis.set("key", "12");
  }
}

fn main() {
  let before = extra::time::precise_time_ns();

  let concurrency: uint = from_str(std::os::args()[1]).unwrap();
  let repeats: uint = from_str(std::os::args()[2]).unwrap();
  let total_reqs = concurrency * repeats;

  let mut pool = green::SchedPool::new(green::PoolConfig { threads: concurrency, event_loop_factory: None });

  for i in range(0, concurrency) {
    println!("Client {} started", i);
    do pool.spawn(std::task::TaskOpts::new()) {
      bench_set(repeats);
    }
  }
  println!("Waiting for all clients to terminate");
  pool.shutdown();

  let after = extra::time::precise_time_ns();

  let time = ((after - before) / 1_000_000) as f64 / 1000f64;

  println!("Concurrency: {}", concurrency);
  println!("Total requests: {}", total_reqs);
  println!("Total time: {}", time);
  let reqs_per_s = total_reqs as f64 / time;
  println!("Requests per second: {}", reqs_per_s); 
}
