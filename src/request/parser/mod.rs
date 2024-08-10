mod result;
mod state;
mod status;

pub use result::*;
pub use state::*;
pub use status::*;

pub struct RequestParser;
impl RequestParser {
  pub fn split_buf_lines<'a>(buf: &'a [u8]) -> std::slice::Split<'a, u8, impl FnMut(&u8) -> bool> {
    buf.split(|c| c == &b'\n')
  }
}

#[cfg(test)]
mod unit_tests {
  use crate::request::{ParserResult, ParserState};

  #[tokio::test]
  async fn parser_invalid_header_error() {
    let buf = b"GET /server_error123 HTTP/1.1\r\nHost: localhost:7070\r\nUser-Agent:";

    let parse = ParserResult::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      !parse.is_success(),
      "Parser should return a Invalid Header Error"
    );
  }
  #[tokio::test]
  async fn invalid_http_version() {
    let buf =
      b"GET / HTTP/1.0\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = ParserResult::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      !parse.is_success(),
      "Parse should return a Invalid HTTP Version Error Result"
    );
  }

  #[tokio::test]
  async fn invalid_header_whitespace() {
    let buf =
      b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nx-custom-header:invalid\r\nThe empty line before the body is missing";

    let parse = ParserResult::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(!parse.is_success(), "Parse should return a Invalid HEADER");
  }

  #[tokio::test]
  async fn invalid_header_termination() {
    let buf =
      b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = ParserResult::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_incomplete(),
      "Parse should return incomplete since it's not sure it's the end of the headers"
    );
  }
}
