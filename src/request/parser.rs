use crate::{request::RequestHeaderParser, utils::range_from_subslice};

use super::{RequestHead, RequestHeaders, VecOffset};

#[derive(Debug)]
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
  pub fn parse_request(buf: Vec<u8>, state: ParserState) -> Result<ParseResponse, anyhow::Error> {
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

    let head = match RequestHead::from_split_iter(&mut lines, &buf) {
      Ok((size, head)) => {
        offset = offset + size;
        head
      }
      Err(head_parse_error) => {
        return Ok(ParseResponse::Incomplete((buf, ParserState::New)));
      }
    };

    let headers = match RequestHeaderParser::parse_headers(&buf, lines) {
      Ok((size, headers)) => {
        offset = offset + size;
        headers
      }
      Err(_) => {
        return Ok(ParseResponse::Incomplete((
          buf,
          ParserState::Head {
            cursor: offset,
            head,
          },
        )))
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

    Ok(ParseResponse::Success(req))
  }
}
