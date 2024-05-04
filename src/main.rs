use anyhow::anyhow;
use std::net::SocketAddr;
use std::time::Duration;

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
  let port = "127.0.0.1:7070".parse::<SocketAddr>().unwrap();
  let socket = TcpListener::bind(port).await.expect("Couldn't bind server");

  println!("Server Listening in : {port}");

  loop {
    let con = socket.accept().await.expect("Socket failed to accept");

    tokio::spawn(async move {
      let n = process_request(con).await;

      if let Err(e) = n {
        dbg!(e);
      }

      ()
    });
  }
}

async fn process_request(socket: (TcpStream, SocketAddr)) -> Result<(), anyhow::Error> {
  let (mut stream, _) = socket;

  let mut _req: Option<RequestParser> = {
    let mut buf = Some(Vec::new());
    let mut state = ParserState::New;

    //TODO:Remove into Parse_frame
    loop {
      let mut taken = buf.take().expect("Taken None Buf");
      let n = tokio::select! {
        n = stream.read_buf(&mut taken) => {
          if let Err(n) = n {
            return Err(n.into());
          }

          n.unwrap()
        },
        // _ = tokio::time::sleep(Duration::from_millis(10)) => return Err(anyhow!("Timedout"))
      };
      let buf_len = taken.len();

      if buf_len == 0 && n == 0 {
        break None;
      }

      if buf_len > 0 {
        match RequestParser::parse_request(taken, state) {
          Ok(res) => match res {
            request::ParseResponse::Success(parser) => break Some(parser),
            request::ParseResponse::Incomplete((b, new_state)) => {
              dbg!("new State _ ", &new_state);
              buf = Some(b);
              state = new_state;

              if n == 0 {
                break None;
              }
            }
          },
          Err(_) => {
            break None;
          }
        };
      }
    }
  };

  let parser = _req.ok_or(anyhow!("Can't unwrap _req data"))?;
  dbg!(parser);

  // let mut _req = old.into_request();

  // let f = _req.buf.get_mut(0).unwrap();
  // *f = b'R';
  // drop(f);

  // dbg!(String::from_utf8_lossy(&_req.buf[..10]));
  // dbg!(String::from_utf8_lossy(&_req.head.method));

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
