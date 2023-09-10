use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct MoovBox {
  pub mvhd: Option<MvhdBox>,
  pub udta: Option<UdtaBox>,
  pub traks: Vec<TrakBox>,
}

impl MoovBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut mvhd = None;
    let mut udta = None;
    let mut traks = Vec::new();
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Mvhd(atom) => mvhd = Some(atom),
          AtomBox::Udta(atom) => udta = Some(atom),
          AtomBox::Trak(trak) => traks.push(trak),
          _ => log!(warn@"#[MOOV] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[MOOV] {e}"),
      }
    }

    Ok(Self { mvhd, traks, udta })
  }
}

#[derive(Debug)]
pub struct MvhdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub timescale: u32,
  pub duration: u32,
  pub rate: f32,
  pub volume: f32,
  pub matrix: Matrix3x3,
  pub next_track_id: u32,
}

impl MvhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    if size != 100 {
      return Err(BoxError::Size("mvhd", size));
    }
    let mut buffer = [0; 100];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let creation_time = u32::from_be_bytes((&buffer[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&buffer[8..12]).try_into()?);
    let timescale = u32::from_be_bytes((&buffer[12..16]).try_into()?);
    let duration = u32::from_be_bytes((&buffer[16..20]).try_into()?);
    let rate = fixed_point_to_f32(i32::from_be_bytes((&buffer[20..24]).try_into()?) as f32, 16);
    let volume = fixed_point_to_f32(i16::from_be_bytes((&buffer[24..26]).try_into()?) as f32, 8);
    // __reserved__    16 bit     (2 bytes)
    // __reserved__    32 bit [2] (8 bytes)
    let matrix = Matrix3x3::from_bytes(&buffer[36..])?;
    // __pre_defined__ 32 bit [6] (24 bytes)
    let next_track_id = u32::from_be_bytes((&buffer[96..100]).try_into()?);

    Ok(Self {
      version,
      flags,
      creation_time,
      modification_time,
      timescale,
      duration,
      rate,
      volume,
      matrix,
      next_track_id,
    })
  }
}

#[derive(Debug)]
pub struct UdtaBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub metas: Vec<MetaBox>,
}

impl UdtaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let mut metas = Vec::new();

    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Meta(meta) => metas.push(meta),
          _ => log!(warn@"#[UDTA] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[UDTA] {e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      metas,
    })
  }
}
