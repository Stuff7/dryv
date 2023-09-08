use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

/// The `TrakBox` struct represents a Track Box (trak) in an MP4 file.
///
/// A Track Box contains information about a specific track within an MP4 file, such as video,
/// audio, or other types of media tracks. It typically includes a `TkhdBox` (Track Header Box)
/// that provides detailed track-specific metadata.
///
/// The `TrakBox` struct is a fundamental building block for organizing and managing tracks within an MP4 file.
/// It allows for the inclusion of track-specific metadata through the `TkhdBox`, which is often necessary for
/// proper playback and synchronization of media content.
/// # Structure
///
/// - `tkhd`: An optional `TkhdBox` struct that contains track-specific metadata. It may be absent
///   if the track information is not available or not needed.
#[derive(Debug)]
pub struct TrakBox {
  tkhd: Option<TkhdBox>,
  mdia: Option<MdiaBox>,
  edts: Vec<EdtsBox>,
}

impl TrakBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut tkhd = None;
    let mut mdia = None;
    let mut edts = Vec::new();
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Tkhd(atom) => tkhd = Some(atom),
          AtomBox::Mdia(atom) => mdia = Some(atom),
          AtomBox::Edts(atom) => edts.push(atom),
          _ => log!(warn@"TRAK SUB-BOX: {atom:#?}"),
        },
        Err(e) => log!(err@"{e}"),
      }
    }

    Ok(Self { tkhd, mdia, edts })
  }
}

/// The `TkhdBox` struct represents the Track Header Box (tkhd) in an MP4 file.
///
/// The Track Header Box contains metadata specific to an individual track within the movie,
/// including information such as its duration, dimensions, and track ID.
///
/// The `TkhdBox` struct is an essential component for managing individual tracks within an MP4 file.
/// It provides key details about each track's characteristics and presentation.
///
/// # Structure
///
/// - `version`: An 8-bit field that specifies the version of this box's format.
///
/// - `flags`: A 24-bit field that contains flags indicating how the box should be treated.
///
/// - `creation_time`: A 32-bit unsigned integer representing the creation time of the track.
///
/// - `modification_time`: A 32-bit unsigned integer representing the time when the track was last modified.
///
/// - `track_id`: A 32-bit unsigned integer representing the unique identifier for the track.
///
/// - `duration`: A 32-bit unsigned integer representing the duration of the track in time scale units.
///
/// - `layer`: A 16-bit integer that specifies the front-to-back ordering of video tracks. A lower value
///   indicates a higher layer.
///
/// - `alternate_group`: A 16-bit integer that identifies a group of tracks that can be used as alternate
///   representations of the same content. Tracks in the same alternate group are mutually exclusive.
///
/// - `volume`: A 32-bit floating-point number representing the audio volume. 1.0 is full volume.
///
/// - `matrix`: A `Matrix3x3` struct representing a 3x3 matrix that describes how to transform
///   the track's visual presentation.
///
/// - `width`: A 32-bit floating-point number representing the width of the track's visual presentation.
///
/// - `height`: A 32-bit floating-point number representing the height of the track's visual presentation.
#[derive(Debug)]
pub struct TkhdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub track_id: u32,
  pub duration: u32,
  pub layer: u16,
  pub alternate_group: u16,
  pub volume: f32,
  pub matrix: Matrix3x3,
  pub width: f32,
  pub height: f32,
}

impl TkhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 84];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let creation_time = u32::from_be_bytes((&buffer[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&buffer[8..12]).try_into()?);
    let track_id = u32::from_be_bytes((&buffer[12..16]).try_into()?);
    // __reserved__ 32 bit     (4 bytes)
    let duration = u32::from_be_bytes((&buffer[20..24]).try_into()?);
    // __reserved__ 32 bit [2] (8 bytes)
    let layer = u16::from_be_bytes((&buffer[32..34]).try_into()?);
    let alternate_group = u16::from_be_bytes((&buffer[34..36]).try_into()?);
    let volume = fixed_point_to_f32(i16::from_be_bytes((&buffer[36..38]).try_into()?) as f32, 8);
    // __reserved__ 16 bit     (2 bytes)
    let matrix = Matrix3x3::from_bytes(&buffer[40..])?;
    let width = fixed_point_to_f32(i32::from_be_bytes((&buffer[76..80]).try_into()?) as f32, 16);
    let height = fixed_point_to_f32(i32::from_be_bytes((&buffer[80..84]).try_into()?) as f32, 16);

    Ok(Self {
      version,
      flags,
      creation_time,
      modification_time,
      track_id,
      duration,
      layer,
      alternate_group,
      volume,
      matrix,
      width,
      height,
    })
  }
}

#[derive(Debug)]
pub struct MdiaBox {
  mdhd: Option<MdhdBox>,
  hdlr: Option<HdlrBox>,
}

impl MdiaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut mdhd = None;
    let mut hdlr = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Mdhd(atom) => mdhd = Some(atom),
          AtomBox::Hdlr(atom) => hdlr = Some(atom),
          _ => log!(warn@"MDIA SUB-BOX: {atom:#?}"),
        },
        Err(e) => log!(err@"{e}"),
      }
    }

    Ok(Self { mdhd, hdlr })
  }
}

#[derive(Debug)]
pub struct MdhdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub timescale: u32,
  pub duration: u32,
  pub language: u16,
}

impl MdhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 22];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let creation_time = u32::from_be_bytes((&buffer[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&buffer[8..12]).try_into()?);
    let timescale = u32::from_be_bytes((&buffer[12..16]).try_into()?);
    let duration = u32::from_be_bytes((&buffer[16..20]).try_into()?);
    let language = u16::from_be_bytes((&buffer[20..22]).try_into()?);

    Ok(Self {
      version,
      flags,
      creation_time,
      timescale,
      modification_time,
      duration,
      language,
    })
  }
}

#[derive(Debug)]
pub struct HdlrBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub component_type: String,
  pub component_name: String,
}

impl HdlrBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 24];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    // __reserved__ 32 bit     (4 bytes)
    let component_type = String::from_utf8_lossy(&buffer[8..12]).to_string();
    // __reserved__ 32 bit [3] (12 bytes)
    let component_name = match size - 25 {
      s if s < 1 => String::new(),
      s => {
        let mut buffer = vec![0; s as usize];
        reader.read_exact(&mut buffer)?;
        String::from_utf8(buffer)?
      }
    };

    Ok(Self {
      version,
      flags,
      component_type,
      component_name,
    })
  }
}

#[derive(Debug)]
pub struct EdtsBox {
  pub elst: Option<ElstBox>,
}

impl EdtsBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut elst = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Elst(atom) => elst = Some(atom),
          _ => log!(warn@"EDTS SUB-BOX: {atom:#?}"),
        },
        Err(e) => log!(err@"{e}"),
      }
    }

    Ok(Self { elst })
  }
}

#[derive(Debug)]
pub struct ElstBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub edit_list_table: Vec<ElstEntry>,
}

impl ElstBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 8];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let number_of_entries = u32::from_be_bytes((&buffer[4..8]).try_into()?);

    let mut buffer = vec![0; number_of_entries as usize * 12];
    reader.read_exact(&mut buffer)?;

    let edit_list_table = buffer
      .chunks(12)
      .map(ElstEntry::from_bytes)
      .collect::<BoxResult<_>>()?;

    Ok(Self {
      version,
      flags,
      number_of_entries,
      edit_list_table,
    })
  }
}

#[derive(Debug)]
pub struct ElstEntry {
  pub track_duration: u32,
  pub media_time: i32,
  pub media_rate: f32,
}

impl ElstEntry {
  pub fn from_bytes(bytes: &[u8]) -> BoxResult<Self> {
    Ok(Self {
      track_duration: u32::from_be_bytes((&bytes[..4]).try_into()?),
      media_time: i32::from_be_bytes((&bytes[4..8]).try_into()?),
      media_rate: fixed_point_to_f32(i32::from_be_bytes((&bytes[8..12]).try_into()?) as f32, 16),
    })
  }
}
