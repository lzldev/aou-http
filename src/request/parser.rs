use crate::{request::RequestHeaderParser, utils::range_from_subslice};

use super::{RequestHead, RequestHeaders, VecOffset};

pub enum ParseResponse {
  Success(RequestParser),
  Incomplete((Vec<u8>, ParserState)),
}

#[derive(Debug)]
pub struct RequestParser {
  buf: Vec<u8>,
  head: RequestHead,
  headers: RequestHeaders,
  body: VecOffset,
}

#[derive(Debug)]
pub enum ParserState {
  New,
  Head {
    cursor: usize,
    head: RequestHead,
  },
  Headers {
    cursor: usize,
    head: RequestHead,
    headers: RequestHeaders,
  },
  Body {
    cursor: usize,
    head: RequestHead,
    headers: RequestHeaders,
    body: VecOffset,
  },
}

impl RequestParser {
  pub fn parse_request(buf: Vec<u8>, state: ParserState) -> ParseResponse {
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

    let head = match RequestHead::from_split_iter(&mut lines, &buf) {
      Ok((size, head)) => {
        offset = offset + size;
        head
      }
      Err(_) => return ParseResponse::Incomplete((buf, ParserState::New)),
    };

    let headers = match RequestHeaderParser::parse_headers(&buf, lines) {
      Ok((size, headers)) => {
        offset = offset + size;
        headers
      }
      Err(_) => {
        return ParseResponse::Incomplete((
          buf,
          ParserState::Head {
            cursor: offset,
            head,
          },
        ))
      }
    };

    let buf_len = buf.len();

    debug_assert!(
      offset <= buf_len,
      "Buf:{:#?}\nOffset larger than buffer size : Offset {offset} : Buf {buf_len} Headers:{}",
      String::from_utf8_lossy(buf.as_slice()),
      headers.len()
    );

    let body = &buf[offset..];
    let body = range_from_subslice(&buf, body);

    let req = RequestParser {
      buf,
      head,
      headers,
      body,
    };

    ParseResponse::Success(req)
  }
}

#[cfg(test)]
mod test {
  use tokio::time::Instant;

  use crate::request::{ParseResponse, RequestParser};

  const GET_REQUEST_MOCK: &[u8] = b"GET / HTTP/1.1
Host: www.example.com
Accept-Language: en

";

  fn print_parser_memory(req: RequestParser) {
    let size_buf = std::mem::size_of_val(POST_REQUEST_MOCK);
    let size_u8 = std::mem::size_of_val(&255u8);
    let size = std::mem::size_of_val(&req);
    let size_req_buf = std::mem::size_of_val(&req.buf);
    let size_headers = std::mem::size_of_val(&req.headers);
    let size_head = std::mem::size_of_val(&req.head);
    let size_body = std::mem::size_of_val(req.body);

    eprintln!("Request: {req:?}");
    eprintln!("Sizes ---");
    eprintln!("U8: {size_u8:?}");
    eprintln!("Original: {size_buf:#?}");
    eprintln!("Request: {size:#?}");
    eprintln!("Internal---");
    eprintln!("  Raw: {size_req_buf:#?}");
    eprintln!("  Head: {size_head:#?}");
    eprintln!("  Headers: {size_headers:#?}");
    eprintln!("  Body: {size_body:#?}");
    eprintln!("    TOTAL: {:#?}", size_head + size_headers + size_body);
  }

  #[tokio::test]
  async fn test_parse_request() {
    let mut mutable_mock = GET_REQUEST_MOCK.to_owned();

    let req = match RequestParser::parse_request(mutable_mock) {
      ParseResponse::Success(r) => r,
      ParseResponse::Failed(_) => panic!("Parse Failed"),
    };

    print_parser_memory(req);
  }

  const POST_REQUEST_MOCK: &[u8] = b"POST /create HTTP/1.1
Host: www.example.com
Accept-Language: en
Content-Type: application/json

{
  \"hello\":\"world\"
}";
  #[tokio::test]
  async fn test_parse_request_with_body() {
    let mut mutable_mock = POST_REQUEST_MOCK.to_owned();
    let req = RequestParser::parse_request(&mut mutable_mock).unwrap();

    let size_buf = std::mem::size_of_val(POST_REQUEST_MOCK);
    let size_u8 = std::mem::size_of_val(&255u8);
    let size = std::mem::size_of_val(&req);
    let size_req_buf = std::mem::size_of_val(req.buf);
    let size_headers = std::mem::size_of_val(&req.headers);
    let size_head = std::mem::size_of_val(&req.head);
    let size_body = std::mem::size_of_val(req.body);

    eprintln!("Request: {req:?}");
    eprintln!("Sizes ---");
    eprintln!("U8: {size_u8:?}");
    eprintln!("Original: {size_buf:#?}");
    eprintln!("Request: {size:#?}");
    eprintln!("Internal---");
    eprintln!("  Raw: {size_req_buf:#?}");
    eprintln!("  Head: {size_head:#?}");
    eprintln!("  Headers: {size_headers:#?}");
    eprintln!("  Body: {size_body:#?}");
    eprintln!("    TOTAL: {:#?}", size_head + size_headers + size_body);

    assert_eq!(
      req.head.method, b"POST",
      "Request method doesn't match MOCK Method"
    );
    assert_eq!(
      size_body,
      std::mem::size_of_val(
        b"{
  \"hello\":\"world\"
}"
      ),
      "Body bigger than actual json"
    );
  }

  #[tokio::test]
  async fn bench_parse_request() {
    let mut mutable_mock = POST_REQUEST_MOCK.to_owned();
    let size = 20_000_000;

    let start = Instant::now();
    for _ in 0..size {
      let _ = RequestParser::parse_request(&mut mutable_mock);
    }
    eprintln!("ITER TOOK: {:?}", start.elapsed());

    let start = Instant::now();
    // for _ in 0..size {
    //   let _ = Request::parse_request_with_vec(POST_REQUEST_MOCK);
    // }
    eprintln!("WITH VEC TOOK: {:?}", start.elapsed());
  }
}
