use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

/// Movie Box
#[derive(Debug)]
pub struct MoovBox {
  mvhd: Option<MvhdBox>,
  udta: Option<UdtaBox>,
  traks: Vec<TrakBox>,
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
          _ => log!(warn@"MOOV SUB-BOX: {atom:#?}"),
        },
        Err(e) => log!(err@"{e}"),
      }
    }

    Ok(Self { mvhd, traks, udta })
  }
}

/// # Movie Header Box
///
/// The Movie Header Box, represented by the `MvhdBox` struct, is a fundamental component in the structure of MPEG-4 (MP4) files. It contains essential metadata about the video, such as timing, duration, and playback characteristics.
///
/// ## Fields
///
/// - `version`: An 8-bit unsigned integer representing the version of the movie header box.
/// - `flags`: A 24-bit array of unsigned integers providing additional control and information flags.
/// - `creation_time`: A 32-bit unsigned integer indicating the creation time of the media presentation.
/// - `modification_time`: A 32-bit unsigned integer indicating the most recent modification time.
/// - `timescale`: A 32-bit unsigned integer specifying the time scale for the media's time units.
/// - `duration`: A 32-bit unsigned integer representing the duration of the media in time units.
/// - `rate`: A 32-bit floating-point number representing the playback rate.
/// - `volume`: A 32-bit floating-point number indicating the audio volume.
/// - `matrix`: A `Matrix3x3` structure representing a 3x3 matrix used for transformation.
/// - `next_track_id`: A 32-bit unsigned integer specifying the next available track ID.
#[derive(Debug)]
pub struct MvhdBox {
  version: u8,
  flags: [u8; 3],
  creation_time: u32,
  modification_time: u32,
  timescale: u32,
  duration: u32,
  rate: f32,
  volume: f32,
  matrix: Matrix3x3,
  next_track_id: u32,
}

impl MvhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    if size != 100 {
      return Err(BoxError::Size("mvhd", size));
    }
    let mut buffer = [0; 100];
    reader.read_exact(&mut buffer)?;

    let version = buffer[0];
    let flags = [buffer[1], buffer[2], buffer[3]];
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

/// User Data Box
#[derive(Debug)]
pub struct UdtaBox {
  version: u8,
  flags: [u8; 3],
  metas: Vec<MetaBox>,
}

impl UdtaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 4];
    reader.read_exact(&mut buffer)?;

    let version = buffer[0];
    let flags = [buffer[1], buffer[2], buffer[3]];
    let mut metas = Vec::new();

    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Meta(meta) => metas.push(meta),
          _ => log!(warn@"UDTA SUB-BOX: {atom:#?}"),
        },
        Err(e) => log!(err@"{e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      metas,
    })
  }
}

/// Metadata Box
#[derive(Debug)]
pub struct MetaBox {
  data: String,
}

impl MetaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = vec![0; size as usize];
    reader.read_exact(&mut buffer)?;
    let data = String::from_utf8_lossy(&buffer[..]).to_string();

    Ok(Self { data })
  }
}
