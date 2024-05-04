use anyhow::{anyhow, Context};

use crate::utils::range_from_subslice;

use super::VecOffset;

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
  ) -> Result<(usize, RequestHead), anyhow::Error> {
    let mut offset: usize = 0;
    let head = iter.next().context("Head not found?")?;
    offset = offset.wrapping_add(head.len() + 1); // Add size of Head + \n to offset
    let mut head_split = head.split(|b| b == &b' ');

    let method = head_split.next().ok_or(anyhow!("Method not found"))?;
    let path = head_split.next().ok_or(anyhow!("Path not found Path"))?;
    let http_version = head_split.next().ok_or(anyhow!("Http Version not found"))?;

    if http_version != b"HTTP/1.1" {
      return Err(anyhow!("Invalid HTTP Version"));
    }

    let method = range_from_subslice(vec, method);
    let path = range_from_subslice(vec, path);
    let http_version = range_from_subslice(vec, http_version);

    Ok::<_, anyhow::Error>((
      offset,
      RequestHead {
        method,
        path,
        http_version,
      },
    ))
  }
}
