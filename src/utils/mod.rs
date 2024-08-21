#[cfg(test)]
pub mod test;

pub fn range_from_subslice<T>(source: &[T], slice: &[T]) -> (usize, usize) {
  let ptr = source.as_ptr() as usize;
  let range = slice.as_ptr_range();

  let start = range.start as usize - ptr;
  let end = range.end as usize - ptr;

  (start, end)
}
