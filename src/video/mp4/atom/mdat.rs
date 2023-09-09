use super::*;
use crate::log;
use crate::{ascii::LogDisplay, math::fixed_point_to_f32};
use std::io::{Read, Seek};

// Metadata Box
#[derive(Debug)]
pub struct MdatBox {
  extended_size: i64,
  data: Vec<AtomBox>,
}

impl MdatBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let extended_size = if size == 1 {
      let mut buffer = [0; 8];
      reader.read_exact(&mut buffer)?;
      i64::from_be_bytes((&buffer[..8]).try_into()?)
    } else {
      0
    };
    let mut atoms = AtomBoxIter::new(reader, offset + 8 + size);
    atoms.offset = offset + 8;
    let mut data = Vec::new();
    for atom in atoms {
      match atom {
        Ok(atom) => log!(warn@"#[MDAT] {atom:#?}"),
        Err(e) => log!(err@"#[MDAT] {e}"),
      }
    }

    Ok(Self {
      extended_size,
      data,
    })
  }
}
