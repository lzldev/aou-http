use std::collections::HashMap;

use serde::Deserialize;

use crate::response::Response;

#[napi(object, js_name = "AouError")]
#[derive(Debug, Deserialize)]
pub struct AouError {
  pub status: Option<u32>,
  #[napi(ts_type = "Record<string,string>")]
  pub status_message: Option<String>,
  pub headers: Option<HashMap<String, String>>,
  pub body: serde_json::Value, //TODO: Make this something else
}

impl Into<Response> for AouError {
  fn into(self) -> Response {
    Response {
      status: self.status.or(Some(400)),
      body: self.body,
      headers: self.headers,
      status_message: self.status_message,
      ..Default::default()
    }
  }
}
