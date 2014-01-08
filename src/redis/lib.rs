#[crate_id = "redis#0.1"];
#[desc = "A Rust client library for Redis"];
#[license = "MIT"];
#[crate_type = "lib"];

pub use redis::{Result,Client,Nil,Int,Data,List,Error,Status,ProtocolError};
pub mod redis;
