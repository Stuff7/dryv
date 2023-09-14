mod edts;
mod mdat;
mod mdia;
mod meta;
mod minf;
mod moov;
mod root;
mod stbl;
mod trak;

pub use edts::*;
pub use mdat::*;
pub use mdia::*;
pub use meta::*;
pub use minf::*;
pub use moov::*;
pub use root::*;
pub use stbl::*;
pub use trak::*;

use crate::{byte::Str, math::MathError};
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
  str::Utf8Error,
  string::FromUtf8Error,
};
use thiserror::Error;

use super::Decoder;

const HEADER_SIZE: u64 = 8;

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
  #[error("Math Error\n{0}")]
  Math(#[from] MathError),
  #[error("Atom not found {:?}", Str(*(.0)))]
  AtomNotFound([u8; 4]),
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
}

pub type AtomResult<T = ()> = Result<T, AtomError>;

pub fn decode_header(data: &[u8]) -> AtomResult<(u32, &[u8])> {
  let size = u32::from_be_bytes((&data[..4]).try_into()?);
  Ok((size, &data[4..8]))
}

pub fn decode_version_flags(bytes: &[u8]) -> (u8, [u8; 3]) {
  (bytes[0], [bytes[1], bytes[2], bytes[3]])
}

pub fn unpack_language_code(bytes: &[u8]) -> AtomResult<[u8; 3]> {
  let code = u16::from_be_bytes((bytes).try_into()?);
  let char1 = ((code >> 10) & 0x1F) as u8 + 0x60;
  let char2 = ((code >> 5) & 0x1F) as u8 + 0x60;
  let char3 = (code & 0x1F) as u8 + 0x60;
  Ok([char1, char2, char3])
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Atom {
  pub size: u32,
  pub name: Str<4>,
  pub offset: u64,
}

impl Atom {
  pub fn read_data<R: Read + Seek>(&mut self, reader: &mut R) -> AtomResult<Vec<u8>> {
    let mut data = vec![0; self.size as usize];
    reader.seek(SeekFrom::Start(self.offset))?;
    reader.read_exact(&mut data)?;
    Ok(data)
  }

  pub fn read_data_exact<R: Read + Seek, const S: usize>(
    &mut self,
    reader: &mut R,
  ) -> AtomResult<[u8; S]> {
    if S as u32 > self.size {
      return Err(AtomError::NotEnoughData(*self, S));
    }
    let mut data = [0; S];
    reader.seek(SeekFrom::Start(self.offset))?;
    reader.read_exact(&mut data)?;
    Ok(data)
  }

  pub fn atoms<'a, R: Read + Seek>(&self, reader: &'a mut R) -> AtomIter<'a, R> {
    AtomIter::new(reader, self.offset, self.offset + self.size as u64)
  }
}

pub struct AtomIter<'a, R: Read + Seek> {
  reader: &'a mut R,
  buffer: [u8; HEADER_SIZE as usize],
  start: u64,
  end: u64,
}

impl<'a, R: Read + Seek> AtomIter<'a, R> {
  pub fn new(reader: &'a mut R, start: u64, end: u64) -> Self {
    Self {
      reader,
      buffer: [0; HEADER_SIZE as usize],
      start,
      end,
    }
  }
}

impl<'a, R: Read + Seek> Iterator for AtomIter<'a, R> {
  type Item = AtomResult<Atom>;

  fn next(&mut self) -> Option<Self::Item> {
    (self.start + HEADER_SIZE < self.end).then(|| {
      self.reader.seek(SeekFrom::Start(self.start))?;
      self.reader.read_exact(&mut self.buffer)?;
      decode_header(&self.buffer).and_then(|(atom_size, atom_type)| {
        let offset = self.start + HEADER_SIZE;
        self.start += atom_size as u64;
        Str::<4>::try_from(atom_type)
          .map(|name| Atom {
            size: if atom_size < HEADER_SIZE as u32 {
              atom_size
            } else {
              atom_size - HEADER_SIZE as u32
            },
            name,
            offset,
          })
          .map_err(AtomError::from)
      })
    })
  }
}

#[derive(Debug)]
pub struct UnknownAtom(Atom);

impl AtomDecoder for UnknownAtom {
  const NAME: [u8; 4] = [0; 4];
  fn decode_unchecked(atom: Atom, _: &mut Decoder) -> AtomResult<Self> {
    Err(AtomError::UnknownAtom(atom))
  }
}

#[derive(Debug, Default)]
pub enum EncodedAtom<T: AtomDecoder = UnknownAtom> {
  Encoded(Atom),
  Decoded(T),
  #[default]
  None,
}

impl<T: AtomDecoder> EncodedAtom<T> {
  pub fn decode(&mut self, decoder: &mut Decoder) -> AtomResult<&mut T> {
    match self {
      EncodedAtom::Decoded(decoded) => Ok(decoded),
      EncodedAtom::Encoded(atom) => {
        let decoded = T::decode(*atom, decoder)?;
        *self = EncodedAtom::Decoded(decoded);
        if let EncodedAtom::Decoded(decoded) = self {
          Ok(decoded)
        } else {
          Err(AtomError::AtomNotFound(T::NAME))
        }
      }
      EncodedAtom::None => Err(AtomError::AtomNotFound(T::NAME)),
    }
  }
}

pub trait AtomDecoder: std::marker::Sized {
  const NAME: [u8; 4] = [0; 4];
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self>;
  #[inline]
  fn decode(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    if *atom.name == Self::NAME {
      Self::decode_unchecked(atom, decoder)
    } else {
      Err(AtomError::AtomType(Str(Self::NAME), atom.name))
    }
  }
}
