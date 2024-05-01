use std::time::Instant;
use std::{collections::HashMap, net::SocketAddrV4};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ErrorStrategy;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi::CallContext;
use napi::JsFunction;
use napi::JsUndefined;
use tokio::{net::TcpListener, sync::oneshot};

#[napi]
pub struct AouServer {
  options: AouOptions,
  handlers: HashMap<String, (Method, ThreadsafeFunction<(), ErrorStrategy::CalleeHandled>)>,
  sender: Option<oneshot::Sender<()>>,
}

#[napi]
impl AouServer {
  #[napi(constructor)]
  pub fn new(options: Option<AouOptions>) -> Self {
    let options = options.unwrap_or_default();

    AouServer {
      options,
      handlers: HashMap::new(),
      sender: None,
    }
  }

  #[napi]
  pub fn is_running(&self) -> bool {
    self.sender.is_some()
  }

  fn make_handler_safe(func: JsFunction) -> ThreadsafeFunction<(), ErrorStrategy::CalleeHandled> {
    func
      .create_threadsafe_function(0, |_| Ok(vec![()]))
      .expect("Couldn't Register Handler")
  }

  #[napi]
  pub fn get(&mut self, route: String, handler: JsFunction) {
    self
      .handlers
      .insert(route, (Method::GET, AouServer::make_handler_safe(handler)));
  }

  #[napi]
  pub fn post(&mut self, route: String, handler: JsFunction) {
    self
      .handlers
      .insert(route, (Method::POST, AouServer::make_handler_safe(handler)));
  }

  #[napi]
  pub async fn fake_listen(&self) -> () {
    let fake_route = "/".to_owned();
    let handler = self.handlers.get(&fake_route);

    if handler.is_none() {
      return ();
    }
    let (route, function) = handler.unwrap();

    eprintln!("will this print");
    let start = Instant::now();
    let _: Result<(), _> = function.call_async(Ok(())).await;
    eprintln!("End: {:?}", start.elapsed());

    ()
  }

  // pub async fn listen(&self, host: String, port: usize) {
  //   let addr = format!("{host}:{port}");

  //   let (sender, receiver) = oneshot::channel::<()>();

  //   let listener = TcpListener::bind(
  //     addr
  //       .as_str()
  //       .parse::<SocketAddrV4>()
  //       .expect("Invalid Server Address"),
  //   )
  //   .await
  //   .expect("Couldn't establish tcp connection");

  //   tokio::spawn(async move {
  //     let mut receiver = receiver;

  //     loop {
  //       tokio::select! {
  //         Ok((socket,addr)) = listener.accept() => {

  //         }
  //           _ = &mut receiver =>{
  //             println!("Server killed");
  //           }
  //       }
  //     }
  //   });
  // }
}

#[napi(object)]
#[derive(Debug, Default)]
pub struct AouOptions {
  pub json: Option<bool>,
}

enum Method {
  GET,
  POST,
}
