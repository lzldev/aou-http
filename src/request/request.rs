use super::{RequestHead, RequestHeaders, VecOffset};
use napi_derive::napi;

#[napi]
#[derive(Debug, Default)]
pub struct Request {
  buf: Vec<u8>,
  head: RequestHead,
  headers: RequestHeaders,
  body: VecOffset,
}

#[napi]
impl Request {
  pub fn new(buf: Vec<u8>, head: RequestHead, headers: RequestHeaders, body: VecOffset) -> Request {
    Request {
      buf,
      head,
      headers,
      body,
    }
  }

  #[napi]
  pub fn path(&self) -> String {
    let (start, end) = self.head.path;
    let slice = &self.buf[start..end];

    String::from_utf8(slice.to_owned()).expect("Couldn't convert path to String")
  }
}
