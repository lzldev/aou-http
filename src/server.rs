use std::any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddrV4;
use std::sync::Arc;

use anyhow::anyhow;
use matchit::Match;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::JsFunction;
use napi_derive::napi;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::debug;
use tracing::error;
use tracing_subscriber::EnvFilter;

use crate::error::AouError;
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

  #[napi]
  pub async fn listen(&self, host: String, port: u32) -> AouInstance {
    let subscriber = tracing_subscriber::fmt()
      .compact()
      .with_env_filter(EnvFilter::from_default_env())
      .with_line_number(true)
      .with_file(true)
      .with_target(false)
      .finish();

    tracing::subscriber::set_global_default(subscriber)
      .unwrap_or_else(|_| error!("Tried to register tracing subscriber twice"));

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
        let (mut stream, mut _addr) = listener.accept().await.expect("Failed to accept socket");
        let handlers = handlers.clone();

        tokio::spawn(async move {
          let mut req = match request::handle_request(&mut stream).await {
            Ok(req) => req,
            Err(err) => {
              error!("Error Handling Request {err}");
              return Err(err);
            }
          };

          let method = HttpMethod::from_str(req.method()).expect("Method not supported"); // TODO : Return actual method not allowed response
                                                                                          //
          let path = req.path().to_owned();
          let (path, _query) = path.split_once('?').unwrap_or((&path, ""));

          let (route, handler) = match AouServer::match_route(handlers.as_ref(), path, method) {
            Some(_match) => _match,
            None => {
              debug!("Route not found {path}");
              let res = Response {
                status: Some(404),
                ..Default::default()
              };

              res.write_to_stream(&mut stream, &HashMap::new()).await?; //TODO: static headers.
              stream.flush().await?;

              return Err(anyhow!("Route Not Found"));
            }
          };

          req.params = route
            .params
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

          let r = handler.call_async::<Promise<Response>>(req).await?;

          let res: Response = match r.await {
            Ok(r) => r,
            Err(err) => {
              let err: napi::Error = err;

              let is_aou_error = err
                .reason
                .starts_with("AouError: ")
                .then(|| &err.reason[10..]);

              match is_aou_error {
                Some(reason) => {
                  debug!("AouMessage: {reason}");
                  let err = serde_json::from_str::<AouError>(reason).unwrap();
                  error!("AouError: {err:?}");

                  <AouError as Into<Response>>::into(err)
                    .write_to_stream(&mut stream, &HashMap::new())
                    .await?;
                }
                None => {
                  //TODO: Return Error on request based on config.
                  error!("Unknown Error: {err:?} {}", any::type_name_of_val(&err));
                  Response {
                    status: Some(500),
                    body: serde_json::Value::String(err.reason),
                    status_message: None,
                    headers: None,
                  }
                  .write_to_stream(&mut stream, &HashMap::new())
                  .await?;
                }
              };

              return Err(anyhow!("505"));
            }
          };

          res.write_to_stream(&mut stream, &HashMap::new()).await?;
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

  fn match_route<'r, 'f>(
    router: &'r matchit::Router<Route<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>,
    route: &'r str,
    method: HttpMethod,
  ) -> Option<(
    Match<'r, 'r, &'r Route<ThreadsafeFunction<Request, ErrorStrategy::Fatal>>>,
    &'f ThreadsafeFunction<Request, ErrorStrategy::Fatal>,
  )>
  where
    'r: 'f,
  {
    let route = match router.at(route) {
      Ok(h) => h,
      Err(_) => {
        return None;
      }
    };

    match (route.value.get_method(method), route.value.get_all()) {
      (Some(r), _) => Some((route, r)),
      (None, Some(r)) => Some((route, r)),
      (None, None) => None,
    }
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
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct AouOptions {
  pub tracing: Option<bool>,
}
