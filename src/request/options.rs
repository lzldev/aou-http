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
  pub has_host: bool, //TODO add content type and ... in this struct
}

impl Default for HeaderOptions {
  fn default() -> Self {
    Self {
      connection: Connection::KeepAlive,
      content_length: None,
      has_host: false,
    }
  }
}

impl HeaderOptions {
  pub fn merge(&mut self, other: HeaderOptions) {
    if other.connection != Connection::KeepAlive {
      self.connection = other.connection;
    }

    if self.content_length.is_none() && other.content_length.is_some() {
      self.content_length = other.content_length;
    }

    if !self.has_host && other.has_host {
      self.has_host = true;
    }
  }
}
