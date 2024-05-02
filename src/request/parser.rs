use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

use super::{Request, RequestHead, RequestHeaders};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestParser<'req> {
  pub buf: &'req [u8],
  pub head: RequestHead<'req>,
  pub headers: RequestHeaders<'req>,
  pub body: &'req [u8],
}

impl<'req> RequestParser<'req> {
  pub fn into_request(self) -> Request<'req> {
    Request {
      buf: self.buf.to_owned(),
      body: self.body,
      head: self.head,
      headers: self.headers,
    }
  }

  pub fn parse_request(buf: &mut [u8]) -> Result<RequestParser, anyhow::Error> {
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

    let (method, path, http_version) = {
      let head = lines.next().context("Head not found?")?;
      offset = offset.wrapping_add(head.len() + 1); // Add size of Head + \n to offset
      let mut head_split = head.split(|b| b == &b' ');

      let method = head_split.next().ok_or(anyhow!("Method not found"))?;
      let path = head_split.next().ok_or(anyhow!("Path not found Path"))?;
      let http_version = head_split.next().ok_or(anyhow!("Http Version not found"))?;

      assert_eq!(method[0], buf[0], "Method[0] and buf[0] not equal");
      assert_eq!(method[0], buf[0], "Method[0] and buf[0] not equal");

      Ok::<_, anyhow::Error>((method, path, http_version))
    }?;

    assert_eq!(http_version, b"HTTP/1.1\r");

    //TODO:Using a vec here might be faster (The streets said)
    let headers = lines
      .take_while(|b| *b != b"" && *b != b"\r") //TODO: Trim the \r correctly , And empty lines
      .filter_map(|header| {
        offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset

        let mut split = header.split(|b| b == &b':');

        let (header, value) = (split.next()?, split.next()?);

        Some((header, value))
      })
      .collect::<Vec<_>>();

    let buf_len = buf.len();
    // if offset < buf_len - 1 {
    // Check if the stream ends here
    //   offset = offset.wrapping_add(1)
    // }; // Skip the empty line between Headers and body + \n

    if offset >= buf_len {
      return Err(anyhow!("Offset Larger than buffer size."));
    }

    assert!(
      offset <= buf_len,
      "Buf:{:#?}\nOffset larger than buffer size : Offset {offset} : Buf {buf_len} Headers:{}",
      String::from_utf8_lossy(buf),
      headers.len()
    );

    let body = &buf[offset..];

    let head = RequestHead {
      method,
      path,
      http_version,
    };

    let req = RequestParser {
      buf,
      head,
      headers,
      body,
    };

    Ok(req)
  }
}

#[cfg(test)]
mod test {
  use tokio::time::Instant;

  use crate::request::RequestParser;

  const GET_REQUEST_MOCK: &[u8] = b"GET / HTTP/1.1
Host: www.example.com
Accept-Language: en

";

  #[tokio::test]
  async fn test_parse_request() {
    let mut mutable_mock = GET_REQUEST_MOCK.to_owned();
    let req = RequestParser::parse_request(&mut mutable_mock).unwrap();

    let size_buf = std::mem::size_of_val(GET_REQUEST_MOCK);
    let size_u8 = std::mem::size_of_val(&255u8);
    let size = std::mem::size_of_val(&req);
    let size_req_buf = std::mem::size_of_val(req.buf);
    let size_headers = std::mem::size_of_val(&req.headers);
    let size_head = std::mem::size_of_val(&req.head);
    let size_body = std::mem::size_of_val(req.body);

    eprintln!("Request: {req:?}");
    eprintln!("Sizes ---");
    eprintln!("U8: {size_u8:?}");
    eprintln!("Original: {size_buf:#?}");
    eprintln!("Request: {size:#?}");
    eprintln!("Internal---");
    eprintln!("  Raw: {size_req_buf:#?}");
    eprintln!("  Head: {size_head:#?}");
    eprintln!("  Headers: {size_headers:#?}");
    eprintln!("  Body: {size_body:#?}");
    eprintln!(
      "    TOTAL: {:#?}",
      size_req_buf + size_head + size_headers + size_body
    )
  }

  const POST_REQUEST_MOCK: &[u8] = b"POST /create HTTP/1.1
Host: www.example.com
Accept-Language: en
Content-Type: application/json

{
  \"hello\":\"world\"
}";
  #[tokio::test]
  async fn test_parse_request_with_body() {
    let mut mutable_mock = POST_REQUEST_MOCK.to_owned();
    let req = RequestParser::parse_request(&mut mutable_mock).unwrap();

    let size_buf = std::mem::size_of_val(POST_REQUEST_MOCK);
    let size_u8 = std::mem::size_of_val(&255u8);
    let size = std::mem::size_of_val(&req);
    let size_req_buf = std::mem::size_of_val(req.buf);
    let size_headers = std::mem::size_of_val(&req.headers);
    let size_head = std::mem::size_of_val(&req.head);
    let size_body = std::mem::size_of_val(req.body);

    eprintln!("Request: {req:?}");
    eprintln!("Sizes ---");
    eprintln!("U8: {size_u8:?}");
    eprintln!("Original: {size_buf:#?}");
    eprintln!("Request: {size:#?}");
    eprintln!("Internal---");
    eprintln!("  Raw: {size_req_buf:#?}");
    eprintln!("  Head: {size_head:#?}");
    eprintln!("  Headers: {size_headers:#?}");
    eprintln!("  Body: {size_body:#?}");
    eprintln!("    TOTAL: {:#?}", size_head + size_headers + size_body);

    assert_eq!(
      req.head.method, b"POST",
      "Request method doesn't match MOCK Method"
    );
    assert_eq!(
      size_body,
      std::mem::size_of_val(
        b"{
  \"hello\":\"world\"
}"
      ),
      "Body bigger than actual json"
    );
  }

  #[tokio::test]
  async fn bench_parse_request() {
    let mut mutable_mock = POST_REQUEST_MOCK.to_owned();
    let size = 20_000_000;

    let start = Instant::now();
    for _ in 0..size {
      let _ = RequestParser::parse_request(&mut mutable_mock);
    }
    eprintln!("ITER TOOK: {:?}", start.elapsed());

    let start = Instant::now();
    // for _ in 0..size {
    //   let _ = Request::parse_request_with_vec(POST_REQUEST_MOCK);
    // }
    eprintln!("WITH VEC TOOK: {:?}", start.elapsed());
  }
}
