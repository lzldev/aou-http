mod head;
mod headers;
mod parser;
mod request;

pub use head::*;
pub use headers::*;
pub use parser::*;
pub use request::*;

type VecOffset = (usize, usize);

use anyhow::anyhow;
use tracing::{debug, error};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub async fn handle_request<T>(stream: &mut T) -> Result<Request, anyhow::Error>
where
  T: AsyncRead + AsyncWrite + Unpin,
{
  let mut _result: Result<ParserResult, anyhow::Error> = {
    let mut iter = 0;
    let mut buf = Some(Vec::new());
    let mut state = ParserState::Start { read_until: None };

    loop {
      iter += 1;
      let mut taken = buf.take().expect("Taken None Buf");
      let prev_until = state.read_until().to_owned();
      let read = tokio::select! {
        read_buf = stream.read_buf(&mut taken) => {
          match read_buf {
            Ok(read) => read,
            Err(err) => break Err(anyhow!("Error reading buffer {err}"))
          }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
          break Err(anyhow!("Connection timeout"))
        }
      };

      let buf_len = taken.len();

      if buf_len == 0 && read == 0 {
        break Err(anyhow!("No new Reads"));
      } else if buf_len == 0 {
        continue;
      }

      let parse = ParserResult::parse_request(taken, state);

      let (new_buf, new_state) = match parse {
        RequestParseResult::Incomplete(state) => state,
        RequestParseResult::Success(parser) => break Ok(parser),
        RequestParseResult::Invalid(reason) => break Err(anyhow!(reason)),
      };

      debug!("Incomplete State: {:#?}", &new_state);

      match new_state {
        ParserState::Start { .. } => (),
        ParserState::Head { read_until, .. }
        | ParserState::Headers { read_until, .. }
        | ParserState::Body { read_until, .. } => {
          if prev_until != 0 && prev_until == read_until {
            error!("Parser returned incomplete twice at : {read_until} | iter : {iter}");
            break Err(anyhow!("Incomplete Twice"));
          }
        }
      };

      buf = Some(new_buf);
      state = new_state;

      if read == 0 {
        debug!("Incomplete && n == 0 ");

        unsafe {
          debug!("{}", String::from_utf8_unchecked(buf.unwrap_unchecked()));
        }

        break Err(anyhow!("incomplete Request"));
      }
    }
  };

  let result = _result?;

  Ok(result.into_request())
}

#[cfg(test)]
mod unit_tests {

  use crate::request;

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
}
