mod edts;
mod mdat;
mod mdia;
mod meta;
mod minf;
mod moov;
mod stbl;
mod trak;

pub use edts::*;
pub use mdat::*;
pub use mdia::*;
pub use meta::*;
pub use minf::*;
pub use moov::*;
pub use stbl::*;
pub use trak::*;

use crate::{ascii::LogDisplay, log, math::MathError};
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
  ops::Deref,
  str::Utf8Error,
  string::FromUtf8Error,
};
use thiserror::Error;

const HEADER_SIZE: u32 = 8;

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
  #[error("Invalid {0:?} atom size {1}")]
  Size(Str<4>, u32),
  #[error("Math Error\n{0}")]
  Math(#[from] MathError),
  #[error("Atom not found {:?}", Str(*(.0)))]
  AtomNotFound([u8; 4]),
  #[error("Unknown atom {0:?}")]
  UnknownAtom(Atom),
  #[error("Atom type mismatch, expected {0:?} got {1:?}")]
  AtomType(Str<4>, Str<4>),
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

#[derive(Clone, Copy)]
pub struct Str<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Str<N> {
  fn default() -> Self {
    Self([0; N])
  }
}

impl<const N: usize> Str<N> {
  pub fn as_string(&self) -> String {
    self.0.map(|c| c as char).iter().collect()
  }
}

impl<const N: usize> TryFrom<&[u8]> for Str<N> {
  type Error = AtomError;
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    Ok(Self(slice.try_into()?))
  }
}

impl<const N: usize> Deref for Str<N> {
  type Target = [u8; N];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<const N: usize> std::fmt::Display for Str<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.as_string())
  }
}

impl<const N: usize> std::fmt::Debug for Str<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "b{:?}", self.as_string())
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Atom {
  pub size: u32,
  pub name: Str<4>,
  pub offset: u32,
}

impl Atom {
  pub fn read_data<R: Read + Seek>(&mut self, reader: &mut R) -> AtomResult<Vec<u8>> {
    if self.size <= HEADER_SIZE {
      return Err(AtomError::Size(self.name, self.size));
    }
    let mut data = vec![0; (self.size - HEADER_SIZE) as usize];
    reader.seek(SeekFrom::Start((self.offset + HEADER_SIZE) as u64))?;
    reader.read_exact(&mut data)?;
    Ok(data)
  }

  pub fn atoms<'a, R: Read + Seek>(&self, reader: &'a mut R) -> AtomIter<'a, R> {
    AtomIter::new(reader, self.offset + HEADER_SIZE, self.offset + self.size)
  }
}

pub struct AtomIter<'a, R: Read + Seek> {
  reader: &'a mut R,
  buffer: [u8; HEADER_SIZE as usize],
  start: u32,
  end: u32,
}

impl<'a, R: Read + Seek> AtomIter<'a, R> {
  pub fn new(reader: &'a mut R, start: u32, end: u32) -> Self {
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
      self.reader.seek(SeekFrom::Start(self.start as u64))?;
      self.reader.read_exact(&mut self.buffer)?;
      decode_header(&self.buffer).and_then(|(atom_size, atom_type)| {
        let offset = self.start;
        self.start += atom_size;
        Str::<4>::try_from(atom_type).map(|name| Atom {
          size: atom_size,
          name,
          offset,
        })
      })
    })
  }
}

#[derive(Debug, Default)]
pub struct FtypAtom {
  pub atom: Atom,
  pub compatible_brands: Vec<Str<4>>,
  pub major_brand: Str<4>,
  pub minor_version: u32,
}

impl AtomDecoder for FtypAtom {
  const NAME: [u8; 4] = *b"ftyp";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let major_brand = Str::try_from(&data[..4])?;
    let minor_version = u32::from_be_bytes((&data[4..8]).try_into()?);
    let compatible_brands: Vec<Str<4>> = data[8..]
      .chunks_exact(4)
      .map(Str::<4>::try_from)
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      compatible_brands,
      major_brand,
      minor_version,
    })
  }
}

#[derive(Debug, Default)]
pub struct RootAtom {
  pub ftyp: FtypAtom,
  pub mdat: MdatAtom,
  pub moov: MoovAtom,
  pub rest: Vec<Atom>,
}

impl RootAtom {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> AtomResult<Self> {
    let mut ftyp = None;
    let mut mdat = None;
    let mut moov = None;
    let mut rest = Vec::new();
    let mut atoms = AtomIter::new(reader, 0, size);

    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"ftyp" => ftyp = Some(FtypAtom::decode_unchecked(atom, atoms.reader)?),
          b"mdat" => mdat = Some(MdatAtom::decode_unchecked(atom, atoms.reader)?),
          b"moov" => moov = Some(MoovAtom::decode_unchecked(atom, atoms.reader)?),
          _ => rest.push(atom),
        },
        Err(e) => log!(err@"#[root] {e}"),
      }
    }

    Ok(Self {
      ftyp: ftyp.ok_or(AtomError::AtomNotFound(*b"ftyp"))?,
      mdat: mdat.ok_or(AtomError::AtomNotFound(*b"mdat"))?,
      moov: moov.ok_or(AtomError::AtomNotFound(*b"moov"))?,
      rest,
    })
  }
}

#[derive(Debug)]
pub struct UnknownAtom(Atom);

impl AtomDecoder for UnknownAtom {
  const NAME: [u8; 4] = [0; 4];
  fn decode_unchecked<R: Read + Seek>(atom: Atom, _: &mut R) -> AtomResult<Self> {
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
  pub fn decode<R: Read + Seek>(&mut self, reader: &mut R) -> AtomResult<&mut T> {
    match self {
      EncodedAtom::Decoded(decoded) => Ok(decoded),
      EncodedAtom::Encoded(atom) => {
        let decoded = T::decode(*atom, reader)?;
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
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self>;
  #[inline]
  fn decode<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    if *atom.name == Self::NAME {
      Self::decode_unchecked(atom, reader)
    } else {
      Err(AtomError::AtomType(Str(Self::NAME), atom.name))
    }
  }
}
