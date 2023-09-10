use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct TrakBox {
  pub tkhd: Option<TkhdBox>,
  pub mdia: Option<MdiaBox>,
  pub edts: Option<EdtsBox>,
}

impl TrakBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut tkhd = None;
    let mut mdia = None;
    let mut edts = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Tkhd(atom) => tkhd = Some(atom),
          AtomBox::Mdia(atom) => mdia = Some(atom),
          AtomBox::Edts(atom) => edts = Some(atom),
          _ => log!(warn@"#[TRAK] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[TRAK] {e}"),
      }
    }

    Ok(Self { tkhd, mdia, edts })
  }
}

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
