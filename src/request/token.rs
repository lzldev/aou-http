#[derive(Debug)]
pub struct RequestToken(usize, usize);

impl RequestToken {
  pub fn from_subslice<T>(source: &[T], slice: &[T]) -> Self {
    let ptr = source.as_ptr() as usize;
    let range = slice.as_ptr_range();

    let start = range.start as usize - ptr;
    let end = range.end as usize - ptr;

    Self(start, end)
  }
}

// RUST DOCS:https://doc.rust-lang.org/std/primitive.str.html#representation
//😈
// // We can re-build a str out of ptr and len. This is all unsafe because
// // we are responsible for making sure the two components are valid:
// let s = unsafe {
//     // First, we build a &[u8]...
//     let slice = slice::from_raw_parts(ptr, len);

//     // ... and then convert that slice into a string slice
//     str::from_utf8(slice)
// };
