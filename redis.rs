extern mod std;

use std::*;
use io::{ReaderUtil,WriterUtil};

enum Result {
  Nil,
  Int(int),
  Data(~str),
  List(~[Result]),
  Error(~str),
  Status(~str)
}

priv fn parse_data(len: uint, sb: net::tcp::TcpSocketBuf) -> Result {
  let res =
    if (len > 0) {
      let bytes = sb.read_bytes(len as uint);
      assert bytes.len() == len;
      Data(str::from_bytes(bytes))
    } else {
      Data(~"")
    };
  assert sb.read_char() == '\r';
  assert sb.read_char() == '\n';
  return res;
}

fn parse_list(len: uint, sb: net::tcp::TcpSocketBuf) -> Result {
  let mut list: ~[Result] = ~[];
  for len.times {
    let line = sb.read_line();
    assert line.len() >= 1;
    let c = line.char_at(0);
    let rest = line.slice(1, line.len() - 1);
    let v = 
      match c {
	'$' =>
	  match int::from_str(rest) {
	    None => fail,
	    Some(-1) => Nil,
	    Some(len) if len >= 0 => parse_data(len as uint, sb),
	    Some(_) => fail
	  },
	_ => fail
      };
    list.push(v);
  }
  return List(list);
}

// XXX: use read_char, then read_line!
fn parse_response(sb: net::tcp::TcpSocketBuf) -> Result {
  let line = sb.read_line();
  assert line.len() >= 1;

  let c = line.char_at(0);
  let rest = line.slice(1, line.len() - 1);

  match c {
    '$' =>
      match int::from_str(rest) {
        None => fail, 
        Some(-1) => Nil,
        Some(len) if len >= 0 => parse_data(len as uint, sb),
        Some(_) => fail
      },
    '*' =>
      match int::from_str(rest) {
        None => fail,
        Some(-1) => Nil,
        Some(0) => List(~[]), 
        Some(len) if len >= 0 => parse_list(len as uint, sb),
        Some(_) => fail
      },
    '+' => Status(rest),
    '-' => Error(rest),
    ':' => match int::from_str(rest) {
             None => fail,
             Some(i) => Int(i)
           },
    _   => fail
  }
}

fn cmd_to_str(cmd: ~[~str]) -> ~str {
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
  io::println(cmd);
  sb.write_str(cmd);
  let res = parse_response(sb);
  io::println(fmt!("%?", res));
  res
}

fn main() {
  let server_ip_addr = net::ip::v4::parse_addr("127.0.0.1");
  let server_port = 6379;
  let iotask = uv::global_loop::get();
  let connect_result = net::tcp::connect(server_ip_addr, server_port, iotask); 
  let sock = result::unwrap(connect_result);
  let sb = net::tcp::socket_buf(sock);

  query(~[~"SET", ~"abc", ~"XXX"], sb);
  query(~[~"SET", ~"def", ~"123"], sb);
  query(~[~"GET", ~"abc"], sb);
  query(~[~"MGET", ~"abc", ~"def", ~"non"], sb);
}
