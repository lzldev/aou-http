#[macro_use]
extern crate napi_derive;

pub mod request;
pub mod server;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
