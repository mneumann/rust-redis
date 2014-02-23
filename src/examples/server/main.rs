/*
 * An example redis server accepting GET and SET requests
 * listening on 127.0.0.1:8000.
 *
 * You can test it with "redis-cli -p 8000".
 *
 * Copyright (c) 2014 by Michael Neumann
 *
 */

extern crate redis = "redis#0.1";
extern crate sync = "sync#0.10-pre";

use std::io::net::ip::SocketAddr;
use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Listener,Acceptor,Writer};
use std::io::BufferedStream;
use std::task;
use std::hashmap::HashMap;
use sync::RWArc;

fn handle_connection(conn: TcpStream, shared_ht: RWArc<HashMap<~[u8],~[u8]>>) {
    debug!("Got connection");

    let mut io = BufferedStream::new(conn);

    loop {
        match redis::parse(&mut io).unwrap() {
            redis::List(ref lst) => {
                match lst.get(0) {
                    Some(&redis::Data(ref command)) => {
                        if command.as_slice() == bytes!("GET") {
                            match (lst.len(), lst.get(1)) {
                                (2, Some(&redis::Data(ref key))) => {
                                    debug!("GET: {:s}", std::str::from_utf8(key.as_slice()).unwrap());
                                    let mut cwr = redis::CommandWriter::new();
                                    shared_ht.read(|ht| {
                                        match ht.find(key) {
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
                                    continue;
                                }
                                _ => { /* fallthrough: error */ }
                            }
                        }
                        else if command.as_slice() == bytes!("SET") {
                            match (lst.len(), lst.get(1), lst.get(2)) {
                                (3, Some(&redis::Data(ref key)), Some(&redis::Data(ref val))) => {
                                    debug!("SET: {:s} {:?}", std::str::from_utf8(key.as_slice()).unwrap(), val);
                                    shared_ht.write(|ht| ht.insert(key.clone(), val.clone()));
                                    let mut cwr = redis::CommandWriter::new();
                                    cwr.status("OK");
                                    cwr.with_buf(|bytes| {io.write(bytes); io.flush()});
                                    continue;
                                }
                                _ => { /* fallthrough: error */ }
                            }
                        }
                        else {
                            /* fallthrough: error */
                        }
                    }
                    _ => { /* fallthrough: error */ }
                }
            }
            _ => { /* fallthrough: error */ }
        }

        /* error */
        let mut cwr = redis::CommandWriter::new();
        cwr.error("Invalid Command");
        cwr.with_buf(|bytes| {io.write(bytes); io.flush()});
    }
}

fn main() {
    let addr: SocketAddr = from_str("127.0.0.1:8000").unwrap();
    let shared_ht = RWArc::new(HashMap::new());

    match TcpListener::bind(addr) {
        Ok(listener) => {
            match listener.listen() {
                Ok(ref mut acceptor) => {
                    loop {
                        match acceptor.accept() {
                            Ok(conn) => {
                                let ht = shared_ht.clone();
                                task::spawn(proc() {
                                    handle_connection(conn, ht)
                                });
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}
