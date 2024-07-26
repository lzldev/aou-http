use core::str;

use super::{RequestHead, RequestHeaders, VecOffset};

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
#[derive(Debug)]
pub struct AouRequest {
  buf: Vec<u8>,
  head: RequestHead,
  headers: RequestHeaders,
  body: VecOffset,
}

#[napi]
impl AouRequest {
  pub fn new(
    buf: Vec<u8>,
    head: RequestHead,
    headers: RequestHeaders,
    body: VecOffset,
  ) -> AouRequest {
    AouRequest {
      buf,
      head,
      headers,
      body,
    }
  }
  #[napi]
  pub fn method(&self) -> String {
    return String::from_utf8_lossy(&self.buf[self.head.method.0..self.head.method.1]).into();
    // unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.method.0..self.head.method.1]) }
  }

  #[napi]
  pub fn path(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.buf[self.head.path.0..self.head.path.1]) }
  }

  #[napi]
  pub fn http_version(&self) -> &str {
    unsafe {
      std::str::from_utf8_unchecked(&self.buf[self.head.http_version.0..self.head.http_version.1])
    }
  }
}
