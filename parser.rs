
fn take_n<'r>(v: &'r [u8], n: uint) -> (&'r [u8], &'r [u8]) {
  (vec::slice(v, 0, n), vec::slice(v, n, v.len()))
}

fn take_atmost_n<'r>(v: &'r [u8], n: uint) -> (&'r [u8], &'r [u8]) {
  take_n(v, uint::min(n, v.len()))
}

fn take_head<'r>(v: &'r [u8]) -> (u8, &'r [u8]) {
  assert!(!v.is_empty());
  (*v.head(), vec::slice(v, 1, v.len()))
}


// in_integer
enum RedisType {
  TyNone,
  TyList,
  TyData,
  TyInt
}

enum Inside {
  in_nothing,
  in_number,
  in_number_digits_only,
  in_number_done,
  in_number_need_newline,
  in_data,
  //in_data_need_newline
}

struct RedisState {
  in: Inside,
  typ: RedisType,
  data_size: uint,
  data_have: uint,
  number: uint,
  negative_number: bool,
}

enum RedisResult {
  Finished,
  NeedMore,
  Error
}

enum Result {
  Invalid,
  Nil,
  Int(int),
  Data(~[u8]),
  List(~[Result]),
  Err(~str),
  Status(~str)
}

// Return
trait Visitor {
  fn on_list(&mut self, len: uint) -> bool;

  fn on_data_beg(&mut self, len: uint);
  fn on_data(&mut self, data: &[u8]);
  fn on_data_end(&mut self) -> bool;

  fn on_integer(&mut self, num: uint, sign: bool) -> bool;

  fn on_nil(&mut self) -> bool;
}

// keep recursion in visitor?
struct MyVisitor {
  stack: ~[~[Result]], // (len, idx)
  result: Result
}

// returns true if we are finished parsing a redis request
fn element_done(visitor: &mut MyVisitor, value: Result) -> bool {
  if visitor.stack.is_empty() {
    visitor.result = value;
    true
  }
  else {
    
    vec::push(&mut (visitor.stack[0/*visitor.stack.len()-1*/]), value);
    false
/*      let (len, idx) = visitor.stack.last();
    assert!(idx+1 <= len);
    visitor.stack[visitor.stack.len()-1] = (len, idx+1);
    if idx + 1 == len {
      error!("Finsihed list");
      let _ = vec::pop(&mut visitor.stack); 
      element_done(visitor)
    }
    else {
      false
    }
*/
  }
}

impl Visitor for MyVisitor {
  fn on_list(&mut self, len: uint) -> bool {
    error!("on_list(len: %u)", len);
    if len == 0 {
      element_done(self, List(~[]))
    }
    else {
      //vec::push(&mut self.stack, (len, 0));
      false
    }
  }

  fn on_data_beg(&mut self, len: uint) {
    error!("on_data_beg(len: %u)", len)
  }

  fn on_data(&mut self, data: &[u8]) {
    error!("on_data(%?)", data)
  }

  fn on_data_end(&mut self) -> bool {
    error!("on_data_end()");
    element_done(self, Data(~[1,2,3]))
  }

  fn on_integer(&mut self, num: uint, sign: bool) -> bool {
    error!("on_integer(%?, %?)", num, sign);
    element_done(self, Int(num as int)) // XXX
  }

  fn on_nil(&mut self) -> bool {
    error!("on_nil");
    element_done(self, Nil)
  }
}

fn parse_redis<'r, V: Visitor>(st: &mut RedisState, buf: &'r [u8], visitor: &mut V) -> (RedisResult, &'r [u8]) {
  let mut buf = buf;

  loop {
    match st.in {
      in_nothing => {
        if (buf.is_empty()) {break}

        let (c, b) = take_head(buf);
        buf = b;

        match c as char {
          '\r' | '\n' | ' ' =>  {
              error!("ignoring carriage return/new/whitespace line")
            }
          '*' => {
              st.typ = TyList;
              st.in = in_number;
              st.number = 0;
              st.negative_number = false;
            }
          '$' => {
              st.typ = TyData;
              st.in = in_number;
              st.number = 0;
              st.negative_number = false;
           }
          ':' => {
              st.typ = TyInt;
              st.in = in_number;
              st.number = 0;
              st.negative_number = false;
           }

           _ => {
              return (Error, buf) // XXX: return a slice including the character that failed
           }
        }

      }

      in_data => {
        if (buf.is_empty()) {break}
        let (data, b) = take_atmost_n(buf, st.data_size - st.data_have);
        buf = b;
        visitor.on_data(data);
        st.data_have += data.len();
        assert!(st.data_have <= st.data_size);
        if st.data_have == st.data_size {
          //XXX: st.in = in_data_need_newline;
          st.in = in_nothing;
          if visitor.on_data_end() {
            return (Finished, buf)
          }
        }
      }

      // we could treat the newline after data differently.
/*
      in_data_need_newline => {
        if (buf.is_empty()) {break}
        let (c, b) = take_head(buf);
        buf = b;
        if c == ('\n' as u8) {
          st.in = in_nothing;
        }
        else if c == ('\r' as u8) || c == (' ' as u8) {
          error("Consume whitespace");
        }
        else {
          return (Error, buf)
        }
      }
*/

      in_number_need_newline => {
        if (buf.is_empty()) {break}
        let (c, b) = take_head(buf);
        buf = b;
        if c == ('\n' as u8) {
          st.in = in_number_done;
        }
        else {
          return (Error, buf)
        }
      }
   
      // XXX: make a function instead of a STATE
      in_number_done => {
        match st.typ {
          TyData => {
            if st.negative_number {
              if st.number == 1 {
                st.in = in_nothing;
                if visitor.on_nil() {
                  return (Finished, buf)
                }
              }
              else {
                return (Error, buf)
              }
            }
            else {
              st.data_size = st.number;
              st.data_have = 0;
              st.in = in_data;
              visitor.on_data_beg(st.data_size);
            }
          }
          TyList => {
            // XXX:
            // push current recursion level and index on stack
            st.in = in_nothing;
            if st.negative_number {
              if (st.number == 1) {
                if visitor.on_nil() {
                  return (Finished, buf)
                }
              }
              else {
                return (Error, buf)
              }
            }
            else {
              if visitor.on_list(st.number) {
                return (Finished, buf)
              }
            }
          }

          TyInt => {
            st.in = in_nothing;
            if visitor.on_integer(st.number, st.negative_number) {
              return (Finished, buf)
            }
          }
          _ => {
            fail!(~"THIS SHOULD NEVER HAPPEN")
          }
        }
      }

      in_number | in_number_digits_only => {
        if (buf.is_empty()) {break}
        let (c, b) = take_head(buf);
        buf = b;

        if c >= ('0' as u8) && c <= ('9' as u8) {
          st.number *= 10;
          st.number += (c - ('0' as u8)) as uint;
          st.in = in_number_digits_only;
        }
        else if c as char == '-' {
          match st.in {
            in_number_digits_only => {
              return (Error, buf)
            }
            _ => {
              st.negative_number = true;
              st.in = in_number_digits_only;
            }
          }
        }
        else if c as char == '\r' || c as char == ' ' {
          st.in = in_number_need_newline;
        }
        else if c as char == '\n' {
          st.in = in_number_done;
        }
        else {
          return (Error, buf)
        }

      }
    }
  }
  (NeedMore, buf)
}


fn main() {
  let mut st = RedisState {
    in: in_nothing,
    typ: TyNone, 
    data_size: 0,
    data_have: 0,
    number: 0,
    negative_number: false,
  }; 

  error!("%?", st);

  let mut visitor = MyVisitor {stack: ~[], result: Invalid};

  let s = ~"*4\r\n$3\r\nabc\r\n:123\n:1\n$-1\n";
  let slice = str::as_bytes_slice(s);
  let x = parse_redis(&mut st, slice, &mut visitor);
  error!("%?", st);
  error!("%?", x);
  error!("%?", visitor.result);
}
