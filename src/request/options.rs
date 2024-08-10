#[derive(Debug, PartialEq)]
pub enum Connection {
  KeepAlive,
  Close,
}

#[derive(Debug)]
pub struct RequestOptions {
  pub connection: Connection,
}

#[derive(Debug)]
pub struct HeaderOptions {
  pub connection: Connection,
  pub content_length: Option<usize>,
  //TODO add content type and ... in this struct
}

impl Default for HeaderOptions {
  fn default() -> Self {
    Self {
      connection: Connection::KeepAlive,
      content_length: None,
    }
  }
}
