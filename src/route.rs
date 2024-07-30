use crate::methods::HttpMethod;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct Route<T> {
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

impl<T> Default for Route<T> {
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

impl<T> Route<T> {
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

  pub fn has_method(&self, method: HttpMethod) -> bool {
    match method {
      HttpMethod::GET => self.GET.is_some(),
      HttpMethod::HEAD => self.HEAD.is_some(),
      HttpMethod::POST => self.POST.is_some(),
      HttpMethod::PUT => self.PUT.is_some(),
      HttpMethod::DELETE => self.DELETE.is_some(),
      HttpMethod::CONNECT => self.CONNECT.is_some(),
      HttpMethod::OPTIONS => self.OPTIONS.is_some(),
      HttpMethod::TRACE => self.TRACE.is_some(),
      HttpMethod::PATCH => self.PATCH.is_some(),
    }
  }

  pub fn has_all(&self) -> bool {
    self.ALL.is_some()
  }

  pub fn set_all(&mut self, value: T) {
    self.ALL = Some(value)
  }
}
