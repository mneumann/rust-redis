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

struct CommandWriter {
  buf: ~[u8]
}

impl CommandWriter {
  fn new() -> CommandWriter {
    CommandWriter { buf: ~[] }
  }

  fn args(&mut self, n: uint) {
    self.write_char('*');
    self.write_uint(n);
    self.write_crnl();
  }

  fn arg(&mut self, arg: &str) {
    self.write_char('$');
    self.write_uint(arg.len());
    self.write_crnl();
    self.write_str(arg);
    self.write_crnl();
  }

  fn write_crnl(&mut self) {
    self.write_byte(13);
    self.write_byte(10);
  }

  fn write_uint(&mut self, n: uint) {
    if n < 10 {
      self.write_byte('0' as u8 + (n as u8));
    }
    else {
      push_bytes(&mut self.buf, n.to_str().into_bytes());
    }
  }

  fn write_str(&mut self, s: &str) {
    push_bytes(&mut self.buf, s.as_bytes());
  }

  fn write_char(&mut self, s: char) {
    self.buf.push(s as u8);
  }

  fn write_byte(&mut self, b: u8) {
    self.buf.push(b);
  }

  fn with_buf<T>(&self, f: |&[u8]| -> T) -> T {
    f(self.buf.as_slice())
  }
}

fn execute(cmd: &[u8], io: &mut BufferedStream<TcpStream>) -> Result {
  io.write(cmd);
  io.flush();
  parse_response(io)
}

pub struct Redis<'a> {
  priv io: &'a mut BufferedStream<TcpStream>
}

impl<'a> Redis<'a> {
  pub fn get(&mut self, key: &str) -> Result {
    let mut cwr = CommandWriter::new();
    cwr.args(2);
    cwr.arg("GET");
    cwr.arg(key);
    cwr.with_buf(|cmd| execute(cmd, self.io))
  }
  
  pub fn set(&mut self, key: &str, val: &str) -> Result {
    let mut cwr = CommandWriter::new();
    cwr.args(3);
    cwr.arg("SET");
    cwr.arg(key);
    cwr.arg(val);
    cwr.with_buf(|cmd| execute(cmd, self.io))
  }
}

fn main() {
  let addr = from_str::<SocketAddr>("127.0.0.1:6379").unwrap();
  let tcp_stream = TcpStream::connect(addr).unwrap();
  let mut reader = BufferedStream::new(tcp_stream);
  let mut redis = Redis { io: &mut reader };

  //let x = redis.get("key");
  //println!("{:?}", x);

  for i in std::iter::range(1, 100_000) {
    redis.set("key", "abc");
  }
}
