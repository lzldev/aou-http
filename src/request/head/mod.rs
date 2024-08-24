use crate::utils::range_from_subslice;

mod error;
pub use error::*;

use super::VecOffset;

const HTTP1_1: &[u8] = b"HTTP/1.1\r";

#[derive(Debug, Default)]
pub struct RequestHead {
  pub method: VecOffset,
  pub path: VecOffset,
  pub http_version: VecOffset,
}

impl RequestHead {
  pub fn from_split_iter<'a>(
    iter: &mut std::slice::Split<'a, u8, impl FnMut(&u8) -> bool>,
    vec: &'a [u8],
  ) -> Result<(usize, RequestHead), RequestHeadParseError> {
    let mut offset: usize = 0;

    let head = iter.next().ok_or(RequestHeadParseError::NoHead)?;
    offset = offset.wrapping_add(head.len() + 1); // Add size of Head + \n to offset

    let mut head_split = head.split(|b| b == &b' ');

    let method = head_split.next().ok_or(RequestHeadParseError::NoMethod)?;
    let path = head_split.next().ok_or(RequestHeadParseError::NoPath)?;
    let http_version = head_split
      .next()
      .ok_or(RequestHeadParseError::NoHTTPVersion)?;

    if http_version != HTTP1_1 {
      //TODO: This should throw a invalid head http error
      return Err(RequestHeadParseError::InvalidHTTPVersion);
    }

    let method = range_from_subslice(vec, method);
    let path = range_from_subslice(vec, path);
    let http_version = range_from_subslice(vec, http_version);

    Ok((
      offset,
      RequestHead {
        method,
        path,
        http_version,
      },
    ))
  }
}
