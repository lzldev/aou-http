#[derive(Debug, PartialEq)]
pub enum Connection {
  KeepAlive,
  Close,
}

#[derive(Debug)]
pub struct RequestOptions {
  connection: Connection,
}

#[derive(Debug)]
pub struct HeaderOptions {
  pub connection: Connection,
  //TODO add content type and ... in this struct
}
