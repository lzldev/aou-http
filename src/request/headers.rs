use core::str;

use crate::utils::range_from_subslice;

use super::VecOffset;

pub type RequestHeaderVec = (VecOffset, VecOffset);
pub type RequestHeaders = Vec<RequestHeaderVec>;

#[derive(Debug, PartialEq)]
pub enum HeaderParseError {
  Incomplete,
  Invalid,
  NoHost,
}

pub struct RequestHeaderParser;
impl RequestHeaderParser {
  pub fn parse_headers<P>(
    buf: &[u8],
    lines: std::slice::Split<u8, P>,
  ) -> Result<(usize, RequestHeaders), HeaderParseError>
  where
    P: FnMut(&u8) -> bool,
  {
    let mut offset: usize = 0;
    let mut has_host = false;

    let header_lines = lines.take_while(|b| *b != b"" && *b != b"\r");

    let mut headers_vec = Vec::new();

    for header in header_lines {
      offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset
      let mut split = header.splitn(2, |b| b == &b':');

      let (header, value) = (
        split.next().ok_or(HeaderParseError::Incomplete)?,
        split.next().ok_or(HeaderParseError::Incomplete)?,
      );

      if !value.starts_with(b" ") {
        //format!( "Header without whitespace at buf {}", offset - header.len() - 1)
        return Err(HeaderParseError::Invalid);
      } else if !value.ends_with(b"\r") {
        return Err(HeaderParseError::Incomplete);
      };

      if has_host == false && header.eq_ignore_ascii_case(b"host") {
        has_host = true;
      }

      let header = range_from_subslice(buf, header);
      let value = range_from_subslice(buf, &value[1..value.len() - 1]);

      headers_vec.push((header, value))
    }

    if headers_vec.len() == 0 {
      return Err(HeaderParseError::Incomplete);
    }

    if has_host == false {
      return Err(HeaderParseError::NoHost);
    }

    Ok((offset, headers_vec))
  }
}

#[cfg(test)]
mod unit_tests {
  use core::str;

  use crate::request::{HeaderParseError, RequestHeaderParser, RequestParser};

  #[tokio::test]
  async fn regular_request_headers() {
    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);
    let (offset, headers) = parser.unwrap();

    assert_eq!(offset, 89, "Invalid Offset");
    assert_eq!(headers.len(), 3, "Invalid amount of headers parsed");
  }

  #[tokio::test]
  async fn ignore_leading_and_trailing_characters() {
    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines).unwrap();

    let (_, value) = parser.1.iter().nth(1).unwrap();
    let header_value = &buf[value.0..value.1];

    assert_eq!(
      header_value, b"new_header",
      "Header should be clean of escape values"
    )
  }

  #[tokio::test]
  async fn incomplete_request_headers() {
    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-s";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_eq!(
      err,
      HeaderParseError::Incomplete,
      "Parser didn't return a incomplete result"
    );
  }

  #[tokio::test]
  async fn invalid_not_incomplete_request_headers() {
    let buf = b"x-random-header: new_header\r\nUser-Agent: chrome-someth";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_ne!(
      err,
      HeaderParseError::Invalid,
      "Parser should return incomplete and not Invalid"
    );
    assert_eq!(
      err,
      HeaderParseError::Incomplete,
      "Parser should return a Incomplete Result"
    );
  }

  #[tokio::test]
  async fn request_without_host_is_invalid() {
    let buf = b"Not-Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_eq!(err, HeaderParseError::NoHost, "should be Invalid");

    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);

    assert!(parser.is_ok(), "should be Valid {parser:?}");
  }

  #[tokio::test]
  async fn request_with_server_123() {
    let buf = b"Not-Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_eq!(err, HeaderParseError::NoHost, "should be Invalid");

    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = RequestHeaderParser::parse_headers(buf, lines);

    assert!(parser.is_ok(), "should be Valid {parser:?}");
  }
}
