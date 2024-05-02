mod parser;
mod request;

pub use parser::*;
pub use request::*;

use serde::{Deserialize, Serialize};

type RequestHeaders<'req> = Vec<(&'req [u8], &'req [u8])>; // Probably should be a hashmap already?

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestHead<'req> {
  pub method: &'req [u8],
  pub path: &'req [u8],
  pub http_version: &'req [u8],
}
