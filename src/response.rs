use std::collections::{HashMap, HashSet};

use napi::bindgen_prelude::Buffer;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

#[napi(object, js_name = "AouResponse")]
pub struct Response {
  pub status: Option<u32>,
  #[napi(ts_type = "Record<string,string>")]
  pub status_message: Option<String>,
  pub headers: Option<HashMap<String, String>>,
  #[napi(ts_type = "any | null")]
  pub body: serde_json::Value,
  pub buffer: Option<Buffer>,
}

unsafe impl Sync for Response {}
unsafe impl Send for Response {}

impl Default for Response {
  fn default() -> Self {
    Self {
      status: None,
      status_message: None,
      headers: Default::default(),
      body: serde_json::Value::Null,
      buffer: None,
    }
  }
}

impl Response {
  pub async fn write_to_stream<TStream>(
    &self,
    stream: &mut TStream,
    static_headers: &HashMap<String, String>,
  ) -> anyhow::Result<()>
  where
    TStream: AsyncRead + AsyncWrite + Unpin,
  {
    let status = self.status.unwrap_or(200);
    let status_message = self
      .status_message
      .as_ref()
      .map(|f| f.as_str())
      .or(Response::status_message(status))
      .unwrap_or("");

    let empty_headers = HashMap::<String, String>::with_capacity(0); // TODO: move to static
    let headers = self.headers.as_ref().unwrap_or(&empty_headers);

    if self.buffer.is_some() {
      let buf: &[u8] = self.buffer.as_ref().unwrap();
      let content_length = buf.len();

      let headers_buf = Self::headers_buf(content_length, static_headers, headers);
      stream
        .write_all(format!("HTTP/1.1 {status} {status_message}\r\n{headers_buf}\r\n",).as_bytes())
        .await?;

      stream.write_all(buf).await?;
    } else {
      let body_buf = match &self.body {
        serde_json::Value::String(str) => str.to_owned(),
        _ => (&self).body.to_string(),
      };
      let content_length = body_buf.len();
      let headers_buf = Self::headers_buf(content_length, static_headers, headers);

      stream
        .write_all(
          format!("HTTP/1.1 {status} {status_message}\r\n{headers_buf}\r\n{body_buf}",).as_bytes(),
        )
        .await?;
    }

    Ok(())
  }

  fn headers_buf(
    content_length: usize,
    static_headers: &HashMap<String, String>,
    headers: &HashMap<String, String>,
  ) -> String {
    let mut r = String::new();
    let mut set = HashSet::<&String>::new();

    static_headers
      .iter()
      .for_each(|(key, value)| match headers.get(key) {
        Some(h) => {
          set.insert(key);
          r.push_str(format!("{key}: {} \r\n", h.as_str()).as_str())
        }
        None => r.push_str(format!("{key}: {value} \r\n").as_str()),
      });

    headers.iter().for_each(|(key, value)| {
      if set.contains(key) || key == "Content-Length" {
        return;
      }
      r.push_str(format!("{key}: {value} \r\n").as_str());
    });

    r.push_str(format!("Content-Length: {} \r\n", content_length).as_str());

    r
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
