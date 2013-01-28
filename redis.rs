extern mod std;

use core::to_bytes::*;

use std::*;
use io::{ReaderUtil,WriterUtil};


enum Result {
  Nil,
  Int(int),
  Data(~[u8]),
  List(~[Result]),
  Error(~str),
  Status(~str)
}

priv fn parse_data(len: uint, io: io::Reader) -> Result {
  let res =
    if (len > 0) {
      let bytes = io.read_bytes(len);
      assert bytes.len() == len;
      Data(bytes)
    } else {
      Data(~[])
    };
  assert io.read_byte() == 13;
  assert io.read_byte() == 10;
  return res;
}

priv fn read_char(io: io::Reader) -> char {
  let ch = io.read_byte();
  if ch < 0 { fail }
  ch as char
}

priv fn parse_list(len: uint, io: io::Reader) -> Result {
  List(vec::from_fn(len, |_| { parse_response(io) }))
}

priv fn parse_int_line(io: io::Reader) -> int {
  let mut i: int = 0;
  let mut digits: uint = 0;
  let mut negative: bool = false;

  loop {
    let ch = read_char(io);
    match ch {
      '0' .. '9' => {
        digits += 1;
        i = (i * 10) + (ch as int - '0' as int);
        },
      '-' => {
        if negative { fail }
        negative = true
        },
      '\r' => {
        assert read_char(io) == '\n';
        break
        },
      '\n' => break,
      _ => fail
    }
  }

  if digits == 0 { fail }

  if negative { -i }
  else { i }
}

priv fn parse_n(io: io::Reader, f: fn(uint, io::Reader) -> Result) -> Result {
  match parse_int_line(io) {
    -1 => Nil,
    len if len >= 0 => f(len as uint, io),
    _ => fail
  }
}

priv fn parse_response(io: io::Reader) -> Result {
  match read_char(io) {
    '$' => parse_n(io, parse_data),
    '*' => parse_n(io, parse_list),
    '+' => Status(io.read_line()),
    '-' => Error(io.read_line()),
    ':' => Int(parse_int_line(io)),
    _   => fail
  }
}

priv fn cmd_to_str(cmd: ~[~str]) -> ~str {
  let mut res = ~"*";
  str::push_str(&mut res, cmd.len().to_str());
  str::push_str(&mut res, "\r\n"); 
  for cmd.each |s| {
    str::push_str(&mut res, str::concat(~[~"$", s.len().to_str(), ~"\r\n", *s, ~"\r\n"]));
  }
  res
}

fn send_query(cmd: ~[~str], sb: net::tcp::TcpSocketBuf) {
  let cmd = cmd_to_str(cmd);
  sb.write_str(cmd);
}

fn recv_query(sb: net::tcp::TcpSocketBuf) -> Result {
  let res = parse_response(sb as io::Reader);
  //io::println(fmt!("%?", res));
  res
}

fn query(cmd: ~[~str], sb: net::tcp::TcpSocketBuf) -> Result {
  send_query(cmd, sb);
  recv_query(sb)
}

fn main() {
  let server_ip_addr = net::ip::v4::parse_addr("127.0.0.1");
  let server_port = 6379;
  let iotask = uv::global_loop::get();
  let connect_result = net::tcp::connect(server_ip_addr, server_port, iotask); 
  let sock = result::unwrap(connect_result);
  let sb = net::tcp::socket_buf(sock);

  //query(~[~"SET", ~"def", ~"abc"], sb);

  //let cmd = cmd_to_str(~[~"GET", ~"def"]);

  for 10000.times {
    //let x = cmd.to_bytes(true);
    //net::tcp::write(&sock, x);

    //do str::as_buf(cmd) {
      //net::tcp::write(&sock, vec::from_buf(ptr, elts))
    //}
    //str::as_bytes(&~"blah", |bytes| {
      //let blah: ~[u8] = *bytes;
      //net::tcp::write(&sock, *bytes)
    //});

/*
    for 10_000.times {
      send_query(~[~"GET", ~"def"], sb);
    }
    for 10_000.times {
      recv_query(sb);
    }
*/
    for 1.times {
      send_query(~[~"GET", ~"def"], sb);
    }
    for 1.times {
      recv_query(sb);
    }

    //query(~[~"EXEC"], sb);
  }



/*
  let mut buf: ~[u8] = vec::from_elem(10_000, 0);

  loop {
    let read_result = net::tcp::read(&sock, 0u);
    if !read_result.is_err() {
      let s = vec::len(result::unwrap(read_result));
      io::println(fmt!("%u\n", s));
    }
    else {
      io::println("ERROR");
      break;
    }
    //if sb.read(buf, 10_000) < 10_000 { break }
    //io::println("read");
  }
*/
/*  query(~[~"MULTI"], sb);
  query(~[~"GET", ~"def"], sb);
  query(~[~"INCR", ~"def"], sb);
  query(~[~"INCR", ~"def"], sb);
  query(~[~"GET", ~"def"], sb);
  query(~[~"EXEC"], sb);

  query(~[~"SET", ~"abc", ~"XXX"], sb);
  query(~[~"SET", ~"def", ~"123"], sb);
  query(~[~"GET", ~"abc"], sb);
  query(~[~"MGET", ~"abc", ~"def", ~"non"], sb);
*/
}
