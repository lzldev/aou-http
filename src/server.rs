use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddrV4;
use std::sync::Arc;

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
use crate::response::Response;
use crate::route::Route;

#[napi]
pub struct AouInstance {
  pub ip: String,
  pub port: u32,
  _options: AouOptions,
  _router: Arc<matchit::Router<Route<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>>,
  _sender: broadcast::Sender<()>,
}

#[napi]
pub struct AouServer {
  router: matchit::Router<Route<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>,
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

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn get(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::GET, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn head(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::HEAD, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn post(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::POST, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn put(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::PUT, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn delete(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::DELETE, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn connect(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::CONNECT, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn options(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::OPTIONS, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn trace(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::TRACE, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn patch(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_route(route, HttpMethod::PATCH, handler);
    Ok(())
  }

  #[napi(ts_args_type = "route:void,handler:void")]
  pub fn all(&mut self, route: String, handler: JsFunction) -> Result<()> {
    self.insert_all(route, handler);
    Ok(())
  }

  fn insert_all(&mut self, route: String, function: JsFunction) {
    let handler: ThreadsafeFunction<Request, ErrorStrategy::Fatal> = function
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    let mut new_route = Route::<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>::default();
    new_route.set_all(handler.clone());

    match self.router.insert(route.as_str(), new_route) {
      Ok(_) => (),
      Err(_) => {
        let entry = self.router.at_mut(route.as_str()).unwrap();
        if entry.value.has_all() {
          panic!("Tried to overwrite ALL at {}", route.as_str());
        }
        entry.value.set_all(handler)
      }
    }
  }

  fn insert_route(&mut self, route: String, method: HttpMethod, function: JsFunction) {
    let handler: ThreadsafeFunction<Request, ErrorStrategy::Fatal> = function
      .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
      .unwrap();

    let mut new_route = Route::<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>::default();
    new_route.set_method(method, handler.clone());

    match self.router.insert(route.as_str(), new_route) {
      Ok(_) => (),
      Err(_) => {
        let entry = self.router.at_mut(route.as_str()).unwrap();
        if entry.value.has_method(method) {
          panic!(
            "Tried to overwrite method {} at {}",
            method.to_str(),
            route.as_str()
          );
        }
        entry.value.set_method(method, handler)
      }
    };
  }

  #[napi]
  pub async fn listen(&self, host: String, port: u32) -> AouInstance {
    let handlers = Arc::new(self.router.clone());
    let handlers_cpy = handlers.clone();

    let (sender, _receiver) = broadcast::channel::<()>(1024);

    let addr = format!("{host}:{port}")
      .parse::<SocketAddrV4>()
      .expect("Invalid Server Address");

    let listener = TcpListener::bind(&addr)
      .await
      .expect("Couldn't establish tcp connection");

    tokio::spawn(async move {
      let handlers = handlers;

      loop {
        let (mut stream, mut addr) = listener.accept().await.expect("Failed to accept socket");
        let handlers = handlers.clone();

        tokio::spawn(async move {
          let mut req = request::handle_request((&mut stream, &mut addr)).await?;

          let hc = req.path().to_owned();
          let (l, _) = hc.split_once("?").unwrap_or((&hc, ""));

          let route = match handlers.as_ref().at(l) {
            Ok(h) => h,
            Err(err) => {
              error!("Couldn't find the handler {hc} -> {err}");
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

          req.params = route
            .params
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

          let r = handler.call_async::<Promise<Response>>(req).await?;
          let res: Response = r.await?;

          res.write_to_stream(HashMap::new(), &mut stream).await?;
          stream.flush().await?;

          Ok::<(), anyhow::Error>(())
        });
      }
    });

    AouInstance {
      ip: addr.ip().to_string(),
      port: addr.port() as u32,
      _router: handlers_cpy,
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
