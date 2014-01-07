extern mod redis = "redis#0.1";

fn main() {
  let mut redis = redis::Client::new("127.0.0.1:6379");
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
