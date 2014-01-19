/*
 * An example redis server accepting GET and SET requests
 * listening on 127.0.0.1:8000.
 *
 * You can test it with "redis-cli -p 8000".
 *
 * Copyright (c) 2014 by Michael Neumann
 *
 */

extern mod redis = "redis#0.1";
extern mod extra;

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Listener,Acceptor,Writer};
use std::io::BufferedStream;
use std::task;
use std::hashmap::HashMap;
use extra::arc::RWArc;

fn handle_connection(conn: TcpStream, shared_ht: RWArc<HashMap<~[u8],~[u8]>>) {
    debug!("Got connection");

    let mut io = BufferedStream::new(conn);

    loop {
        match redis::parse(&mut io) {
            redis::List([redis::Data(/*GET*/[71, 69, 84]), redis::Data(key)]) => {
                debug!("GET: {:s}", std::str::from_utf8(key));
                let mut cwr = redis::CommandWriter::new();
                shared_ht.read(|ht| {
                    match ht.find(&key) {
                        Some(val) => {
                            cwr.args(1);
                            cwr.arg_bin(*val);
                        }
                        None => {
                            cwr.nil();
                        }
                    }
                });
                cwr.with_buf(|bytes| {io.write(bytes); io.flush()});
            }
            redis::List([redis::Data(/*SET*/[83, 69, 84]), redis::Data(key),
                         redis::Data(val)]) => {
                debug!("SET: {:s} {:?}", std::str::from_utf8(key), val);

                shared_ht.write(|ht| ht.insert(key.clone(), val.clone()));
                let mut cwr = redis::CommandWriter::new();
                cwr.status("OK");
                cwr.with_buf(|bytes| {io.write(bytes); io.flush()});
            }
            _ => {
                let mut cwr = redis::CommandWriter::new();
                cwr.error("Invalid Command");
                cwr.with_buf(|bytes| {io.write(bytes); io.flush()});
            }
        }
    }
}

fn main() {
    let addr: SocketAddr = from_str("127.0.0.1:8000").unwrap();
    let shared_ht = RWArc::new(HashMap::new());

    match TcpListener::bind(addr) {
        Some(listener) => {
            match listener.listen() {
                Some(ref mut acceptor) => {
                    loop {
                        match acceptor.accept() {
                            Some(conn) => {
                                let ht = shared_ht.clone();
                                do task::spawn {
                                    handle_connection(conn, ht)
                                }
                            }
                            None => {}
                        }
                    }
                }
                None => {}
            }
        }
        None => {}
    }
}
