use super::*;
use crate::ascii::LogDisplay;
use crate::log;
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
  #[error("Invalid size {0}")]
  Size(u32),
}

pub type BoxResult<T = ()> = Result<T, BoxError>;

#[derive(Debug)]
pub enum BoxType {
  Ftyp,
  Moov,
  Mvhd,
  Free,
}

#[derive(Debug)]
pub struct BoxHeader {
  box_type: BoxType,
  size: u32,
}

impl BoxHeader {
  pub fn parse<R: Read + Seek>(reader: &mut R, offset: u64) -> BoxResult<Self> {
    reader.seek(SeekFrom::Start(offset))?;

    let mut buffer = [0; BOX_HEADER_SIZE as usize];
    reader.read_exact(&mut buffer)?;

    let size = u32::from_be_bytes((&buffer[..4]).try_into()?);
    log!(info@"BOX {}", String::from_utf8_lossy(&buffer[4..]).to_string());
    let box_type = match &buffer[4..] {
      b"ftyp" => BoxType::Ftyp,
      b"mvhd" => BoxType::Mvhd,
      b"moov" => BoxType::Moov,
      b"free" | b"mdat" => BoxType::Free,
      e => {
        return Err(BoxError::InvalidType(
          String::from_utf8_lossy(e).to_string(),
        ))
      }
    };

    Ok(Self { box_type, size })
  }
}

#[derive(Debug)]
pub enum AtomBox {
  Ftyp(FtypBox),
  Mvhd(MvhdBox),
  Moov(MoovBox),
  Free,
}

pub struct AtomBoxIter<'a, R: Read + Seek> {
  reader: &'a mut R,
  offset: u32,
  end: u32,
}

impl<'a, R: Read + Seek> AtomBoxIter<'a, R> {
  pub fn new(reader: &'a mut R, end: u32) -> Self {
    Self {
      reader,
      offset: 0,
      end,
    }
  }
}

impl<'a, R: Read + Seek> Iterator for AtomBoxIter<'a, R> {
  type Item = AtomBox;

  fn next(&mut self) -> Option<Self::Item> {
    if self.offset >= self.end {
      return None;
    }
    match BoxHeader::parse(self.reader, self.offset as u64) {
      Ok(header) => {
        self.offset += header.size;
        let size = header.size - BOX_HEADER_SIZE;
        return match header.box_type {
          BoxType::Ftyp => FtypBox::new(self.reader, size).ok().map(AtomBox::Ftyp),
          BoxType::Moov => MoovBox::new(self.reader, self.offset - header.size, size)
            .ok()
            .map(AtomBox::Moov),
          BoxType::Mvhd => MvhdBox::new(self.reader, size).ok().map(AtomBox::Mvhd),
          BoxType::Free => Some(AtomBox::Free),
        };
      }
      Err(e) => log!(err@"{e}"),
    }
    None
  }
}

#[derive(Debug)]
pub struct FtypBox {
  compatible_brands: [String; 4],
  major_brand: String,
  minor_version: u32,
}

impl FtypBox {
  pub fn new(reader: &mut dyn Read, size: u32) -> BoxResult<Self> {
    if size != 24 {
      return Err(BoxError::Size(size));
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

#[derive(Debug)]
pub struct MoovBox {
  mvhd: Option<MvhdBox>,
}

impl MoovBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, size);
    atoms.offset = offset;
    for atom in atoms {
      if let AtomBox::Mvhd(mvhd) = atom {
        return Ok(Self { mvhd: Some(mvhd) });
      }
    }

    Ok(Self { mvhd: None })
  }
}

#[derive(Debug)]
pub struct MvhdBox {
  version: u8,
  timescale: u32,
  duration: u32,
  rate: Fixed32,
  volume: Fixed16,
}

impl MvhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    if size != 26 {
      return Err(BoxError::Size(size));
    }

    let mut buffer = [0; 26];
    reader.read_exact(&mut buffer)?;

    let version = buffer[0];
    let timescale = u32::from_be_bytes((&buffer[12..16]).try_into()?);
    let duration = u32::from_be_bytes((&buffer[16..20]).try_into()?);
    let rate = Fixed32(bytes_to_i64(&buffer[20..24]));
    let volume = Fixed16(bytes_to_i32(&buffer[24..26]));

    Ok(Self {
      version,
      timescale,
      duration,
      rate,
      volume,
    })
  }
}
