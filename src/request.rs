use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {}

impl Request {
  //   fn parse_request(buf: [u8]) -> Result<Request, anyhow::Error> {
  //     todo!()
  //   }
}

impl Request {
  pub fn parse_request_with_vec(buf: &[u8]) {
    let mut lines = buf.split(|b| b == &b'\n');

    let head = lines.next().unwrap();

    let mut head_split = head.split(|b| b == &b' ');

    let (method, path, http_version) = (
      head_split.next().unwrap(),
      head_split.next().unwrap(),
      head_split.next().unwrap(),
    );

    assert_eq!(method, b"GET");
    assert_eq!(path, b"/");
    assert_eq!(http_version, b"HTTP/1.1");

    let mut header_vec: Vec<(&[u8], &[u8])> = Vec::new();

    for line in lines.by_ref() {
      if line == b"" {
        break;
      }
      let mut split = line.split(|b| b == &b':');

      let (header, value) = (split.next().unwrap(), split.next().unwrap());

      //TODO:REMOVE DEBUG
      let hh = String::from_utf8_lossy(header);
      let hv = String::from_utf8_lossy(value);

      // eprintln!("iter -> k/v | {hh}|{hv}");

      header_vec.push((header, value));
    }

    assert_eq!(header_vec.len(), 2);

    let lines = lines.skip(header_vec.len());

    let hint = lines.size_hint();
    let col = lines.collect::<Vec<_>>();
    // eprintln!("HINT: {hint:?}");
    // eprintln!("col: {col:?}");
  }

  pub fn parse_request(buf: &[u8]) -> (&[u8], &[u8]) {
    let mut offset: usize = 0;
    let mut lines = buf.split(|b| b == &b'\n');

    let head = lines.next().unwrap();
    offset = offset.wrapping_add(head.len() + 1); // Add Headsize + \n to offset

    let mut head_split = head.split(|b| b == &b' ');

    let (method, path, http_version) = (
      head_split.next().unwrap(),
      head_split.next().unwrap(),
      head_split.next().unwrap(),
    );

    assert_eq!(method, b"GET");
    assert_eq!(path, b"/");
    assert_eq!(http_version, b"HTTP/1.1");

    let headers = lines
      .clone()
      .take_while(|b| *b != b"")
      .map(|header| {
        offset = offset.wrapping_add(header.len() + 1); // Add line size + \n to offset

        let mut split = header.split(|b| b == &b':');

        let (header, value) = (split.next().unwrap(), split.next().unwrap());

        //TODO:REMOVE DEBUG
        let hh = String::from_utf8_lossy(header);
        let hv = String::from_utf8_lossy(value);

        // eprintln!("iter -> k/v | {hh}|{hv}");

        (header, value)
      })
      .collect::<Vec<_>>();

    let lines = lines.skip(headers.len() + 1);
    offset = offset.wrapping_add(1); // Skip the empty line + \n

    let len = buf.len();
    assert!(
      offset <= len,
      "Offset larger than buffer size : Offset {offset} : Buf {len}"
    );

    let hint = lines.size_hint();
    let col = lines.collect::<Vec<_>>();
    let body = &buf[offset..];

    assert_eq!(body.len(), 0, "body lenght isn't 0");

    // eprintln!("HINT: {hint:?}");
    // eprintln!("col: {col:?}");
    (buf, body)
  }
}

#[cfg(test)]
mod test {
  use tokio::time::Instant;

  use crate::request::Request;

  const GET_REQUEST_MOCK: &[u8] = b"GET / HTTP/1.1
Host: www.example.com
Accept-Language: en

";

  #[tokio::test]
  async fn test_parse_request() {
    let req = Request::parse_request(GET_REQUEST_MOCK);

    eprintln!("{req:?}");
  }

  #[tokio::test]
  async fn bench_parse_request() {
    let size = 1_000_000;

    let start = Instant::now();
    for _ in 0..size {
      Request::parse_request(GET_REQUEST_MOCK);
    }
    eprintln!("ITER TOOK: {:?}", start.elapsed());

    let start = Instant::now();
    for _ in 0..size {
      Request::parse_request_with_vec(GET_REQUEST_MOCK);
    }
    eprintln!("WITH VEC TOOK: {:?}", start.elapsed());
  }
}
