use super::{RequestHead, RequestHeaders};

pub struct Request<'req> {
  pub buf: Vec<u8>,
  pub head: RequestHead<'req>,
  pub headers: RequestHeaders<'req>,
  pub body: &'req [u8],
}
