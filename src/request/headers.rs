use crate::utils::range_from_subslice;

use super::VecOffset;

pub type RequestHeaderVec = (VecOffset, VecOffset);
pub type RequestHeaders = Vec<RequestHeaderVec>;

pub struct RequestHeaderParser;
impl RequestHeaderParser {
  pub fn parse_headers<P>(
    buf: &[u8],
    lines: std::slice::Split<u8, P>,
  ) -> Result<(usize, RequestHeaders), anyhow::Error>
  where
    P: FnMut(&u8) -> bool,
  {
    let mut offset: usize = 0;

    let headers = lines
      .take_while(|b| *b != b"" && *b != b"\r") //TODO: Trim the \r correctly , And empty lines
      .filter_map(|header| {
        offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset

        let mut split = header.split(|b| b == &b':');
        let (header, value) = (split.next()?, split.next()?);

        let header = range_from_subslice(buf, header);
        let value = range_from_subslice(buf, value);

        Some((header, value))
      })
      .collect::<Vec<_>>();

    Ok((offset, headers))
  }
}
