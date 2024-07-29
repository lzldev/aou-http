use std::collections::{HashMap, HashSet};

use serde_json::json;
use tokio::{io::AsyncWriteExt, net::TcpStream};

#[napi(object)]
#[derive(Debug)]
pub struct AouResponse {
  pub status: Option<u32>,
  #[napi(ts_type = "Record<String,String>")]
  pub status_message: Option<String>,
  pub headers: Option<serde_json::Value>,
  pub body: Option<serde_json::Value>,
}

impl Default for AouResponse {
  fn default() -> Self {
    Self {
      status: None,
      status_message: None,
      headers: Default::default(),
      body: Default::default(),
    }
  }
}

impl AouResponse {
  pub async fn write_response(
    &self,
    static_headers: HashMap<String, String>,
    stream: &mut TcpStream,
  ) -> Result<(), anyhow::Error> {
    let status = self.status.unwrap_or(200);
    let status_message = self
      .status_message
      .as_ref()
      .map(|f| f.as_str())
      .or(AouResponse::status_message(status))
      .unwrap_or("");

    let h_holder = self
      .headers
      .as_ref()
      .and_then(|value| {
        if value.is_object() {
          Some(value.to_owned())
        } else {
          Some(json!({}))
        }
      })
      .unwrap_or(json!({}));

    let headers = h_holder.as_object().unwrap();

    let body_buf = self
      .body
      .as_ref()
      .map(|f| f.to_string())
      .unwrap_or(String::new());

    let headers: String = {
      let mut r = String::new();
      let mut set = HashSet::<&String>::new();

      static_headers
        .iter()
        .for_each(|(key, value)| match headers.get(key) {
          Some(h) => {
            set.insert(key);
            r.push_str(format!("{key}: {} \r\n", h.to_string()).as_str())
          }
          None => r.push_str(format!("{key}: {value} \r\n").as_str()),
        });

      if !self.headers.is_none() {
        headers.iter().for_each(|(key, value)| {
          if set.contains(key) || key == "Content-Length" {
            return;
          }
          r.push_str(format!("{key}: {value} \r\n").as_str());
        });
      }

      r.push_str(format!("Content-Length: {} \r\n", body_buf.len()).as_str());
      r.into()
    };

    stream
      .write_all(
        format!("HTTP/1.1 {status} {status_message}\r\n{headers}\r\n{body_buf}").as_bytes(),
      )
      .await?;

    Ok(())
  }

  fn status_message<'r>(status_code: u32) -> Option<&'r str> {
    match status_code {
      100 => Some("Continue"),
      101 => Some("Switching Protocol"),
      102 => Some("Processing"),
      103 => Some("Early Hints"),

      200 => Some("OK"),
      201 => Some("Created"),
      202 => Some("Accepted"),
      203 => Some("Non-Authoritative Information"),
      204 => Some("No Content"),
      205 => Some("Reset Content"),
      206 => Some("Partial Content"),
      207 => Some("Multi-Status"),
      208 => Some("Already Reported"),
      226 => Some("IM USED"),

      300 => Some("Multiple Choices"),
      301 => Some("Moved Permanently"),
      302 => Some("Found"),
      303 => Some("See Other"),
      304 => Some("Not Modified"),
      307 => Some("Temporary Redirect"),
      308 => Some("Permanent Redirect"),

      400 => Some("Bad Request"),
      401 => Some("Unauthorized"),
      402 => Some("Payment Required"),
      403 => Some("Forbidden"),
      404 => Some("Not Found"),
      405 => Some("Method Not Allowed"),
      406 => Some("Not Acceptable"),
      407 => Some("Proxy Authentication Required"),
      408 => Some("Request Timeout"),
      409 => Some("Conflict"),
      410 => Some("Gone"),
      411 => Some("Length Required"),
      412 => Some("Precondition Failed"),
      413 => Some("Content Too Large"),
      414 => Some("URI Too Long"),
      415 => Some("Unsupported Media Type"),
      416 => Some("Range Not Satisfiable"),
      417 => Some("Expectation Failed"),
      418 => Some("I'm a teapot"),
      421 => Some("Misdirected Request"),
      422 => Some("Unprocessable Content"),
      423 => Some("Locked"),
      424 => Some("Failed Dependency"),
      425 => Some("Too Early"),
      426 => Some("Upgrade Required"),
      428 => Some("Precondition Required"),
      429 => Some("Too Many Requests"),
      431 => Some("Request Header Fields Too Large"),
      451 => Some("Unavailable For Legal Reasons"),

      500 => Some("Internal Server Error"),
      501 => Some("Not Implemented"),
      502 => Some("Bad Gateway"),
      503 => Some("Service Unavailable"),
      504 => Some("Gateway Timeout"),
      505 => Some("HTTP Version Not Supported"),
      506 => Some("Variant Also Negotiates"),
      507 => Some("Insufficient Storage"),
      508 => Some("Loop Detected"),
      510 => Some("Not Extended"),
      511 => Some("Network Authentication Required"),

      _ => None,
    }
  }
}
