use super::*;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct MdatAtom {
  pub atom: Atom,
  pub extended_size: u64,
}

impl MdatAtom {
  pub fn new<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let extended_size = if atom.size == 1 {
      let mut buffer = [0; 8];
      reader.read_exact(&mut buffer)?;
      atom.offset += 8;
      u64::from_be_bytes((&buffer[..8]).try_into()?) - 8
    } else {
      atom.size as u64
    };

    Ok(Self {
      atom,
      extended_size,
    })
  }
}
