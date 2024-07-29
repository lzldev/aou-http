use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddrV4;
use std::sync::Arc;

use matchit::MatchError;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::JsFunction;
use napi_derive::napi;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::error;

use crate::methods::HttpMethod;
use crate::request::{self, Request};
use crate::response::AouResponse;
use crate::route::AouRoute;

#[napi]
pub struct AouInstance {
  _options: AouOptions,
  _router: Arc<matchit::Router<AouRoute<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>>,
  _sender: broadcast::Sender<()>,
}

#[napi]
pub struct AouServer {
  router: matchit::Router<AouRoute<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>,
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

  #[napi(ts_args_type = "route:string,: handler:(request: AouRequest) => Promise<AouResponse>")]
  pub fn post(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::POST, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:string,: handler:(request: AouRequest) => Promise<AouResponse>")]
  pub fn get(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::GET, handler);
    Ok(())
  }

  fn insert_route(&mut self, route: String, method: HttpMethod, function: JsFunction) {
    let handler: ThreadsafeFunction<Request, ErrorStrategy::Fatal> = function
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    let entry = self.router.at_mut(route.as_str());

    if let Err(MatchError::NotFound) = entry {
      let mut new_route = AouRoute::<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>::default();
      new_route.set_method(method, handler);

      self.router.insert(route.as_str(), new_route).unwrap();

      return;
    }

    let entry = entry.unwrap().value;
    entry.set_method(method, handler)
  }

  #[napi]
  pub async fn listen(&self, host: String, port: u32) -> AouInstance {
    let handlers = Arc::new(self.router.clone());
    let handlers2 = handlers.clone();

    let (sender, _receiver) = broadcast::channel::<()>(1024);

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
        let (mut stream, mut addr) = listener.accept().await.expect("Failed to accept socket");

        let handlers = handlers.clone();
        tokio::spawn(async move {
          let req = request::handle_request((&mut stream, &mut addr)).await?;

          let route = match handlers.as_ref().at(req.path()) {
            Ok(h) => h,
            Err(err) => {
              error!("Couldn't find the handler -> {err}");
              return Err(anyhow::anyhow!("Route not found"));
            }
          };

          let method = HttpMethod::from_str(req.method()).expect("Method not supported"); // TODO : Return actual method not allowed response

          let handler: Option<&ThreadsafeFunction<Request, ErrorStrategy::Fatal>> = {
            match (route.value.get_method(method), route.value.get_all()) {
              (Some(r), _) => Some(r),
              (None, Some(r)) => Some(r),
              (None, None) => None,
            }
          };

          if let None = handler {
            todo!("404");
          }

          let handler = handler.unwrap();

          let r = handler.call_async::<Promise<AouResponse>>(req).await?;
          let res: AouResponse = r.await?;

          res.write_response(HashMap::new(), &mut stream).await?;
          stream.flush().await?;

          Ok::<(), anyhow::Error>(())
        });
      }
    });

    AouInstance {
      _router: handlers2,
      _options: self.options,
      _sender: sender,
    }
  }
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct AouOptions {
  pub json: Option<bool>,
}
