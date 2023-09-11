use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use crate::math::fixed_point_to_f32;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct EdtsAtom {
  pub elst: EncodedAtom<ElstAtom>,
}

impl AtomDecoder for EdtsAtom {
  const NAME: [u8; 4] = *b"edts";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut edts = Self::default();
    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"elst" => edts.elst = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[edts] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[edts] {e}"),
      }
    }

    Ok(edts)
  }
}

#[derive(Debug, Default)]
pub struct ElstAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub edit_list_table: Vec<ElstItem>,
}

impl AtomDecoder for ElstAtom {
  const NAME: [u8; 4] = *b"elst";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    let edit_list_table = data[8..]
      .chunks(12)
      .map(ElstItem::from_bytes)
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      version,
      flags,
      number_of_entries,
      edit_list_table,
    })
  }
}

#[derive(Debug, Default)]
pub struct ElstItem {
  pub track_duration: u32,
  pub media_time: i32,
  pub media_rate: f32,
}

impl ElstItem {
  pub fn from_bytes(bytes: &[u8]) -> AtomResult<Self> {
    Ok(Self {
      track_duration: u32::from_be_bytes((&bytes[..4]).try_into()?),
      media_time: i32::from_be_bytes((&bytes[4..8]).try_into()?),
      media_rate: fixed_point_to_f32(i32::from_be_bytes((&bytes[8..12]).try_into()?) as f32, 16),
    })
  }
}
