extern mod redis = "redis#0.1";
extern mod extra;
extern mod green;
use redis::Client;

fn bench_set(n: uint) {
  let mut redis = Client::new("127.0.0.1:6379");

  for _ in range(0, n) {
    redis.set("key", "12");
  }
}

fn main() {
  let before = extra::time::precise_time_ns();

  let concurrency: uint = from_str(std::os::args()[1]).unwrap();
  let repeats: uint = from_str(std::os::args()[2]).unwrap();
  let per_thread: uint = repeats / concurrency;
  let total_reqs = per_thread * concurrency;

  let mut pool = green::SchedPool::new(green::PoolConfig { threads: concurrency, event_loop_factory: None });

  for i in range(0, concurrency) {
    println!("Client {} started", i);
    do pool.spawn(std::task::TaskOpts::new()) {
      bench_set(per_thread);
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
