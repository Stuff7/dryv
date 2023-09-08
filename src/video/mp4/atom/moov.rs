use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

/// The `MoovBox` struct represents a Movie Box (moov) in an MP4 file.
///
/// A Movie Box is a significant structural element in an MP4 file, containing essential metadata
/// and information about the movie, including details about individual tracks and user-defined metadata.
///
/// # Structure
///
/// - `mvhd`: An optional `MvhdBox` struct that provides metadata about the entire movie, including its duration
///   and creation time. This is a crucial component for describing the movie as a whole.
///
/// - `udta`: An optional `UdtaBox` struct that can store user-defined metadata associated with the movie.
///   This allows for custom annotations or additional information to be included in the MP4 file.
///
/// - `traks`: A vector of `TrakBox` instances, each representing a track within the movie. These tracks can
///   contain video, audio, or other types of media, each with its own specific characteristics.
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

/// The `MvhdBox` struct represents the Movie Header Box (mvhd) in an MP4 file.
///
/// The Movie Header Box provides essential metadata about the entire movie, such as its
/// duration, time scale, creation, and modification times, among other properties.
///
/// # Structure
///
/// - `version`: An 8-bit field that specifies the version of this box's format.
///
/// - `flags`: A 24-bit field that contains flags indicating how the box should be treated.
///
/// - `creation_time`: A 32-bit unsigned integer representing the creation time of the movie.
///
/// - `modification_time`: A 32-bit unsigned integer representing the time when the movie was last modified.
///
/// - `timescale`: A 32-bit unsigned integer representing the time scale for the entire movie.
///   The time scale is used to interpret the `duration` field, which is also a 32-bit unsigned integer.
///
/// - `duration`: A 32-bit unsigned integer representing the duration of the movie in the time scale units.
///
/// - `rate`: A 32-bit floating-point number representing the preferred playback rate for the movie.
///
/// - `volume`: A 16-bit fixed-point number representing the audio volume. 1.0 (0x0100) is full volume.
///
/// - `matrix`: A `Matrix3x3` struct representing a 3x3 matrix that describes how to transform
///   the movie's visual presentation.
///
/// - `next_track_id`: A 32-bit unsigned integer specifying the next available track ID for new tracks.
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

/// The `UdtaBox` struct represents a User Data Box (udta) in an MP4 file.
///
/// A User Data Box typically contains user-defined metadata or other custom data associated with
/// the MP4 file. It can include a list of `MetaBox` instances, each of which may store various types
/// of metadata.
///
/// The `UdtaBox` struct provides a container for user-defined metadata, which can be valuable for
/// storing additional information or annotations related to an MP4 file.
///
/// # Structure
///
/// - `version`: An 8-bit field that specifies the version of this box's format.
///
/// - `flags`: A 24-bit field that contains flags indicating how the box should be treated.
///
/// - `metas`: A vector of `MetaBox` instances, each containing specific metadata associated with the MP4 file.
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

    let (version, flags) = decode_version_flags(&buffer);
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
