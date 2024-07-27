use std::any::{type_name, type_name_of_val, Any};
use std::fmt::{Debug, Display};
use std::net::SocketAddrV4;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{bindgen_prelude::*, JsObject, NapiValue};
use napi::{JsFunction, NapiRaw};
use napi_derive::napi;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::error;

use crate::request::{self, Request};
use crate::response::AouResponse;

#[napi]
pub struct AouInstance {
  options: AouOptions,
  router: Arc<matchit::Router<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>,
  sender: broadcast::Sender<()>,
}

#[napi]
pub struct AouServer {
  router: matchit::Router<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>,
  options: AouOptions,
}

#[napi]
impl AouServer {
  #[napi(constructor)]
  pub fn new(options: Option<AouOptions>) -> Self {
    let options = options.unwrap_or_default();

    AouServer {
      router: matchit::Router::new(),
      options,
    }
  }

  #[napi(ts_args_type = "route:string,: handler:( request: AouRequest) => Promise<AouResponse>")]
  pub fn get(&mut self, route: String, handler: JsFunction) -> Result<()> {
    let h: ThreadsafeFunction<Request, ErrorStrategy::Fatal> = handler
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    self
      .router
      .insert(route, h)
      .expect("failed to insert handler");

    Ok(())
  }

  #[napi]
  pub async fn listen(&self, host: String, port: u32) -> AouInstance {
    let handlers = Arc::new(self.router.clone());
    let handlers2 = handlers.clone();

    let (sender, receiver) = broadcast::channel::<()>(1024);

    let addr = format!("{host}:{port}");

    let listener = TcpListener::bind(
      addr
        .as_str()
        .parse::<SocketAddrV4>()
        .expect("Invalid Server Address"),
    )
    .await
    .expect("Couldn't establish tcp connection");

    tokio::spawn(async move {
      let handlers = handlers.clone();
      loop {
        let (mut stream, mut addr) = listener.accept().await?;

        let handlers = handlers.clone();
        tokio::spawn(async move {
          let req = request::handle_request((&mut stream, &mut addr)).await?;

          let h = match handlers.as_ref().at(req.path()) {
            Ok(h) => h,
            Err(err) => {
              error!("Didn't find the handler -> {err}");
              return Err(anyhow::anyhow!("Route not found"));
            }
          };

          let r = h.value.call_async::<Promise<AouResponse>>(req).await?;

          eprintln!("PROMISE TYPE {}", type_name_of_val(&r));

          let r: AouResponse = r.await?;

          eprintln!("RETURN TYPE {}", type_name_of_val(&r));
          eprintln!("VALUE {r:#?}");

          // eprintln!("RETURN {r:#?}");

          let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

          let body_buf = json!({
            "message":"Hello World",
            "instant":ms,
            "data":r.data
          })
          .to_string();

          let content_length = body_buf.len();

          let response = format!(
            "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            r.status, content_length, body_buf
          );
          let _ = stream.write_all(response.as_bytes()).await?;
          stream.flush().await?;

          Ok::<(), anyhow::Error>(())
        });
      }

      Ok::<(), anyhow::Error>(())
    });

    AouInstance {
      router: handlers2,
      options: self.options,
      sender,
    }
  }
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct AouOptions {
  pub json: Option<bool>,
}
