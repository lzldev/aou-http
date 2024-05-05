use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};

#[tokio::test]
async fn reading_buffers() {
  let mut file = tokio::fs::File::open("fixtures/hello_world.txt")
    .await
    .unwrap();
  let mut buf = BytesMut::new();

  loop {
    let red = match file.read_buf(&mut buf).await {
      Ok(0) => {
        dbg!("End");
        break;
      }
      Ok(n) => {
        dbg!("reading..... {:#?} {buf:#?}", n);
        continue;
      }
      Err(err) => {
        dbg!(err);
        panic!("oh nyoo")
      }
    };
  }

  dbg!(buf);
}

#[tokio::test]
async fn streams() {
  let addr = "0.0.0.0:0";

  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Socket open");

  let addr = listener.local_addr().expect("to receive address");

  let connection = tokio::net::TcpStream::connect(addr)
    .await
    .expect("Socket connect");

  let h1 = tokio::spawn(async move {
    let listener = listener;

    let (mut stream, _addr) = listener.accept().await?;
    let mut buf = BytesMut::new();

    loop {
      match stream.read_buf(&mut buf).await {
        Ok(0) => {
          dbg!("0 bytes");
          break;
        }
        Ok(n) => {
          dbg!("read", n);
          continue;
        }
        Err(e) => {
          dbg!("listen error {:?}", e);
          break;
        }
      }
    }

    dbg!("finished Reading", buf);

    Ok::<(), anyhow::Error>(())
  });

  let h2 = tokio::spawn(async move {
    let mut connection = connection;

    connection
      .ready(Interest::WRITABLE)
      .await
      .expect("Connection to be ready.");

    connection.write_all(b"Hello World").await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    Ok::<(), anyhow::Error>(())
  });

  let _ = h1.await;
  let _ = h2.await;
}

#[tokio::test]
async fn write_to_socket() {
  let addr = "0.0.0.0:3000";

  let connection = tokio::net::TcpStream::connect(addr)
    .await
    .expect("Socket connect");

  let (read, write) = connection.into_split();

  let (start_tx, start_rx) = tokio::sync::oneshot::channel::<()>();

  let h1 = tokio::spawn(async move {
    let mut write = write;

    write
      .ready(Interest::WRITABLE)
      .await
      .expect("Connection to be ready.");

    write
      .write_all(b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nx-hello: hello there\r\n\r\n")
      .await?;

    start_tx.send(()).unwrap();

    Ok::<(), anyhow::Error>(())
  });

  let h2 = tokio::spawn(async move {
    let mut read = read;
    let _ = start_rx.await.unwrap();

    read
      .ready(Interest::READABLE)
      .await
      .expect("Connection to be ready.");

    let mut buf = BytesMut::new();

    loop {
      tokio::select! {
        r = read.read_buf(&mut buf) => {
          match r {
            Ok(0)=>{
              break;

            },
            Ok(n) => {
              continue;

            },
            Err(err) => {
              panic!("Error reading socket");
            }
          }
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {
            break;
        }
      }
    }

    let res = unsafe { String::from_utf8_unchecked(buf.into()) };

    // let res = res.replace("\r\n", "\n");

    println!("\n\n{res}");

    Ok::<(), anyhow::Error>(())
  });

  let _ = h1.await;
  let _ = h2.await;
}
