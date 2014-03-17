extern crate redis = "redis#0.1";
extern crate time;
extern crate native;

use redis::Client;
use native::task;
use std::comm::channel;

fn bench_set(tid: uint, n: uint) {
  let mut redis = Client::new("127.0.0.1:6379");

  for _ in range(0, n) {
    redis.set("key", "12");
  }

  println!("Thread {} finished", tid);
}

fn main() {
  let before = time::precise_time_ns();

  let concurrency: uint = from_str(std::os::args()[1]).unwrap();
  let repeats: uint = from_str(std::os::args()[2]).unwrap();
  let per_thread: uint = repeats / concurrency;
  let total_reqs = per_thread * concurrency;

  let mut threads = ~[];

  for tid in range(0, concurrency) {
    println!("Thread {} started", tid);
    
    let (sender, receiver) = channel();
    task::spawn(proc() {
      bench_set(tid, per_thread);
      sender.send(());
    });
    threads.push(receiver);
  }

  println!("Waiting for all clients to terminate");
  for receiver in threads.iter() {
      receiver.recv();
  }

  let after = time::precise_time_ns();

  let time = ((after - before) / 1_000_000) as f64 / 1000f64;

  println!("Concurrency: {}", concurrency);
  println!("Total requests: {}", total_reqs);
  println!("Total time: {}", time);
  let reqs_per_s = total_reqs as f64 / time;
  println!("Requests per second: {}", reqs_per_s); 
}
