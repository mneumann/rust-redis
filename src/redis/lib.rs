#[crate_id = "redis#0.1"];
#[desc = "A Rust client library for Redis"];
#[license = "MIT"];
#[crate_type = "lib"];

pub use redis::parse;
pub use redis::{Result,Client,Nil,Int,Data,List,Error,Status,ProtocolError};
pub use redis::CommandWriter;
pub mod redis;
