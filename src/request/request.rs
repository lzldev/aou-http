use core::str;
use std::collections::{BTreeMap, HashMap};

use super::{
  options::Connection, RequestHead, RequestHeaders, RequestOptions, RequestParser, VecOffset,
};

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
  #[napi(ts_type = "{}")]
  pub params: HashMap<String, String>,
  pub query: HashMap<String, String>,
  options: RequestOptions,
  cache: RequestFieldCache,
}

#[derive(Debug)]
struct RequestFieldCache {
  path: Option<String>,
  method: Option<String>,
  http_version: Option<String>,
  headers: Option<BTreeMap<String, String>>,
  body: Option<String>,
}
impl Default for RequestFieldCache {
  fn default() -> Self {
    Self {
      path: None,
      method: None,
      http_version: None,
      headers: None,
      body: None,
    }
  }
}

impl Default for Request {
  fn default() -> Self {
    Self {
      buf: Default::default(),
      head: Default::default(),
      headers: Default::default(),
      body: Default::default(),
      context: Default::default(),
      params: Default::default(),
      query: Default::default(),
      options: RequestOptions {
        connection: Connection::KeepAlive,
      },
      cache: Default::default(),
    }
  }
}

#[napi]
impl Request {
  pub fn new(
    buf: Vec<u8>,
    head: RequestHead,
    headers: RequestHeaders,
    body: VecOffset,
    query: HashMap<String, String>,
    params: HashMap<String, String>,
    options: RequestOptions,
  ) -> Request {
    Request {
      buf,
      head,
      headers,
      body,
      context: serde_json::Value::Object(Map::new()),
      params,
      query,
      options,
      ..Default::default()
    }
  }

  #[napi(factory)]
  pub fn from_string(request: String) -> Self {
    let parse = RequestParser::parse_request(
      Vec::from(request.as_bytes()),
      super::ParserState::Start { read_until: None },
    );

    let req = match parse {
      super::ParserStatus::Success(p) => p.into_request(),
      super::ParserStatus::Incomplete((buf, state)) => match state {
        super::ParserState::Body { .. } => state.into_parser_result(buf).unwrap().into_request(),
        _ => panic!("Incomplete Request"),
      },
      super::ParserStatus::Invalid(reason) => panic!("Failed to parse: {reason}"),
    };

    req
  }

  #[napi(getter)]
  pub fn method(&mut self) -> &str {
    if self.cache.method.is_some() {
      return self.cache.method.as_ref().unwrap();
    }
    let method =
      unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.method.0..self.head.method.1]) };

    self.cache.method = Some(method.to_string());

    method
  }

  #[napi(getter)]
  pub fn path(&mut self) -> &str {
    if self.cache.path.is_some() {
      return self.cache.path.as_ref().unwrap();
    }
    let path =
      unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.path.0..self.head.path.1]) };

    self.cache.path = Some(path.to_string());

    path
  }

  #[napi(getter)]
  pub fn http_version(&mut self) -> &str {
    if self.cache.http_version.is_some() {
      return self.cache.http_version.as_ref().unwrap();
    }

    let http_version = unsafe {
      std::str::from_utf8_unchecked(&self.buf[self.head.http_version.0..self.head.http_version.1])
    };
    self.cache.http_version = Some(http_version.to_string());

    http_version
  }

  #[napi(getter)]
  pub fn headers(&mut self) -> BTreeMap<String, String> {
    if self.cache.headers.is_some() {
      return self.cache.headers.as_ref().unwrap().clone();
    }

    let mut map = BTreeMap::<String, String>::new();
    unsafe {
      self.headers.iter().for_each(|v| {
        map.insert(
          std::str::from_utf8_unchecked(&self.buf[v.0 .0..v.0 .1]).to_string(),
          std::str::from_utf8_unchecked(&self.buf[v.1 .0..v.1 .1]).to_string(),
        );
      });
    }

    self.cache.headers = Some(map.clone());

    map
  }

  #[napi(getter)]
  pub fn body(&mut self) -> &str {
    if self.cache.body.is_some() {
      return self.cache.body.as_ref().unwrap();
    }

    let body = unsafe { std::str::from_utf8_unchecked(&self.buf[self.body.0..self.body.1]) };
    self.cache.body = Some(body.to_string());

    body
  }

  pub fn get_connection(&self) -> &Connection {
    &self.options.connection
  }
}
