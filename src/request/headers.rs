use crate::utils::range_from_subslice;

use super::{
  options::{Connection, HeaderOptions},
  VecOffset,
};

pub type RequestHeaders = Vec<RequestHeaderVec>;
pub type RequestHeaderVec = (VecOffset, VecOffset);

#[derive(Debug, PartialEq)]
pub enum HeaderParseError {
  Incomplete,
  Invalid,
  NoHost,
}

#[derive(Debug)]
pub struct HeaderParserResult {
  pub size: usize,
  pub headers: RequestHeaders,
  pub options: HeaderOptions,
}

pub struct HeaderParser;
impl HeaderParser {
  pub fn parse_headers<P>(
    buf: &[u8],
    lines: std::slice::Split<u8, P>,
  ) -> Result<HeaderParserResult, HeaderParseError>
  where
    P: FnMut(&u8) -> bool,
  {
    let mut offset: usize = 0;
    let mut has_host = false;
    let mut connection = Connection::KeepAlive;

    let header_lines = lines.take_while(|b| *b != b"" && *b != b"\r");

    let mut headers = Vec::new();

    for header in header_lines {
      offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset
      let mut split = header.splitn(2, |b| b == &b':'); // TODO: This could be &b': '

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
      } else if header.eq_ignore_ascii_case(b"connection")
        && value.eq_ignore_ascii_case(b" close\r\n")
      {
        connection = Connection::Close
      };

      let header = range_from_subslice(buf, header);
      let value = range_from_subslice(buf, &value[1..value.len() - 1]);

      headers.push((header, value))
    }

    if headers.len() == 0 {
      return Err(HeaderParseError::Incomplete);
    }

    if has_host == false {
      return Err(HeaderParseError::NoHost);
    }

    Ok(HeaderParserResult {
      headers,
      size: offset,
      options: HeaderOptions { connection },
    })
  }
}

#[cfg(test)]
mod unit_tests {
  use crate::request::{HeaderParseError, HeaderParser, HeaderParserResult, RequestParser};

  #[tokio::test]
  async fn regular_request_headers() {
    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = HeaderParser::parse_headers(buf, lines);
    let HeaderParserResult { size, headers, .. } = parser.unwrap();

    assert_eq!(size, 89, "Invalid Offset");
    assert_eq!(headers.len(), 3, "Invalid amount of headers parsed");
  }

  #[tokio::test]
  async fn ignore_leading_and_trailing_characters() {
    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let HeaderParserResult { headers, .. } = HeaderParser::parse_headers(buf, lines).unwrap();

    let (_, value) = headers.iter().nth(1).unwrap();
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

    let parser = HeaderParser::parse_headers(buf, lines);
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

    let parser = HeaderParser::parse_headers(buf, lines);
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

    let parser = HeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_eq!(err, HeaderParseError::NoHost, "should be Invalid");

    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = HeaderParser::parse_headers(buf, lines);

    assert!(parser.is_ok(), "should be Valid {parser:?}");
  }

  #[tokio::test]
  async fn request_with_server_123() {
    let buf = b"Not-Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = HeaderParser::parse_headers(buf, lines);
    let err = parser.unwrap_err();

    assert_eq!(err, HeaderParseError::NoHost, "should be Invalid");

    let buf = b"Host: localhost:3000\r\nx-random-header: new_header\r\nUser-Agent: chrome-something:::::idk\r\n\r\n";
    let lines = RequestParser::split_buf_lines(buf);

    let parser = HeaderParser::parse_headers(buf, lines);

    assert!(parser.is_ok(), "should be Valid {parser:?}");
  }
}
