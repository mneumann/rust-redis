extern mod redis = "redis#0.1";
extern mod extra;
extern mod native;

use redis::Client;
use native::task;

fn bench_set(tid: uint, n: uint) {
  let mut redis = Client::new("127.0.0.1:6379");

  for _ in range(0, n) {
    redis.set("key", "12");
  }

  println!("Thread {} finished", tid);
}

fn main() {
  let before = extra::time::precise_time_ns();

  let concurrency: uint = from_str(std::os::args()[1]).unwrap();
  let repeats: uint = from_str(std::os::args()[2]).unwrap();
  let per_thread: uint = repeats / concurrency;
  let total_reqs = per_thread * concurrency;

  let mut threads = ~[];

  for tid in range(0, concurrency) {
    println!("Thread {} started", tid);
    
    let (port, chan) = Chan::new();
    do task::spawn {
      bench_set(tid, per_thread);
      chan.send(());
    }
    threads.push(port);
  }

  println!("Waiting for all clients to terminate");
  for port in threads.iter() {
      port.recv();
  }

  let after = extra::time::precise_time_ns();

  let time = ((after - before) / 1_000_000) as f64 / 1000f64;

  println!("Concurrency: {}", concurrency);
  println!("Total requests: {}", total_reqs);
  println!("Total time: {}", time);
  let reqs_per_s = total_reqs as f64 / time;
  println!("Requests per second: {}", reqs_per_s); 
}
