pub mod atom;

use atom::*;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use thiserror::Error;

use crate::byte::Str;

#[derive(Debug, Error)]
pub enum QTError {
  #[error("QuickTime Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
  #[error("Decoder does not support brand {0:?}")]
  Unsupported(Str<4>),
}

pub type QTResult<T = ()> = Result<T, QTError>;

pub enum DecoderBrand {
  QuickTime,
  Isom,
}

impl DecoderBrand {
  pub fn is_isom(&self) -> bool {
    matches!(self, DecoderBrand::Isom)
  }
}

impl TryFrom<Str<4>> for DecoderBrand {
  type Error = QTError;
  fn try_from(brand: Str<4>) -> Result<Self, Self::Error> {
    match &*brand {
      b"qt  " => Ok(Self::QuickTime),
      b"isom" => Ok(Self::Isom),
      _ => Err(QTError::Unsupported(brand)),
    }
  }
}

pub struct Decoder {
  pub file: File,
  pub size: u64,
  pub root: RootAtom,
  pub brand: DecoderBrand,
}

impl Decoder {
  pub fn open<P: AsRef<Path>>(path: P) -> QTResult<Self> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let root = RootAtom::new(&mut file, size as u32)?;
    let brand = DecoderBrand::try_from(root.ftyp.major_brand).or_else(|e| {
      root
        .ftyp
        .compatible_brands
        .iter()
        .find_map(|b| DecoderBrand::try_from(*b).ok())
        .ok_or(e)
    })?;
    Ok(Self {
      size,
      file,
      root,
      brand,
    })
  }

  pub fn decode(&mut self) -> QTResult<RootAtom> {
    RootAtom::new(&mut self.file, self.size as u32).map_err(QTError::from)
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
