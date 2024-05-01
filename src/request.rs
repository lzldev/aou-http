use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {}

impl Request {
  //   fn parse_request(buf: [u8]) -> Result<Request, anyhow::Error> {
  //     todo!()
  //   }
}

mod test {
  #[tokio::test]
  async fn parse_request() {}
}
