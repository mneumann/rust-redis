extern mod std;

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
      let bytes = io.read_bytes(len as uint);
      assert bytes.len() == len;
      Data(bytes)
    } else {
      Data(~[])
    };
  assert io.read_char() == '\r';
  assert io.read_char() == '\n';
  return res;
}

priv fn parse_list(len: uint, io: io::Reader) -> Result {
  let mut list: ~[Result] = ~[];
  for len.times {
    let v =
      match io.read_char() {
        '$' => parse_bulk(io),
        ':' => parse_int(io),
         _  => fail
      };
    list.push(v);
  }
  return List(list);
}

priv fn chop(s: ~str) -> ~str {
  s.slice(0, s.len() - 1)
}
  
priv fn parse_bulk(io: io::Reader) -> Result {
  match int::from_str(chop(io.read_line())) {
    None => fail,
    Some(-1) => Nil,
    Some(len) if len >= 0 => parse_data(len as uint, io),
    Some(_) => fail
  }
}

priv fn parse_multi(io: io::Reader) -> Result {
  match int::from_str(chop(io.read_line())) {
    None => fail,
    Some(-1) => Nil,
    Some(0) => List(~[]),
    Some(len) if len >= 0 => parse_list(len as uint, io),
    Some(_) => fail
  }
}

priv fn parse_int(io: io::Reader) -> Result {
  match int::from_str(chop(io.read_line())) {
    None => fail,
    Some(i) => Int(i)
  }
}

priv fn parse_response(io: io::Reader) -> Result {
  match io.read_char() {
    '$' => parse_bulk(io),
    '*' => parse_multi(io),
    '+' => Status(chop(io.read_line())),
    '-' => Error(chop(io.read_line())),
    ':' => parse_int(io),
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

fn query(cmd: ~[~str], sb: net::tcp::TcpSocketBuf) -> Result {
  let cmd = cmd_to_str(cmd);
  //io::println(cmd);
  sb.write_str(cmd);
  let res = parse_response(sb as io::Reader);
  //io::println(fmt!("%?", res));
  res
}

fn query2(cmd: ~[~str]) -> Result {
  let cmd = cmd_to_str(cmd);
  do io::with_str_reader(~"$3\r\nXXX\r\n") |sb| {
    let res = parse_response(sb as io::Reader);
    io::println(fmt!("%?", res));
    res
  }
}


fn main() {
  let server_ip_addr = net::ip::v4::parse_addr("127.0.0.1");
  let server_port = 6379;
  let iotask = uv::global_loop::get();
  let connect_result = net::tcp::connect(server_ip_addr, server_port, iotask); 
  let sock = result::unwrap(connect_result);
  let sb = net::tcp::socket_buf(sock);

  query(~[~"SET", ~"abc", ~"XXX"], sb);

  for 10_000.times {
    query(~[~"GET", ~"abc"], sb);
  }

  /*query(~[~"MULTI"], sb);
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
