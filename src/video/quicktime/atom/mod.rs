mod decoder;
mod edts;
mod iter;
mod mdat;
mod mdia;
mod meta;
mod minf;
mod moov;
mod root;
mod stbl;
mod stsd;
mod trak;

pub use decoder::*;
pub use edts::*;
pub use iter::*;
pub use mdat::*;
pub use mdia::*;
pub use meta::*;
pub use minf::*;
pub use moov::*;
pub use root::*;
pub use stbl::*;
pub use stsd::*;
pub use trak::*;

use super::Decoder;
use crate::byte::Str;
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
  str::Utf8Error,
  string::FromUtf8Error,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AtomError {
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  SliceConversion(#[from] TryFromSliceError),
  #[error(transparent)]
  StringConversion(#[from] FromUtf8Error),
  #[error(transparent)]
  Utf8Conversion(#[from] Utf8Error),
  #[error("Required atom {:?} was not found", Str(*(.0)))]
  Required([u8; 4]),
  #[error("Unknown atom {0:?}")]
  UnknownAtom(Atom),
  #[error("Atom type mismatch, expected {0:?} got {1:?}")]
  AtomType(Str<4>, Str<4>),
  #[error("Unsupported meta handler {0:?}")]
  MetaHandler(Str<4>),
  #[error("Meta value index {0} not found in keys {1:?}")]
  MetaKeyValue(usize, String),
  #[error("Requested {1} bytes from {} with {} bytes of data", .0.name, .0.size)]
  NotEnoughData(Atom, usize),
  #[error("Meta item has no data")]
  IlstData,
  #[error("MinfAtom is missing media handler [vmhd | smhd | gmhd]")]
  NoMinfHandler,
  #[error("MetaAtom is missing handler")]
  NoMetaHandler,
}

pub type AtomResult<T = ()> = Result<T, AtomError>;

#[derive(Debug, Default, Clone, Copy)]
pub struct Atom {
  pub size: u32,
  pub name: Str<4>,
  pub offset: u64,
}

impl Atom {
  fn new(size: u32, name: &[u8], offset: u64) -> AtomResult<Self> {
    Ok(Self {
      size: if size <= HEADER_SIZE as u32 {
        size
      } else {
        size - HEADER_SIZE as u32
      },
      name: Str::try_from(name)?,
      offset,
    })
  }

  pub fn read_data<R: Read + Seek>(&mut self, reader: &mut R) -> AtomResult<AtomData> {
    reader.seek(SeekFrom::Start(self.offset))?;
    let data = if self.size == 0 {
      let mut data = Vec::new();
      reader.read_to_end(&mut data)?;
      data
    } else {
      let mut data = vec![0; self.size as usize];
      reader.read_exact(&mut data)?;
      data
    };
    Ok(AtomData::new(&data, self.offset))
  }

  pub fn read_data_exact<const S: usize, R: Read + Seek>(
    &mut self,
    reader: &mut R,
  ) -> AtomResult<AtomData> {
    if S as u32 > self.size {
      return Err(AtomError::NotEnoughData(*self, S));
    }
    let mut data = [0; S];
    reader.seek(SeekFrom::Start(self.offset))?;
    reader.read_exact(&mut data)?;
    Ok(AtomData::new(&data, self.offset))
  }

  pub fn atoms<'a, R: Read + Seek>(&self, reader: &'a mut R) -> AtomIter<'a, R> {
    AtomIter::new(reader, self.offset, self.offset + self.size as u64)
  }
}
