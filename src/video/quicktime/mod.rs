pub mod atom;

use atom::*;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use thiserror::Error;

use crate::byte::Str;

#[derive(Debug, Error)]
pub enum DecoderError {
  #[error("Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
  #[error("Decoder does not support brand {0:?}")]
  Unsupported(Str<4>),
}

pub type DecoderResult<T = ()> = Result<T, DecoderError>;

pub enum DecoderBrand {
  QuickTime,
  Isom,
  None,
}

impl DecoderBrand {
  pub fn is_isom(&self) -> bool {
    matches!(self, DecoderBrand::Isom)
  }
}

impl TryFrom<Str<4>> for DecoderBrand {
  type Error = DecoderError;
  fn try_from(brand: Str<4>) -> Result<Self, Self::Error> {
    match &*brand {
      b"qt  " => Ok(Self::QuickTime),
      b"isom" => Ok(Self::Isom),
      _ => Err(DecoderError::Unsupported(brand)),
    }
  }
}

pub struct Decoder {
  pub file: File,
  pub size: u64,
  pub brand: DecoderBrand,
}

impl Decoder {
  pub fn open<P: AsRef<Path>>(path: P) -> DecoderResult<Self> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    Ok(Self {
      size,
      file,
      brand: DecoderBrand::None,
    })
  }

  pub fn decode_root(&mut self) -> DecoderResult<RootAtom> {
    let root = RootAtom::new(&mut self.file, self.size as u32)?;
    self.brand = DecoderBrand::try_from(root.ftyp.major_brand).or_else(|e| {
      root
        .ftyp
        .compatible_brands
        .iter()
        .find_map(|b| DecoderBrand::try_from(*b).ok())
        .ok_or(e)
    })?;
    Ok(root)
  }
}

impl Read for Decoder {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.file.read(buf)
  }
}

impl Seek for Decoder {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    self.file.seek(pos)
  }
}
