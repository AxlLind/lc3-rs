use std::fs;
use std::io::Result;
use itertools::Itertools;

pub fn read_image(path: &str) -> Result<(Vec<u16>, u16)> {
  let buf = fs::read(path)?;
  let mut inst_iter = buf.iter()
    .tuples()
    .map(|(&a,&b)| (a as u16) << 8 | b as u16);
  let pc_start = inst_iter.next().unwrap();
  let program  = inst_iter.collect();
  Ok((program, pc_start))
}
