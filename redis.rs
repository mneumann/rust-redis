use std::io::buffered::BufferedStream;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::vec::bytes::push_bytes;

pub enum Result {
  Nil,
  Int(int),
  Data(~[u8]),
  List(~[Result]),
  Error(~str),
  Status(~str)
}

fn read_char(io: &mut BufferedStream<TcpStream>) -> char {
  match io.read_byte() {
    Some(ch) => ch as char,
    None     => fail!()
  }
}

fn parse_data(len: uint, io: &mut BufferedStream<TcpStream>) -> Result {
  let res =
    if (len > 0) {
      let bytes = io.read_bytes(len);
      assert!(bytes.len() == len);
      Data(bytes)
    } else {
      Data(~[])
    };
  assert!(io.read_byte() == Some(13));
  assert!(io.read_byte() == Some(10));
  return res;
}

fn parse_list(len: uint, io: &mut BufferedStream<TcpStream>) -> Result {
  List(std::vec::from_fn(len, |_| { parse_response(io) }))
}

fn parse_int_line(io: &mut BufferedStream<TcpStream>) -> int {
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
        if negative { fail!() }
        negative = true
        },
      '\r' => {
        assert!(read_char(io) == '\n');
        break
        },
      '\n' => break,
      _ => fail!()
    }
  }

  if digits == 0 { fail!() }

  if negative { -i }
  else { i }
}

fn parse_n(io: &mut BufferedStream<TcpStream>, f: |uint, &mut BufferedStream<TcpStream>| -> Result) -> Result {
  match parse_int_line(io) {
    -1 => Nil,
    len if len >= 0 => f(len as uint, io),
    _ => fail!()
  }
}

fn parse_status(io: &mut BufferedStream<TcpStream>) -> Result {
  match io.read_line() {
    Some(line) => Status(line),
    None       => fail!()
  }
}

fn parse_error(io: &mut BufferedStream<TcpStream>) -> Result {
  match io.read_line() {
    Some(line) => Error(line),
    None       => fail!()
  }
}

fn parse_response(io: &mut BufferedStream<TcpStream>) -> Result {
  match read_char(io) {
    '$' => parse_n(io, parse_data),
    '*' => parse_n(io, parse_list),
    '+' => parse_status(io),
    '-' => parse_error(io),
    ':' => Int(parse_int_line(io)),
    _   => fail!()
  }
}

fn prepare_cmd(cmd: ~[~str]) -> ~[u8] {
  let mut res: ~[u8] = ~[];
  push_bytes(&mut res, bytes!("*"));
  push_bytes(&mut res, cmd.len().to_str().into_bytes());
  push_bytes(&mut res, bytes!("\r\n"));
  for c in cmd.iter() {
    push_bytes(&mut res, bytes!("$"));
    push_bytes(&mut res, c.len().to_str().into_bytes());
    push_bytes(&mut res, bytes!("\r\n"));
    push_bytes(&mut res, c.as_bytes());
    push_bytes(&mut res, bytes!("\r\n"));
  }
  res
}

fn send_query(cmd: &[u8], io: &mut BufferedStream<TcpStream>) {
  io.write(cmd);
  io.flush();
}

fn recv_query(io: &mut BufferedStream<TcpStream>) -> Result {
  parse_response(io)
}

fn query(cmd: &[u8], io: &mut BufferedStream<TcpStream>) -> Result {
  send_query(cmd, io);
  let res = recv_query(io);
  res
}

fn main() {
  let addr = from_str::<SocketAddr>("127.0.0.1:6379").unwrap();
  let tcp_stream = TcpStream::connect(addr).unwrap();
  let mut reader = BufferedStream::new(tcp_stream);

  let cmd = prepare_cmd(~[~"SET", ~"def", ~"abc"]);
 
  for i in std::iter::range(1, 100_000) {
    query(cmd, &mut reader);
  }
}
