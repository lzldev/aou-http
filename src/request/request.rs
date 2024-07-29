use core::str;
use std::collections::{BTreeMap};

use super::{RequestHead, RequestHeaders, RequestParser, VecOffset};

use napi_derive::napi;
use serde_json::Map;

#[napi(js_name = "AouRequest")]
#[derive(Debug)]
pub struct Request {
  buf: Vec<u8>,
  head: RequestHead,
  headers: RequestHeaders,
  body: VecOffset,
  #[napi(writable = true, enumerable = true)]
  pub context: serde_json::Value,
}

#[napi]
impl Request {
  pub fn new(buf: Vec<u8>, head: RequestHead, headers: RequestHeaders, body: VecOffset) -> Request {
    Request {
      buf,
      head,
      headers,
      body,
      context: serde_json::Value::Object(Map::new()),
    }
  }

  #[napi(factory)]
  pub fn from_string(request: String) -> Self {
    let parse = RequestParser::parse_request(
      Vec::from(request.as_bytes()),
      super::ParserState::Start { read_until: None },
    )
    .unwrap();

    let req = match parse {
      super::RequestParseResponse::Success(p) => p.into_request(),
      super::RequestParseResponse::Incomplete(_) => panic!(),
    };

    req
  }

  #[napi(getter)]
  pub fn method(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.method.0..self.head.method.1]) }
  }

  #[napi(getter)]
  pub fn path(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.path.0..self.head.path.1]) }
  }

  #[napi(getter)]
  pub fn http_version(&self) -> &str {
    unsafe {
      std::str::from_utf8_unchecked(&self.buf[self.head.http_version.0..self.head.http_version.1])
    }
  }

  #[napi(getter)]
  pub fn headers(&self) -> BTreeMap<String, String> {
    let mut map = BTreeMap::<String, String>::new();
    unsafe {
      self.headers.iter().for_each(|v| {
        map.insert(
          std::str::from_utf8_unchecked(&self.buf[v.0 .0..v.0 .1]).to_string(),
          std::str::from_utf8_unchecked(&self.buf[v.1 .0..v.1 .1]).to_string(),
        );
      });
    }

    map
  }

  #[napi(getter)]
  pub fn body(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.buf[self.body.0..self.body.1]) }
  }
}
