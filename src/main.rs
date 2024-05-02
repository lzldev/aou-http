use anyhow::anyhow;
use std::net::SocketAddr;
use std::time::Duration;

mod request;

use request::{Request, RequestParser};
use tokio::time::Instant;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream},
};

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

  let mut buf = Vec::new();

  let mut _req: Option<RequestParser> = {
    //TODO:Remove into Parse_frame
    loop {
      let buf_len = buf.len();
      let n = tokio::select! {
        n = stream.read_buf(&mut buf) => {
          if let Err(n) = n {
            return Err(n.into());
          }

          n.unwrap()
        },
        _ = tokio::time::sleep(Duration::from_millis(10)) => return Err(anyhow!("Timedout"))
      };

      if buf_len == 0 && n == 0 {
        break None;
      }
      if buf_len > 0 {
        if let Ok(req) = RequestParser::parse_request(&mut buf) {
          break Some(req);
        }

        if n == 0 {
          break None;
        }
      }
    }
  };

  let old = _req.ok_or(anyhow!("Can't unwrap _req data"))?;
  let mut _req = old.into_request();

  let f = _req.buf.get_mut(0).unwrap();
  *f = b'R';
  drop(f);

  dbg!(String::from_utf8_lossy(&_req.buf[..10]));
  dbg!(String::from_utf8_lossy(&_req.head.method));

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
