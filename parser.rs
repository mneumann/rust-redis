
  fn take_n(v: &r/[u8], n: uint) -> (&r/[u8], &r/[u8]) {
    (vec::view(v, 0, n), vec::view(v, n, v.len()))
  }

  fn take_atmost_n(v: &r/[u8], n: uint) -> (&r/[u8], &r/[u8]) {
    take_n(v, uint::min(n, v.len()))
  }

  fn take_head(v: &r/[u8]) -> (u8, &r/[u8]) {
    assert v.is_not_empty();
    (v.head(), vec::view(v, 1, v.len()))
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

  //in_number(TYPE), 
  //in_number_done(TYPE),
  //in_number_need_newline(TYPE)

  struct RedisState {
    mut in: Inside,
    mut typ: RedisType,
    mut data_size: uint,
    mut data_have: uint,
    mut number: uint,
    mut negative_number: bool,
    mut stack: ~[(uint, uint)] // (len, idx)
  }

  enum RedisResult {
    Finished,
    NeedMore,
    Error
  }
  
  // returns true if we are finished parsing a redis request
  fn element_done(st: &mut RedisState) -> bool {
    if st.stack.is_empty() {
      true
    }
    else {
      let (len, idx) = st.stack.last();
      assert idx+1 <= len;
      st.stack[st.stack.len()-1] = (len, idx+1);
      if idx + 1 == len {
        error!("Finsihed list");
        let _ = vec::pop(&mut st.stack); 
        element_done(st)
      }
      else {
        false
      }
    }
  }

  trait Visitor {
    fn on_list(rec: int, len: uint);
    //fn on_list_elt(rec: int, len: uint);
  }

// keep recursion in visitor?
  struct MyVisitor {
    i: int
  }

  impl MyVisitor : Visitor {
    fn on_list(rec: int, len: uint) {
      error!("on_list(len: %u)", len)
    }
  }

  fn parse_redis(st: &mut RedisState, buf: &r/[u8]) -> (RedisResult, &r/[u8]) {
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
          error!("GOT data: %?", data);
          st.data_have += data.len();
          assert st.data_have <= st.data_size;
          if st.data_have == st.data_size {
            error!("GOT DATA COMPLETE");
            //st.in = in_data_need_newline;
            st.in = in_nothing;
            if element_done(st) {
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
                  error!("GOT NIL VALUE");
                  st.in = in_nothing;
                  if element_done(st) {
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
              }
            }
            TyList => {
              // XXX:
              // push current recursion level and index on stack
              st.in = in_nothing;
              if st.negative_number {
                if (st.number == 1) {
                  // NIL
                  if element_done(st) {
                    return (Finished, buf)
                  }
                }
                else {
                  return (Error, buf)
                }
              }
              else {
                if st.number > 0 {
                  vec::push(&mut st.stack, (st.number, 0));
                }
                else {
                  error!("GOT EMPTY LIST");
                  if element_done(st) {
                    return (Finished, buf)
                  }
                }
              }
            }

            TyInt => {
              error!("GOT INTEGER: %?", st.number);
              st.in = in_nothing;
              if element_done(st) {
                return (Finished, buf)
              }
            }
            _ => {
              fail ~"THIS SHOULD NEVER HAPPEN"
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
            error!("number: %?", st.number);
            st.in = in_number_digits_only;
          }
          else if c as char == '-' {
            error!("NEGATIVE NUMBER");
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
            error!("number need newline");
            st.in = in_number_need_newline;
          }
          else if c as char == '\n' {
            error!("number finsihed");
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
    stack: ~[]
  }; 

  error!("%?", st);

  let s = ~"*4\r\n$3\r\nabc\r\n:123\n:1\n$-1\n";
  do str::as_bytes(&s) |v| {
    let x = parse_redis(&mut st, vec::view(*v, 0, (*v).len() - 1)); 
    error!("%?", st);
    error!("%?", x);
  }

}
