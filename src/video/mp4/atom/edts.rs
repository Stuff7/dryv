use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use crate::math::fixed_point_to_f32;
use std::io::{Read, Seek};

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
          _ => log!(warn@"#[EDTS] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[EDTS] {e}"),
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
