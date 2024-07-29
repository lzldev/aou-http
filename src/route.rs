use crate::methods::HttpMethod;

#[derive(Debug, Clone, Copy)]
pub struct AouRoute<T> {
  GET: Option<T>,
  HEAD: Option<T>,
  POST: Option<T>,
  PUT: Option<T>,
  DELETE: Option<T>,
  CONNECT: Option<T>,
  OPTIONS: Option<T>,
  TRACE: Option<T>,
  PATCH: Option<T>,
  ALL: Option<T>,
}

impl<T> Default for AouRoute<T> {
  fn default() -> Self {
    Self {
      GET: None,
      HEAD: None,
      POST: None,
      PUT: None,
      DELETE: None,
      CONNECT: None,
      OPTIONS: None,
      TRACE: None,
      PATCH: None,
      ALL: None,
    }
  }
}

impl<T> AouRoute<T> {
  pub fn get_method(&self, method: HttpMethod) -> &Option<T> {
    match method {
      HttpMethod::GET => &self.GET,
      HttpMethod::HEAD => &self.HEAD,
      HttpMethod::POST => &self.POST,
      HttpMethod::PUT => &self.PUT,
      HttpMethod::DELETE => &self.DELETE,
      HttpMethod::CONNECT => &self.CONNECT,
      HttpMethod::OPTIONS => &self.OPTIONS,
      HttpMethod::TRACE => &self.TRACE,
      HttpMethod::PATCH => &self.PATCH,
    }
  }
  pub fn get_all(&self) -> &Option<T> {
    &self.ALL
  }

  pub fn set_method(&mut self, method: HttpMethod, value: T) {
    match method {
      HttpMethod::GET => self.GET = Some(value),
      HttpMethod::HEAD => self.HEAD = Some(value),
      HttpMethod::POST => self.POST = Some(value),
      HttpMethod::PUT => self.PUT = Some(value),
      HttpMethod::DELETE => self.DELETE = Some(value),
      HttpMethod::CONNECT => self.CONNECT = Some(value),
      HttpMethod::OPTIONS => self.OPTIONS = Some(value),
      HttpMethod::TRACE => self.TRACE = Some(value),
      HttpMethod::PATCH => self.PATCH = Some(value),
    }
  }

  pub fn set_all(&mut self, value: T) {
    self.ALL = Some(value)
  }
}
