use std::collections::HashMap;

use crate::request::{HeaderOptions, Request, RequestHead, RequestHeaders, VecOffset};

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
      crate::request::RequestOptions {
        connection: self.header_options.connection,
      },
    );
  }
}
