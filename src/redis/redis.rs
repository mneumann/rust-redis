use std::io::buffered::BufferedStream;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::Stream;
use std::vec::bytes::push_bytes;
use std::vec;

pub enum Result {
  Nil,
  Int(int),
  Data(~[u8]),
  List(~[Result]),
  Error(~str),
  Status(~str),
  ProtocolError(&'static str)
}

fn read_char<T: Stream>(io: &mut BufferedStream<T>) -> Option<char> {
  match io.read_byte() {
    Some(ch) => Some(ch as char),
    None     => None
  }
}

fn parse_data<T: Stream>(len: uint, io: &mut BufferedStream<T>) -> Result {
  let res =
    if (len > 0) {
      let bytes = io.read_bytes(len);
      if bytes.len() != len {
        return ProtocolError("Invalid number of bytes")
      } else {
        Data(bytes)
      }
    } else {
      Data(~[])
    };

  if (read_char(io) != Some('\r')) {
    return ProtocolError("Carriage return expected"); // TODO: ignore
  }

  if (read_char(io) != Some('\n')) {
    return ProtocolError("Newline expected");
  }

  return res;
}

fn parse_list<T: Stream>(len: uint, io: &mut BufferedStream<T>) -> Result {
  let mut list: ~[Result] = vec::with_capacity(len);

  for _ in range(0, len) {
    match parse(io) {
      ProtocolError(err) => {
        return ProtocolError(err);
      }
      other => {
        list.push(other);
      }
    }
  }

  List(list)
}

fn parse_int_line<T: Stream>(io: &mut BufferedStream<T>) -> Option<int> {
  let mut i: int = 0;
  let mut digits: uint = 0;
  let mut negative: bool = false;

  loop {
    match read_char(io) {
      None => { return None }
      Some(ch) => {
        match ch {
          '0' .. '9' => {
            digits += 1;
            i = (i * 10) + (ch as int - '0' as int);
          }
          '-' => {
            if negative { return None }
            negative = true
          }
          '\r' => {
            if read_char(io) != Some('\n') { return None } 
            break
          }
          '\n' => { break }
          _ => { return None }
        }
      }
    }
  }

  if digits == 0 { return None }

  if negative { Some(-i) }
  else { Some(i) }
}

fn parse_n<T: Stream>(io: &mut BufferedStream<T>, f: |uint, &mut BufferedStream<T>| -> Result) -> Result {
  match parse_int_line(io) {
    Some(-1) => Nil,
    Some(len) if len >= 0 => f(len as uint, io),
    _ => ProtocolError("Invalid number")
  }
}

fn parse_status<T: Stream>(io: &mut BufferedStream<T>) -> Result {
  match io.read_line() {
    Some(line) => Status(line),
    None       => ProtocolError("Invalid status line") 
  }
}

fn parse_error<T: Stream>(io: &mut BufferedStream<T>) -> Result {
  match io.read_line() {
    Some(line) => Error(line),
    None       => ProtocolError("Invalid error line")
  }
}

pub fn parse<T: Stream>(io: &mut BufferedStream<T>) -> Result {
  match read_char(io) {
    Some('$') => parse_n(io, parse_data),
    Some('*') => parse_n(io, parse_list),
    Some('+') => parse_status(io),
    Some('-') => parse_error(io),
    Some(':') => {
      match parse_int_line(io) {
        Some(i) => Int(i),
        None => ProtocolError("Invalid number")
      }
    },
    Some(_)   => ProtocolError("Invalid character"),
    None      => ProtocolError("Invalid EOF")
  }
}

pub struct CommandWriter {
  buf: ~[u8]
}

impl CommandWriter {
  pub fn new() -> CommandWriter {
    CommandWriter { buf: ~[] }
  }

  pub fn args<'a>(&'a mut self, n: uint) -> &'a mut CommandWriter {
    self.write_char('*');
    self.write_uint(n);
    self.write_crnl();
    self
  }

  pub fn arg_bin<'a>(&'a mut self, arg: &[u8]) -> &'a mut CommandWriter {
    self.write_char('$');
    self.write_uint(arg.len());
    self.write_crnl();
    self.write(arg);
    self.write_crnl();
    self
  }

  pub fn nil(&mut self) {
    self.write_str("$-1");
    self.write_crnl();
  }

  pub fn arg_str<'a>(&'a mut self, arg: &str) -> &'a mut CommandWriter {
    self.write_char('$');
    self.write_uint(arg.len());
    self.write_crnl();
    self.write_str(arg);
    self.write_crnl();
    self
  }

  pub fn error(&mut self, err: &str) {
    self.write_char('-');
    self.write_str(err);
    self.write_crnl();
  }

  pub fn status(&mut self, status: &str) {
    self.write_char('+');
    self.write_str(status);
    self.write_crnl();
  }

  fn write_crnl(&mut self) {
    self.write_char('\r');
    self.write_char('\n');
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

  fn write(&mut self, s: &[u8]) {
    push_bytes(&mut self.buf, s);
  }
 
  fn write_char(&mut self, s: char) {
    self.buf.push(s as u8);
  }

  fn write_byte(&mut self, b: u8) {
    self.buf.push(b);
  }

  pub fn with_buf<T>(&self, f: |&[u8]| -> T) -> T {
    f(self.buf.as_slice())
  }
}

fn execute<T: Stream>(cmd: &[u8], io: &mut BufferedStream<T>) -> Result {
  io.write(cmd);
  io.flush();
  parse(io)
}

pub struct Client<T> {
  priv io: BufferedStream<T>
}

impl Client<TcpStream> {
  pub fn new(sock_addr: &str) -> Client<TcpStream> {
    let addr = from_str::<SocketAddr>(sock_addr).unwrap();
    let tcp_stream = TcpStream::connect(addr).unwrap();
    Client::new_from_stream(tcp_stream)
  }
}

impl<T: Stream> Client<T> {
  pub fn new_from_stream(io: T) -> Client<T> {
    Client { io: BufferedStream::new(io) }
  }

  pub fn get(&mut self, key: &str) -> Result {
    let mut cwr = CommandWriter::new();
    cwr.args(2).
        arg_str("GET").
        arg_str(key).
        with_buf(|cmd| execute(cmd, &mut self.io))
  }
  
  pub fn set(&mut self, key: &str, val: &str) -> Result {
    let mut cwr = CommandWriter::new();
    cwr.args(3).
        arg_str("SET").
        arg_str(key).
        arg_str(val).
        with_buf(|cmd| execute(cmd, &mut self.io))
  }
}
