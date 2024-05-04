use crate::utils::range_from_subslice;
use anyhow::anyhow;

use super::{RequestToken, VecOffset};

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
    let mut has_host = false;
    let mut offset: usize = 0;

    let headerLines = lines.take_while(|b| *b != b"" && *b != b"\r");
    let mut headers_vec = Vec::new();

    for header in headerLines {
      offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset
      let mut split = header.split(|b| b == &b':');

      let (header, value) = (
        split.next().ok_or(anyhow!("Invalid Header Name"))?,
        split.next().ok_or(anyhow!("Invalid Header Value"))?,
      );

      unsafe {
        if std::str::from_utf8_unchecked(header).eq_ignore_ascii_case("Host") {
          has_host = true;
        }
      }

      let header = range_from_subslice(buf, header);
      let value = range_from_subslice(buf, value);

      headers_vec.push((header, value))
    }

    // dbg!(has_host);

    if headers_vec.len() == 0 {
      return Err(anyhow!("No Headers"));
    } else if has_host == false {
      return Err(anyhow!("Host header not found yet."));
    }

    //unwrap: headers_vec already checked.
    let (_, last_key_token) = headers_vec.last().unwrap();
    let last_char = &buf[(last_key_token.1) - 1..last_key_token.1];
    if last_char != b"\r" {
      // dbg!(&last_char);
      return Err(anyhow!("Last Header was invalid."));
    }

    // let headers = lines
    //   .take_while(|b| *b != b"" && *b != b"\r") //TODO: Trim the \r correctly , And empty lines
    //   .filter_map(|header| {
    //     offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset

    //     let mut split = header.split(|b| b == &b':');
    //     let (header, value) = (split.next()?, split.next()?);

    //     let header = range_from_subslice(buf, header);
    //     let value = range_from_subslice(buf, value);

    //     Some((header, value))
    //   })
    //   .collect::<Vec<_>>();

    Ok((offset, headers_vec))
  }
}
