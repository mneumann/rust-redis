#![crate_id = "redis#0.1"]
#![desc = "A Rust client library for Redis"]
#![license = "MIT"]
#![crate_type = "lib"]

use std::io::{IoResult,IoError,InvalidInput};
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::{BufferedStream, Stream};
use std::vec::bytes::push_bytes;
use std::vec;
use std::str::from_utf8;

pub enum Result {
  Nil,
  Int(i64),
  Data(~[u8]),
  List(~[Result]),
  Error(~str),
  Status(~str)
}

fn invalid_input(desc: &'static str) -> IoError {
  IoError {kind: InvalidInput, desc: desc, detail: None}
}

fn read_char<T: Stream>(io: &mut BufferedStream<T>) -> IoResult<char> {
  Ok(try!(io.read_byte()) as char)
}

fn parse_data<T: Stream>(len: uint, io: &mut BufferedStream<T>) -> IoResult<Result> {
  let res =
    if len > 0 {
      let bytes = try!(io.read_bytes(len));
      if bytes.len() != len {
        return Err(invalid_input("Invalid number of bytes"))
      } else {
        Data(bytes)
      }
    } else {
      // XXX: needs this to be special case or is read_bytes() working with len=0?
      Data(~[])
    };

  if try!(read_char(io)) != '\r' {
    return Err(invalid_input("Carriage return expected")); // TODO: ignore
  }

  if try!(read_char(io)) != '\n' {
    return Err(invalid_input("Newline expected"));
  }

  return Ok(res);
}

fn parse_list<T: Stream>(len: uint, io: &mut BufferedStream<T>) -> IoResult<Result> {
  let mut list: ~[Result] = vec::with_capacity(len);

  for _ in range(0, len) {
    list.push(try!(parse(io)))
  }

  Ok(List(list))
}

fn parse_int_line<T: Stream>(io: &mut BufferedStream<T>) -> IoResult<i64> {
  let mut i: i64 = 0;
  let mut digits: uint = 0;
  let mut negative: bool = false;

  loop {
    match try!(read_char(io)) {
      ch @ '0' .. '9' => {
        digits += 1;
        i = (i * 10) + (ch as i64 - '0' as i64);
      }
      '-' => {
        if negative { return Err(invalid_input("Invalid negative number")) }
        negative = true
      }
      '\r' => {
        if try!(read_char(io)) != '\n' {
          return Err(invalid_input("Newline expected"))
        }
        break
      }
      '\n' => { break }
      _ => { return Err(invalid_input("Invalid character")) }
    }
  }

  if digits == 0 { return Err(invalid_input("No number given")) }

  if negative { Ok(-i) }
  else { Ok(i) }
}

fn parse_n<T: Stream>(io: &mut BufferedStream<T>, f: |uint, &mut BufferedStream<T>| -> IoResult<Result>) -> IoResult<Result> {
  match try!(parse_int_line(io)) {
    -1 => Ok(Nil),
    len if len >= 0 => f(len as uint, io), // XXX: i64 might be larger than uint
    _ => Err(invalid_input("Invalid number"))
  }
}

fn parse_status<T: Stream>(io: &mut BufferedStream<T>) -> IoResult<Result> {
  Ok(Status(try!(io.read_line())))
}

fn parse_error<T: Stream>(io: &mut BufferedStream<T>) -> IoResult<Result> {
  Ok(Error(try!(io.read_line())))
}

pub fn parse<T: Stream>(io: &mut BufferedStream<T>) -> IoResult<Result> {
  match try!(read_char(io)) {
    '$' => parse_n(io, parse_data),
    '*' => parse_n(io, parse_list),
    '+' => parse_status(io),
    '-' => parse_error(io),
    ':' => Ok(Int(try!(parse_int_line(io)))),
    _   => Err(invalid_input("Invalid character")) 
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
      push_bytes(&mut self.buf, n.to_str().into_bytes()); // XXX: Optimize
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

fn execute<T: Stream>(cmd: &[u8], io: &mut BufferedStream<T>) -> IoResult<Result> {
  try!(io.write(cmd));
  try!(io.flush());
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

  pub fn get(&mut self, key: &str) -> IoResult<Result> {
    let mut cwr = CommandWriter::new();
    cwr.args(2).
        arg_str("GET").
        arg_str(key).
        with_buf(|cmd| execute(cmd, &mut self.io))
  }

  pub fn get_str(&mut self, key: &str) -> IoResult<Option<~str>> {
    match try!(self.get(key)) {
      Nil => Ok(None),
      Int(i) => Ok(Some(i.to_str())),
      Data(ref bytes) => Ok(Some(from_utf8(*bytes).unwrap().to_owned())),
      _ => fail!("Invalid result type from Redis") 
    }
  }

  pub fn get_int(&mut self, key: &str) -> IoResult<Option<i64>> {
    match try!(self.get(key)) {
      Nil => Ok(None),
      Data(ref bytes) => Ok(from_str(from_utf8(*bytes).unwrap())), // XXX
      _ => fail!("Invalid result type from Redis") 
    }
  }
  
  pub fn set(&mut self, key: &str, val: &str) -> IoResult<Result> {
    let mut cwr = CommandWriter::new();
    cwr.args(3).
        arg_str("SET").
        arg_str(key).
        arg_str(val).
        with_buf(|cmd| execute(cmd, &mut self.io))
  }

  pub fn set_int(&mut self, key: &str, val: i64) -> IoResult<Result> {
    self.set(key, val.to_str())
  }

  pub fn incr(&mut self, key: &str) -> IoResult<i64> {
    let mut cwr = CommandWriter::new();
    let res = try!(cwr.args(2).
        arg_str("INCR").
        arg_str(key).
        with_buf(|cmd| execute(cmd, &mut self.io)));
    match res {
      Int(i) => Ok(i),
      _ => fail!()
    }
  }
}
