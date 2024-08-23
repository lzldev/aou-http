use crate::{
  constants::{CRLF_SIZE, LF},
  request::{
    HeaderParseError, HeaderParser, HeaderParserResult, RequestHead, RequestHeadParseError,
  },
  utils,
};
use std::slice::Split;

use super::{FullParserState, ParserResult, ParserState, ParserStatus};

pub struct RequestParser;
impl RequestParser {
  pub fn parse_request(buf: Vec<u8>, _state: ParserState) -> ParserStatus {
    let buf_len = buf.len();
    let mut offset: usize = 0;
    let mut lines = RequestParser::split_buf_lines(&buf);

    let FullParserState {
      read_until: _,
      cursor,
      head,
      headers,
      header_options,
      body: _,
    } = FullParserState::from_state(_state);

    let head = match head
      .and_then(|v| {
        offset = offset + (v.http_version.1 + 1);
        lines.advance_by(1).expect("Advanced lines by too much");

        Some(v)
      })
      .map_or_else(
        || {
          let (size, head) = RequestHead::from_split_iter(&mut lines, &buf)?;
          offset = offset + size;

          return Ok::<RequestHead, RequestHeadParseError>(head);
        },
        |ok| Ok(ok),
      ) {
      Ok(head) => head,
      Err(RequestHeadParseError::InvalidHTTPVersion) => {
        return ParserStatus::Invalid("HTTP Version Not supported".into());
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

    let (headers, header_options) = match (headers, header_options) {
      (Some(mut headers), Some(mut header_options)) => {
        offset = cursor.unwrap_or(offset);
        let n = headers.len();
        lines.advance_by(n).expect("Advanced Lines by too Much");

        let mut peeker = lines.clone().peekable();

        if let Some(&b"\r") = peeker.peek() {
          (headers, header_options)
        } else {
          match HeaderParser::parse_headers(&buf, &mut lines) {
            Ok(HeaderParserResult {
              size,
              headers: mut headers2,
              options,
            }) => {
              offset = offset + size;

              header_options.merge(options);

              headers.append(&mut headers2);
              (headers, header_options)
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
          }
        }
      }
      (_, _) => match HeaderParser::parse_headers(&buf, &mut lines) {
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
      },
    };

    if !header_options.has_host {
      //TODO: Make this a real error
      return ParserStatus::Invalid("Invalid Headers".into());
    }

    let buf_len = buf.len();

    if offset >= buf_len {
      return ParserStatus::Incomplete((
        buf,
        ParserState::Headers {
          cursor: offset,
          read_until: buf_len,
          head,
          headers,
          header_options,
        },
      ));
    }

    //TODO:This means only the header has been read.
    //should be checked and not panic
    debug_assert!(
      offset <= buf_len,
      "Buf:{:#?}\nOffset larger than buffer size : Offset {offset} : Buf {buf_len} Headers:{}",
      String::from_utf8_lossy(buf.as_slice()),
      headers.len()
    );

    //TODO: This should be a peekable iterator from the start.
    let mut lines = lines.peekable();

    if let None = lines.peek() {
      return ParserStatus::Incomplete((
        buf,
        ParserState::Headers {
          cursor: offset,
          read_until: buf_len,
          head,
          headers,
          header_options,
        },
      ));
    };

    let (body, content_length) = if let Some(content_length) = header_options.content_length {
      if buf_len < offset + CRLF_SIZE + content_length {
        return ParserStatus::Incomplete((
          buf,
          ParserState::Headers {
            cursor: offset,
            read_until: buf_len,
            head,
            headers,
            header_options,
          },
        ));
      }

      (
        &buf[offset + CRLF_SIZE..offset + CRLF_SIZE + content_length],
        content_length,
      )
    } else {
      (&buf[offset..offset], 0)
    };

    let body = utils::range_from_subslice(&buf, body);
    let body_len = body.1 - body.0;

    if body_len == content_length {
      return ParserStatus::Success(ParserResult {
        buf,
        head,
        headers,
        body,
        header_options,
      });
    }

    return ParserStatus::Incomplete((
      buf,
      ParserState::Body {
        cursor: offset,
        read_until: buf_len,
        head,
        headers,
        header_options,
        body,
      },
    ));
  }

  pub fn split_buf_lines<'a>(buf: &'a [u8]) -> Split<'a, u8, impl FnMut(&u8) -> bool + Clone> {
    buf.split(|c| c == &LF)
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

  #[tokio::test]
  async fn respect_content_length() {
    let buf =
      b"POST / HTTP/1.1\r\nHost: localhost:3000\r\nContent-Length: 14\r\n\r\n{\"valid\":true}";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_success(),
      "This request should be considered complete"
    );
  }

  #[tokio::test]
  async fn respect_content_length_incomplete() {
    let buf = b"POST / HTTP/1.1\r\nHost: localhost:3000\r\nContent-Length: 14\r\n\r\n{\"vali";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(
      parse.is_incomplete(),
      "Request body is smaller than the content-length"
    );
  }

  #[tokio::test]
  async fn invalid_head_version() {
    let buf = b"POST / HTTP/1.0\r\nHost: localhost:3000\r\nContent-Length: 14\r\n\r\n{\"vali";

    let parse = RequestParser::parse_request(buf.into(), ParserState::Start { read_until: None });

    assert!(parse.is_invalid(), "Header should be invalid");
  }
}
