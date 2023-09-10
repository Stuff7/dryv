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

use crate::math::MathError;
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
  ops::Deref,
  str::Utf8Error,
  string::FromUtf8Error,
};
use thiserror::Error;

const BOX_HEADER_SIZE: u32 = 8;

#[derive(Debug, Error)]
pub enum BoxError {
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  SliceConversion(#[from] TryFromSliceError),
  #[error("Not enough chunks")]
  ChunkConversion,
  #[error(transparent)]
  StringConversion(#[from] FromUtf8Error),
  #[error(transparent)]
  Utf8Conversion(#[from] Utf8Error),
  #[error("Unknown box type {0:?}")]
  UnknownType(String),
  #[error("Invalid {0:?} box size {1}")]
  Size(&'static str, u32),
  #[error("Math Error\n{0}")]
  Math(#[from] MathError),
}

pub type BoxResult<T = ()> = Result<T, BoxError>;

pub fn decode_header(data: &[u8]) -> BoxResult<(u32, &[u8])> {
  let size = u32::from_be_bytes((&data[..4]).try_into()?);
  Ok((size, &data[4..8]))
}

pub fn decode_version_flags(bytes: &[u8]) -> (u8, [u8; 3]) {
  (bytes[0], [bytes[1], bytes[2], bytes[3]])
}

#[derive(Debug)]
pub enum AtomBox {
  Ftyp(FtypBox),
  Mvhd(MvhdBox),
  Moov(MoovBox),
  Udta(UdtaBox),
  Meta(MetaBox),
  Trak(TrakBox),
  Mdia(MdiaBox),
  Mdhd(MdhdBox),
  Hdlr(HdlrBox),
  Ilst(IlstBox),
  Tkhd(TkhdBox),
  Mdat(MdatBox),
  Edts(EdtsBox),
  Elst(ElstBox),
  Minf(MinfBox),
  Vmhd(VmhdBox),
  Smhd(SmhdBox),
  Dinf(DinfBox),
  Dref(DrefBox),
  Stbl(StblBox),
  Stsd(StsdBox),
  Free,
}

pub struct AtomBoxIter<'a, R: Read + Seek> {
  buffer: [u8; BOX_HEADER_SIZE as usize],
  reader: &'a mut R,
  offset: u32,
  end: u32,
}

impl<'a, R: Read + Seek> AtomBoxIter<'a, R> {
  pub fn new(reader: &'a mut R, end: u32) -> Self {
    Self {
      buffer: [0; BOX_HEADER_SIZE as usize],
      reader,
      offset: 0,
      end,
    }
  }
}

impl<'a, R: Read + Seek> Iterator for AtomBoxIter<'a, R> {
  type Item = BoxResult<AtomBox>;

  fn next(&mut self) -> Option<Self::Item> {
    (self.offset + BOX_HEADER_SIZE < self.end).then(|| {
      self.reader.seek(SeekFrom::Start(self.offset as u64))?;
      self.reader.read_exact(&mut self.buffer)?;
      decode_header(&self.buffer).and_then(|(bsize, btype)| {
        self.offset += bsize;
        let size = bsize - BOX_HEADER_SIZE;
        let offset = self.offset - bsize + BOX_HEADER_SIZE;
        match btype {
          b"ftyp" => FtypBox::new(self.reader, size).map(AtomBox::Ftyp),
          b"moov" => MoovBox::new(self.reader, offset, size).map(AtomBox::Moov),
          b"mdat" => MdatBox::new(self.reader, offset, size).map(AtomBox::Mdat),
          b"edts" => EdtsBox::new(self.reader, offset, size).map(AtomBox::Edts),
          b"elst" => ElstBox::new(self.reader, size).map(AtomBox::Elst),
          b"udta" => UdtaBox::new(self.reader, offset, size).map(AtomBox::Udta),
          b"meta" => MetaBox::new(self.reader, offset, size).map(AtomBox::Meta),
          b"mvhd" => MvhdBox::new(self.reader, size).map(AtomBox::Mvhd),
          b"trak" => TrakBox::new(self.reader, offset, size).map(AtomBox::Trak),
          b"mdia" => MdiaBox::new(self.reader, offset, size).map(AtomBox::Mdia),
          b"mdhd" => MdhdBox::new(self.reader, size).map(AtomBox::Mdhd),
          b"hdlr" => HdlrBox::new(self.reader, size).map(AtomBox::Hdlr),
          b"ilst" => IlstBox::new(self.reader, offset, size).map(AtomBox::Ilst),
          b"tkhd" => TkhdBox::new(self.reader, size).map(AtomBox::Tkhd),
          b"minf" => MinfBox::new(self.reader, offset, size).map(AtomBox::Minf),
          b"stbl" => StblBox::new(self.reader, offset, size).map(AtomBox::Stbl),
          b"stsd" => StsdBox::new(self.reader, offset, size).map(AtomBox::Stsd),
          b"vmhd" => VmhdBox::new(self.reader, size).map(AtomBox::Vmhd),
          b"smhd" => SmhdBox::new(self.reader, size).map(AtomBox::Smhd),
          b"dinf" => DinfBox::new(self.reader, offset, size).map(AtomBox::Dinf),
          b"dref" => DrefBox::new(self.reader, offset, size).map(AtomBox::Dref),
          b"free" => Ok(AtomBox::Free),
          e => Err(BoxError::UnknownType(
            String::from_utf8_lossy(e).to_string(),
          )),
        }
      })
    })
  }
}

#[derive(Debug)]
pub struct BoxHeader {
  size: u32,
  name: Str<4>,
  data: Vec<u8>,
}

pub struct BoxHeaderIter<'a, R: Read + Seek> {
  buffer: [u8; BOX_HEADER_SIZE as usize],
  reader: &'a mut R,
  offset: u32,
  end: u32,
}

impl<'a, R: Read + Seek> BoxHeaderIter<'a, R> {
  pub fn new(reader: &'a mut R, offset: u32, end: u32) -> Self {
    Self {
      buffer: [0; BOX_HEADER_SIZE as usize],
      reader,
      offset,
      end,
    }
  }
}

impl<'a, R: Read + Seek> Iterator for BoxHeaderIter<'a, R> {
  type Item = BoxResult<BoxHeader>;

  fn next(&mut self) -> Option<Self::Item> {
    (self.offset + BOX_HEADER_SIZE < self.end).then(|| {
      self.reader.seek(SeekFrom::Start(self.offset as u64))?;
      self.reader.read_exact(&mut self.buffer)?;
      decode_header(&self.buffer).and_then(|(bsize, btype)| {
        self.offset += bsize;
        let size = bsize - BOX_HEADER_SIZE;
        let mut data = vec![0; size as usize];
        Str::<4>::try_from(btype).and_then(|name| {
          self
            .reader
            .read_exact(&mut data)
            .map(|_| BoxHeader { size, name, data })
            .map_err(BoxError::from)
        })
      })
    })
  }
}

#[derive(Debug)]
pub struct FtypBox {
  pub compatible_brands: Vec<Str<4>>,
  pub major_brand: Str<4>,
  pub minor_version: u32,
}

impl FtypBox {
  pub fn new(reader: &mut dyn Read, size: u32) -> BoxResult<Self> {
    let mut buffer = vec![0; size as usize];
    reader.read_exact(&mut buffer)?;

    let major_brand = Str::try_from(&buffer[..4])?;
    let minor_version = u32::from_be_bytes((&buffer[4..8]).try_into()?);
    let compatible_brands: Vec<Str<4>> = buffer[8..]
      .chunks_exact(4)
      .map(Str::<4>::try_from)
      .collect::<BoxResult<_>>()?;

    Ok(Self {
      compatible_brands,
      major_brand,
      minor_version,
    })
  }
}

pub fn unpack_language_code(bytes: &[u8]) -> BoxResult<[u8; 3]> {
  let code = u16::from_be_bytes((bytes).try_into()?);
  let char1 = ((code >> 10) & 0x1F) as u8 + 0x60;
  let char2 = ((code >> 5) & 0x1F) as u8 + 0x60;
  let char3 = (code & 0x1F) as u8 + 0x60;
  Ok([char1, char2, char3])
}

pub struct Str<const N: usize>(pub [u8; N]);

impl<const N: usize> Str<N> {
  pub fn as_string(&self) -> String {
    self.0.map(|c| c as char).iter().collect()
  }
}

impl<const N: usize> TryFrom<&[u8]> for Str<N> {
  type Error = BoxError;
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
    write!(f, "{:?}", self.as_string())
  }
}
