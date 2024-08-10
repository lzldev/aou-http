mod result;
mod state;
mod status;

pub use result::*;
pub use state::*;
pub use status::*;

use crate::{
  request::{HeaderParseError, HeaderParser, HeaderParserResult, RequestHead},
  utils,
};

pub struct RequestParser;
impl RequestParser {
  pub fn split_buf_lines<'a>(buf: &'a [u8]) -> std::slice::Split<'a, u8, impl FnMut(&u8) -> bool> {
    buf.split(|c| c == &b'\n')
  }

  pub fn parse_request(buf: Vec<u8>, _state: ParserState) -> ParserStatus {
    let buf_len = buf.len();
    let mut offset: usize = 0;
    let mut lines = RequestParser::split_buf_lines(&buf);

    //TODO: Turn this into a Option and Add a method to ParserState :: hasHead , ignoring this parsing if needed
    // Then Later add a method Called ParserState :: getHead -> Option<Head> then Head.or(parserState::getHead)
    let head = match RequestHead::from_split_iter(&mut lines, &buf) {
      Ok((size, head)) => {
        offset = offset + size;
        head
      }
      Err(_) => {
        return ParserStatus::Incomplete((
          buf,
          ParserState::Start {
            read_until: Some(buf_len),
          },
        ));
      }
    };

    let (headers, header_options) = match HeaderParser::parse_headers(&buf, lines) {
      Ok(HeaderParserResult {
        size,
        headers,
        options,
      }) => {
        offset = offset + size;
        (headers, options)
      }
      Err(HeaderParseError::Incomplete) | Err(HeaderParseError::Invalid) => {
        return ParserStatus::Incomplete((
          buf,
          ParserState::Head {
            cursor: offset,
            read_until: buf_len,
            head,
          },
        ));
      }
      Err(HeaderParseError::NoHost) => return ParserStatus::Invalid("Invalid Headers".into()),
    };

    let buf_len = buf.len();

    if offset >= buf_len {
      return ParserStatus::Incomplete((
        buf,
        ParserState::Head {
          cursor: offset,
          read_until: buf_len,
          head,
        },
      ));
    }

    //TODO:This means only the header has been read.
    //should be chekced and not panic
    debug_assert!(
      offset <= buf_len,
      "Buf:{:#?}\nOffset larger than buffer size : Offset {offset} : Buf {buf_len} Headers:{}",
      String::from_utf8_lossy(buf.as_slice()),
      headers.len()
    );

    let body = &buf[offset..];
    let body = utils::range_from_subslice(&buf, body);

    let req = ParserResult {
      buf,
      head,
      headers,
      body,
      header_options,
    };

    ParserStatus::Success(req)
  }
}

#[cfg(test)]
mod unit_tests {
  use crate::request::{ParserState, RequestParser};

  #[tokio::test]
  async fn parser_invalid_header_error() {
    let buf = b"GET /server_error123 HTTP/1.1\r\nHost: localhost:7070\r\nUser-Agent:";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      !parse.is_success(),
      "Parser should return a Invalid Header Error"
    );
  }

  #[tokio::test]
  async fn invalid_http_version() {
    let buf =
      b"GET / HTTP/1.0\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      !parse.is_success(),
      "Parse should return a Invalid HTTP Version Error Result"
    );
  }

  #[tokio::test]
  async fn invalid_header_whitespace() {
    let buf =
      b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nx-custom-header:invalid\r\nThe empty line before the body is missing";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(!parse.is_success(), "Parse should return a Invalid HEADER");
  }

  #[tokio::test]
  async fn invalid_header_termination() {
    let buf =
      b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nThe empty line before the body is missing";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_incomplete(),
      "Parse should return incomplete since it's not sure it's the end of the headers"
    );
  }
}
