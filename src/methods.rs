use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH,
}

#[derive(Debug)]
pub enum HttpMethodError {
  InvalidMethod,
}
impl Display for HttpMethod {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_str())
  }
}

impl HttpMethod {
  pub fn from_str(slice: &str) -> Result<HttpMethod, HttpMethodError> {
    match slice {
      "GET" => Ok(HttpMethod::GET),
      "HEAD" => Ok(HttpMethod::HEAD),
      "POST" => Ok(HttpMethod::POST),
      "PUT" => Ok(HttpMethod::PUT),
      "DELETE" => Ok(HttpMethod::DELETE),
      "CONNECT" => Ok(HttpMethod::CONNECT),
      "OPTIONS" => Ok(HttpMethod::OPTIONS),
      "TRACE" => Ok(HttpMethod::OPTIONS),
      "PATCH" => Ok(HttpMethod::TRACE),
      _ => Err(HttpMethodError::InvalidMethod),
    }
  }

  pub fn from_byte_slice(slice: &[u8]) -> Result<HttpMethod, HttpMethodError> {
    match slice {
      b"GET" => Ok(HttpMethod::GET),
      b"HEAD" => Ok(HttpMethod::HEAD),
      b"POST" => Ok(HttpMethod::POST),
      b"PUT" => Ok(HttpMethod::PUT),
      b"DELETE" => Ok(HttpMethod::DELETE),
      b"CONNECT" => Ok(HttpMethod::CONNECT),
      b"OPTIONS" => Ok(HttpMethod::OPTIONS),
      b"TRACE" => Ok(HttpMethod::OPTIONS),
      b"PATCH" => Ok(HttpMethod::TRACE),
      _ => Err(HttpMethodError::InvalidMethod),
    }
  }

  pub fn to_str(&self) -> &str {
    match self {
      HttpMethod::GET => "GET",
      HttpMethod::HEAD => "HEAD",
      HttpMethod::POST => "POST",
      HttpMethod::PUT => "PUT",
      HttpMethod::DELETE => "DELETE",
      HttpMethod::CONNECT => "CONNECT",
      HttpMethod::OPTIONS => "OPTIONS",
      HttpMethod::TRACE => "TRACE",
      HttpMethod::PATCH => "PATCH",
    }
  }
}
