mod moov;
mod trak;

pub use moov::*;
pub use trak::*;

use crate::ascii::LogDisplay;
use crate::log;
use crate::math::MathError;
use std::{
  array::TryFromSliceError,
  io::{Read, Seek, SeekFrom},
};
use thiserror::Error;

const BOX_HEADER_SIZE: u32 = 8;

#[derive(Debug, Error)]
pub enum BoxError {
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  SliceConversion(#[from] TryFromSliceError),
  #[error("Could not convert Vec to array\n{0:?}")]
  VecConversion(Vec<String>),
  #[error("Invalid box type {0:?}")]
  InvalidType(String),
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

#[derive(Debug)]
pub enum AtomBox {
  Ftyp(FtypBox),
  Mvhd(MvhdBox),
  Moov(MoovBox),
  Udta(UdtaBox),
  Meta(MetaBox),
  Trak(TrakBox),
  Tkhd(TkhdBox),
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
    (self.offset < self.end).then_some(
      decode_header(&mut self.buffer, self.reader, &mut self.offset).and_then(|(bsize, btype)| {
        let size = bsize - BOX_HEADER_SIZE;
        let offset = self.offset - bsize + BOX_HEADER_SIZE;
        match btype {
          b"ftyp" => FtypBox::new(self.reader, size).map(AtomBox::Ftyp),
          b"moov" => MoovBox::new(self.reader, offset, size).map(AtomBox::Moov),
          b"udta" => UdtaBox::new(self.reader, offset, size).map(AtomBox::Udta),
          b"meta" => MetaBox::new(self.reader, size).map(AtomBox::Meta),
          b"mvhd" => MvhdBox::new(self.reader, size).map(AtomBox::Mvhd),
          b"trak" => TrakBox::new(self.reader, offset, size).map(AtomBox::Trak),
          b"tkhd" => TkhdBox::new(self.reader, size).map(AtomBox::Tkhd),
          b"free" => Ok(AtomBox::Free),
          e => Err(BoxError::InvalidType(
            String::from_utf8_lossy(e).to_string(),
          )),
        }
      }),
    )
  }
}

/// # File Type Box
/// The File Type Box, often referred to as 'ftyp', is an essential component found in the Audio and Video initialization segments of MP4 files. It serves as the highest-level box within these initialization segments. The information contained within the 'ftyp' box provides vital initial details necessary for decoding the incoming segments, particularly for video elements.
///
/// ## Usage
/// - There can only be one 'ftyp' box per file.
/// - The 'ftyp' box is mandatory within the initialization segments of an MP4 file, ensuring proper decoding of the media.
///
/// ## Structure
/// The 'FtypBox' struct represents the 'ftyp' box and contains the following fields:
///
/// - `compatible_brands`: An array of four strings, specifying compatible brands or file types.
/// - `major_brand`: A string representing the major brand of the file type.
/// - `minor_version`: A 32-bit unsigned integer indicating the minor version of the file type.
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
