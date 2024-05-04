mod head;
mod headers;
mod parser;
mod request;

pub use head::*;
pub use headers::*;
pub use parser::*;
pub use request::*;

type VecOffset = (usize, usize);
