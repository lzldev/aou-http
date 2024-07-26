#[macro_use]
extern crate napi;
extern crate napi_derive;

pub mod request;
pub mod utils;

use std::net::SocketAddr;
use tracing::{debug, error, info};

use tokio::{io::AsyncWriteExt, net::TcpListener, time::Instant};
use tracing_subscriber::EnvFilter;

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
    .unwrap_or("0.0.0.0:7070".into());

  let addr = host.parse::<SocketAddr>().expect("Invalid Address");
  let socket = TcpListener::bind(addr).await.expect("Couldn't bind server");
  let mut req_n: usize = 0;

  println!("Server Listening in : {addr}");
  info!("Starting Server on port : {addr}");

  loop {
    let (mut stream, mut addr) = socket.accept().await.expect("Socket failed to accept");

    tokio::spawn(async move {
      let start = tokio::time::Instant::now();
      let req = request::handle_request((&mut stream, &mut addr)).await;

      let req = match req {
        Ok(req) => req,
        Err(err) => {
          error!("REQUEST ERROR:{err}");
          return Err(err);
        }
      };

      // debug!("Request {req:?}");

      let body_buf = format!("\nHello World\n{:#?}", Instant::now());
      let body_length = body_buf.len() + 4;
      let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
        body_length, body_buf
      );

      let _ = stream.write_all(response.as_bytes()).await?;

      // stream.flush().await?;
      stream.shutdown().await?;

      info!("r: {:?}", start.elapsed());

      Ok::<(), anyhow::Error>(())
    });

    req_n = req_n.wrapping_add(1);
  }
}
