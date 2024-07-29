mod head;
mod headers;
mod parser;
mod request;
mod token;

pub use head::*;
pub use headers::*;
pub use parser::*;
pub use request::*;
pub use token::*;

type VecOffset = (usize, usize);

use anyhow::anyhow;
use std::net::SocketAddr;
use tracing::{debug, error};

use tokio::{
  io::{AsyncReadExt},
  net::TcpStream,
};

pub async fn handle_request(
  socket: (&mut TcpStream, &mut SocketAddr),
) -> Result<Request, anyhow::Error> {
  let (stream, _) = socket;

  let mut _req: Result<RequestParser, anyhow::Error> = {
    let mut buf = Some(Vec::new());
    let mut state = ParserState::Start { read_until: None };

    loop {
      let mut taken = buf.take().expect("Taken None Buf");
      let prev_until = state.read_until().to_owned();
      let read = stream.read_buf(&mut taken).await?;

      let buf_len = taken.len();

      if buf_len == 0 && read == 0 {
        break Err(anyhow!("No new Reads"));
      } else if buf_len == 0 {
        continue;
      }

      let parse = RequestParser::parse_request(taken, state)?;

      let (new_buf, new_state) = match parse {
        RequestParseResponse::Incomplete(state) => state,
        RequestParseResponse::Success(parser) => break Ok(parser),
      };

      debug!("Incomplete State: {:#?}", &new_state);

      match new_state {
        ParserState::Start { .. } => (),
        ParserState::Head { read_until, .. }
        | ParserState::Headers { read_until, .. }
        | ParserState::Body { read_until, .. } => {
          if prev_until == read_until {
            error!("Parser returned incomplete twice at : {read_until}");
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

  let req = _req?;

  let unsafe_buf = unsafe { String::from_utf8_unchecked(req.buf.clone()) };
  debug!("[REQ] RAW: {}", unsafe_buf);
  debug!("[REQ] RESULT: {:?}", req);
  drop(unsafe_buf);

  Ok(req.into_request())
}
