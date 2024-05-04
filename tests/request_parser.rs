#[cfg(test)]
use aou::utils::range_from_subslice;

fn main() {
  let ppp = b"Hello World";

  let mut iter = ppp.split(|c| c == &b' ');
  let hello = iter.next().unwrap();
  let world = iter.next().unwrap();

  dbg!(range_from_subslice(ppp, hello));
  let world_range = range_from_subslice(ppp, world);
  dbg!(world_range);

  let world2 = &ppp[world_range.0..world_range.1];
  dbg!(String::from_utf8_lossy(world2));
}
