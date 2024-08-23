mod head;
mod headers;
mod method;
mod options;
mod parser;
mod request;

pub use head::*;
pub use headers::*;
pub use method::*;
pub use options::*;
pub use parser::*;
pub use request::*;

type VecOffset = (usize, usize);

use anyhow::anyhow;
use tracing::error;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[derive(thiserror::Error, Debug)]
pub enum HandleRequestError {
  #[error("Connection Timeout")]
  Timeout,
  #[error("End of File")]
  EOF,
  #[error(transparent)]
  Invalid(#[from] anyhow::Error),
}

impl From<ParserStateError> for HandleRequestError {
  fn from(err: ParserStateError) -> Self {
    HandleRequestError::Invalid(anyhow!(err))
  }
}

pub async fn handle_request<T>(stream: &mut T) -> Result<Request, HandleRequestError>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  let mut _result: Result<ParserResult, HandleRequestError> = {
    let mut iter = 0;
    let mut buf = Some(Vec::new());
    let mut state = ParserState::Start { read_until: None };

    loop {
      iter += 1;
      let mut taken = buf.take().expect("Taken None Buf");
      let prev_until = state.read_until().to_owned();

      /*
      TODO: Move into Config
        5   = ReadTimeout
        200 = KeepAliveTimeout
      */
      let sleep_ms = if iter == 0 { 5 } else { 200 };

      let read = tokio::select! {
        read_buf = stream.read_buf(&mut taken) => {
          match read_buf {
            Ok(read) => read,
            Err(err) => break Err(HandleRequestError::Invalid(anyhow!("Error reading buffer {err}")))
          }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)) => {
          if state.is_body() {
            break Ok(state.into_parser_result(taken)?);
          }
          break Err(HandleRequestError::Invalid(anyhow!("Connection timeout")))
        }
      };

      let buf_len = taken.len();

      if read == 0 && !state.is_body() {
        break Err(HandleRequestError::EOF);
      } else if read == 0 && state.is_body() {
        break Ok(state.into_parser_result(taken)?);
      } else if buf_len == 0 && read == 0 {
        break Err(HandleRequestError::Invalid(anyhow!("No new Reads")));
      } else if buf_len == 0 {
        continue;
      }

      let (new_buf, new_state) = match RequestParser::parse_request(taken, state) {
        ParserStatus::Incomplete(state) => state,
        ParserStatus::Success(parser) => break Ok(parser),
        ParserStatus::Invalid(reason) => break Err(HandleRequestError::Invalid(anyhow!(reason))),
      };

      match new_state {
        ParserState::Start { .. } => (),
        ParserState::Head { read_until, .. }
        | ParserState::Headers { read_until, .. }
        | ParserState::Body { read_until, .. } => {
          if prev_until != 0 && prev_until == read_until {
            error!("Parser returned incomplete twice at : {read_until} | iter : {iter}");
            break Err(HandleRequestError::Invalid(anyhow!("Incomplete Twice")));
          }
        }
      };

      buf = Some(new_buf);
      state = new_state;
    }
  };

  let result = _result?;

  Ok(result.into_request())
}

#[cfg(test)]
mod unit_tests {

  use crate::{
    request::{self, Connection},
    utils::test::BuilderWithBody,
  };

  #[tokio::test]
  async fn incomplete_once() {
    let mut mock = tokio_test::io::Builder::new()
      .read(b"GET /server_error123 HTTP/1.1\r\nHost: localhost:7070\r\nUser-Agent:")
      .read(b" curl/8.2.1\r\nAccept: */*\r\n\r\n")
      .read(b"")
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(
      r.is_ok(),
      "Request should return true even though it erroed once {r:?}",
    )
  }

  #[tokio::test]
  async fn should_timeout() {
    let mut mock = tokio_test::io::Builder::new()
      .read(b"GET /server_error123 HTTP/1.1\r\nHost: localhost:7070\r\nUser-Agent:")
      .wait(tokio::time::Duration::from_millis(1000))
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(r.is_err(), "Request should timeout")
  }

  #[tokio::test]
  async fn complete_and_zero() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.1\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4\r\nhost: 192.168.3.29:7070\r\n\r\n",
      )
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(
      r.is_ok(),
      "Request should return true even though it erroed once {r:?}",
    );

    let mut r = r.unwrap();

    assert_eq!(r.body().len(), 0, "Request body should be of length 0")
  }

  #[tokio::test]
  async fn incomplete_and_zero() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.1\r\nHost: 192.168.0.1\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4",
      )
      .read(b"")
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(
      r.is_err(),
      "Request should error after Writing \"\" in a invalid state",
    );
  }

  #[tokio::test]
  async fn multiple_valid_header_states() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.1\r\nHost: 192.168.3.29:7070\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4",
      )
      .read(b"\r\nConnection: close")
      .with_body(b"{\"valid\":\"json\"}")
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(
      r.is_ok(),
      "Long request with multiple valid states should be ok",
    );

    let r = r.unwrap();

    assert_eq!(
      r.get_connection(),
      &Connection::Close,
      "Connection header should be close"
    )
  }

  #[tokio::test]
  async fn headers_cache_happy_path() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.1\r\nHost: 192.168.3.29:7070\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4",
      )
      .with_body(b"{\"valid\":\"json\"}")
      .read(b"")
      .build();

    let r = request::handle_request(&mut mock).await;

    assert!(
      r.is_ok(),
      "Long request with multiple valid states should be ok",
    );
  }

  #[tokio::test]
  async fn invalid_headers_stream() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.0\r\nHost: 192.168.3.29:7070\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4\r\n\r\n{\"valid\":\"json\"",
      )
      .build();

    let r = request::handle_request(&mut mock).await;

    let mut sink = tokio::io::sink();
    tokio::io::copy(&mut mock, &mut sink).await.unwrap();

    assert!(r.is_err(), "Request should error with invalid Header",);
  }

  #[tokio::test]
  async fn multiple_requests_with_content_length() {
    let mut mock = tokio_test::io::Builder::new()
      .read(
        b"GET /json HTTP/1.1\r\nHost: 192.168.3.29:7070\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4\r\n",
      )
      .read(b"Content-Length: 16\r\nConnection: close\r\n")
      .read(b"\r\n{\"valid\":\"json\"")
      .read(b"}")
      .read(b"")
      // .wait(Duration::from_millis(1000))
      .read(
        b"GET /json HTTP/1.1\r\nHost: 192.168.3.29:7070\r\naccept: */*\r\naccept-encoding: gzip, compress, deflate, br\r\nuser-agent: oha/1.4.4\r\n",
      )
      .read(b"Content-Length: 16\r\nConnection: close\r\n")
      .read(b"\r\n{\"valid\":\"json\"")
      .read(b"}")
      .read(b"")
      .build();

    let r = request::handle_request(&mut mock).await;
    assert!(r.is_ok(), "First Request should be parsed correctly");

    let r = request::handle_request(&mut mock).await;
    assert!(r.is_ok(), "Second Request should be parsed correctly");
  }
}
