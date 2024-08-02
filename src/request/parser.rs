use std::collections::HashMap;

use crate::{
  request::{HeaderParseError, RequestHeaderParser},
  utils::range_from_subslice,
};

use super::{Request, RequestHead, RequestHeaders, VecOffset};

pub struct RequestParser;
impl RequestParser {
  pub fn split_buf_lines<'a>(buf: &'a [u8]) -> std::slice::Split<'a, u8, impl FnMut(&u8) -> bool> {
    buf.split(|c| c == &b'\n')
  }
}

#[derive(Debug)]
pub struct ParserResult {
  pub buf: Vec<u8>,
  pub head: RequestHead,
  pub headers: RequestHeaders,
  pub body: VecOffset,
}

#[derive(Debug)]
pub enum ParserState {
  Start {
    read_until: Option<usize>,
  },
  Head {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
  },
  Headers {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
  },
  Body {
    cursor: usize,
    read_until: usize,
    head: RequestHead,
    headers: RequestHeaders,
    body: VecOffset,
  },
}

impl ParserState {
  pub fn read_until(&self) -> usize {
    match self {
      ParserState::Start { read_until } => read_until.unwrap_or(0),
      ParserState::Head { read_until, .. } => *read_until,
      ParserState::Headers { read_until, .. } => *read_until,
      ParserState::Body { read_until, .. } => *read_until,
    }
  }
}

#[derive(Debug)]
pub enum RequestParseResult {
  Success(ParserResult),
  Incomplete((Vec<u8>, ParserState)),
  Invalid(String),
}
impl RequestParseResult {
  pub fn is_incomplete(&self) -> bool {
    match self {
      RequestParseResult::Incomplete(_) => true,
      _ => false,
    }
  }
  pub fn is_success(&self) -> bool {
    match self {
      RequestParseResult::Success(_) => true,
      _ => false,
    }
  }
  pub fn is_invalid(&self) -> bool {
    match self {
      RequestParseResult::Invalid(_) => true,
      _ => false,
    }
  }
}

impl ParserResult {
  pub fn into_request(self) -> Request {
    let path =
      unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.path.0..self.head.path.1]) };

    let query = {
      let (_, query) = path.split_once('?').unwrap_or(("", ""));

      query
        .to_owned()
        .split("&")
        .map(|p| p.split_once("=").unwrap_or((p, "")))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect::<HashMap<String, String>>()
    };

    return Request::new(
      self.buf,
      self.head,
      self.headers,
      self.body,
      query,
      HashMap::new(),
    );
  }

  pub fn parse_request(buf: Vec<u8>, _state: ParserState) -> RequestParseResult {
    let buf_len = buf.len();
    let mut offset: usize = 0;

    let mut lines = buf.split(|b| b == &b'\n');

    let head = match RequestHead::from_split_iter(&mut lines, &buf) {
      Ok((size, head)) => {
        offset = offset + size;
        head
      }
      Err(_) => {
        return RequestParseResult::Incomplete((
          buf,
          ParserState::Start {
            read_until: Some(buf_len),
          },
        ));
      }
    };

    let headers = match RequestHeaderParser::parse_headers(&buf, lines) {
      Ok((size, headers)) => {
        offset = offset + size;
        headers
      }
      Err(HeaderParseError::Incomplete) | Err(HeaderParseError::Invalid) => {
        return RequestParseResult::Incomplete((
          buf,
          ParserState::Head {
            cursor: offset,
            read_until: buf_len,
            head,
          },
        ));
      }
      Err(HeaderParseError::NoHost) => {
        return RequestParseResult::Invalid("Invalid Headers".into())
      }
    };

    let buf_len = buf.len();

    if offset >= buf_len {
      return RequestParseResult::Incomplete((
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
    let body = range_from_subslice(&buf, body);

    let req = ParserResult {
      buf,
      head,
      headers,
      body,
    };

    RequestParseResult::Success(req)
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
