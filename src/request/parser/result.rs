use std::collections::HashMap;

use crate::{
  request::{
    HeaderOptions, HeaderParseError, HeaderParserResult, Request, RequestHead, RequestHeaderParser,
    RequestHeaders, VecOffset,
  },
  utils::range_from_subslice,
};

use super::{ParserState, ParserStatus};

#[derive(Debug)]
pub struct ParserResult {
  pub buf: Vec<u8>,
  pub head: RequestHead,
  pub headers: RequestHeaders,
  pub body: VecOffset,
  pub header_options: HeaderOptions,
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

  pub fn parse_request(buf: Vec<u8>, _state: ParserState) -> ParserStatus {
    let buf_len = buf.len();
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

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

    let (headers, header_options) = match RequestHeaderParser::parse_headers(&buf, lines) {
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
    let body = range_from_subslice(&buf, body);

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
