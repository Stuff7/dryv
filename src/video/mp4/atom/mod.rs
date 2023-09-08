mod mdat;
mod meta;
mod moov;
mod trak;

pub use mdat::*;
pub use meta::*;
pub use moov::*;
pub use trak::*;

use crate::ascii::LogDisplay;
use crate::log;
use crate::math::MathError;
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
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
  #[error(transparent)]
  StringConversion(#[from] FromUtf8Error),
  #[error("Could not convert Vec to array\n{0:?}")]
  VecConversion(Vec<String>),
  #[error("Unknown box type {0:?}")]
  UnknownType(String),
  #[error("Invalid {0:?} box size {1}")]
  Size(&'static str, u32),
  #[error("Math Error\n{0}")]
  Math(#[from] MathError),
}

pub type BoxResult<T = ()> = Result<T, BoxError>;

pub fn decode_header<'a, R: Read + Seek>(
  buffer: &'a mut [u8; BOX_HEADER_SIZE as usize],
  reader: &mut R,
  offset: &mut u32,
) -> BoxResult<(u32, &'a [u8])> {
  reader.seek(SeekFrom::Start(*offset as u64))?;
  reader.read_exact(buffer)?;

  let size = u32::from_be_bytes((&buffer[..4]).try_into()?);
  *offset += size;

  log!(info@"BOX {} {size}", String::from_utf8_lossy(&buffer[4..]).to_string());
  Ok((size, &buffer[4..]))
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
  Tool(ToolBox),
  Tkhd(TkhdBox),
  Data(DataBox),
  Mdat(MdatBox),
  Edts(EdtsBox),
  Elst(ElstBox),
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
    (self.offset + BOX_HEADER_SIZE < self.end).then_some(
      decode_header(&mut self.buffer, self.reader, &mut self.offset).and_then(|(bsize, btype)| {
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
          b"data" => DataBox::new(self.reader, size).map(AtomBox::Data),
          b"\xA9too" => ToolBox::new(self.reader, offset, size).map(AtomBox::Tool),
          b"free" => Ok(AtomBox::Free),
          e => Err(BoxError::UnknownType(
            String::from_utf8_lossy(e).to_string(),
          )),
        }
      }),
    )
  }
}

/// The `FtypBox` struct represents a File Type Box (ftyp) in an MP4 file.
///
/// A File Type Box specifies the major brand and compatible brands of an MP4 file, providing information
/// about the file's format and compatibility. It helps media players identify whether they can handle
/// the given MP4 file.
///
/// The `FtypBox` struct is essential for identifying the format and compatibility of an MP4 file,
/// allowing media players to determine if they can correctly interpret and play the file.
///
/// # Structure
///
/// - `compatible_brands`: An array of four strings representing the compatible brands. These brands
///   indicate which brands can be used in the file for various features.
///
/// - `major_brand`: A string representing the major brand, which defines the core specification that
///   the file adheres to. This brand is crucial for identifying the file's format.
///
/// - `minor_version`: A 32-bit unsigned integer representing the minor version of the major brand.
///   It provides additional information about the version of the format.
#[derive(Debug)]
pub struct FtypBox {
  compatible_brands: [String; 4],
  major_brand: String,
  minor_version: u32,
}

impl FtypBox {
  pub fn new(reader: &mut dyn Read, size: u32) -> BoxResult<Self> {
    if size != 24 {
      return Err(BoxError::Size("ftyp", size));
    }

    let mut buffer = [0; 24];
    reader.read_exact(&mut buffer)?;

    let major_brand = String::from_utf8_lossy(&buffer[..4]).to_string();
    let minor_version = u32::from_be_bytes((&buffer[4..8]).try_into()?);
    let compatible_brands: [String; 4] = buffer[8..]
      .chunks_exact(4)
      .map(|bytes| String::from_utf8_lossy(bytes).to_string())
      .collect::<Vec<_>>()
      .try_into()
      .map_err(BoxError::VecConversion)?;

    Ok(Self {
      compatible_brands,
      major_brand,
      minor_version,
    })
  }
}
