use std::{borrow::BorrowMut, sync::LazyLock};

use tokio::{
  io::{AsyncWriteExt, Sink},
  sync::Mutex,
};

/**
 * Returns the buffer with a content length
 */
pub fn body_buf<'f>(body: &'static [u8]) -> (&'f [u8], usize) {
  (body, body.len())
}

pub trait BuilderWithBody {
  /**
  Appends a Content-Length and a Body Reader into a `tokio_test::io::Builder`

    Format:
    ```
    \r\ncontent-length: 5\r\n\r\n
    hello
    ```
    */
  fn with_body(&mut self, body: &'static [u8]) -> &mut tokio_test::io::Builder;
}

impl BuilderWithBody for tokio_test::io::Builder {
  fn with_body(&mut self, body: &'static [u8]) -> &mut tokio_test::io::Builder {
    let len = body.len();
    self
      .read(format!("\r\ncontent-length: {len}\r\n\r\n").as_bytes())
      .read(body)
  }
}

static IO_SINK: LazyLock<Mutex<Sink>> = LazyLock::new(|| Mutex::new(tokio::io::sink()));

pub async fn sink_read_stream(mock: &mut tokio_test::io::Mock) {
  tokio::io::sink();

  let mut sink = IO_SINK.lock().await;

  tokio::io::copy(mock, &mut *sink)
    .await
    .expect("To pipe read stream into sink");
}
