use anyhow::anyhow;
use std::net::SocketAddr;
use tracing::{debug, error, info, info_span};
use tracing_subscriber::EnvFilter;

pub mod request;
pub mod utils;

use tokio::time::Instant;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream},
};

use request::{ParserState, RequestParser};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  let subscriber = tracing_subscriber::fmt()
    .compact()
    .with_env_filter(EnvFilter::from_default_env())
    .with_line_number(true)
    .with_file(true)
    .with_target(false)
    .finish();

  tracing::subscriber::set_global_default(subscriber)
    .expect("Couldn't register tracing default subscriber.");

  let host = std::env::args()
    .skip(1)
    .next()
    .unwrap_or("127.0.0.1:7070".into());

  let addr = host.parse::<SocketAddr>().expect("Invalid Address");
  let socket = TcpListener::bind(addr).await.expect("Couldn't bind server");
  let mut req_n: usize = 0;

  println!("Server Listening in : {addr}");

  info!("Starting Server on port : {addr}");
  debug!("TRUE");

  loop {
    let con = socket.accept().await.expect("Socket failed to accept");

    tokio::spawn(async move {
      let span = info_span!("request");
      let _lock = span.enter();

      let n = process_request(con).await;

      match n {
        Ok(_) => info!("Ok : {req_n}"),
        Err(err) => error!("Err : {req_n}\n{err:?}"),
      }

      ()
    });

    req_n = req_n.wrapping_add(1);
  }
}

async fn process_request(socket: (TcpStream, SocketAddr)) -> Result<(), anyhow::Error> {
  let (mut stream, _) = socket;

  let mut _req: Option<RequestParser> = {
    let mut buf = Some(Vec::new());
    let mut state = ParserState::Start { read_until: None };

    //TODO:Remove into Parse_frame
    loop {
      let mut taken = buf.take().expect("Taken None Buf");
      let prev_until = state.read_until().to_owned();
      let n = stream.read_buf(&mut taken).await?;

      let buf_len = taken.len();

      if buf_len == 0 && n == 0 {
        break None;
      }

      if buf_len > 0 {
        match RequestParser::parse_request(taken, state) {
          Ok(res) => match res {
            request::RequestParseResponse::Success(parser) => break Some(parser),
            request::RequestParseResponse::Incomplete((b, new_state)) => {
              dbg!(String::from_utf8_lossy(&b), n);
              debug!("new State _ {:#?}", &new_state);

              match new_state {
                ParserState::Start { .. } => (),
                ParserState::Head { read_until, .. }
                | ParserState::Headers { read_until, .. }
                | ParserState::Body { read_until, .. } => {
                  if prev_until == read_until {
                    error!("Parser returned incomplete twice at : {read_until}");
                    break None;
                  }
                }
              };

              buf = Some(b);
              state = new_state;

              if n == 0 {
                debug!("Incomplete && n == 0 ");

                unsafe {
                  debug!("{}", String::from_utf8_unchecked(buf.unwrap_unchecked()));
                }

                break None;
              }
            }
          },
          Err(parse_fatal) => {
            dbg!(&parse_fatal);
            error!("parse_request {parse_fatal:#?}");
            break None;
          }
        };
      }
    }
  };

  let req = _req.ok_or(anyhow!("Can't unwrap _req data"))?;
  dbg!("{:}", String::from_utf8_lossy(&req.buf));

  let body_buf = format!("\nHello World\n{:#?}", Instant::now());
  let body_length = body_buf.len();

  let response = format!(
    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length:{}\r\n\r\n{}\r\n\r\n",
    body_length, body_buf
  );

  let _ = stream.write_all(response.as_bytes()).await?;

  stream.flush().await?;

  Ok(())
}
