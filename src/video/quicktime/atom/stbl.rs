use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct StblAtom {
  pub stsd: EncodedAtom<StsdAtom>,
  pub stts: EncodedAtom<SttsAtom>,
}

impl AtomDecoder for StblAtom {
  const NAME: [u8; 4] = *b"stbl";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut stbl = Self::default();
    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"stsd" => stbl.stsd = EncodedAtom::Encoded(atom),
          b"stts" => stbl.stts = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[stbl] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[stbl] {e}"),
      }
    }

    Ok(stbl)
  }
}

#[derive(Debug)]
pub struct StsdAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sample_description_table: Vec<StsdItem>,
}

impl AtomDecoder for StsdAtom {
  const NAME: [u8; 4] = *b"stsd";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    atom.offset += 8;
    let mut sample_description_table = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(mut atom) => {
          sample_description_table.push(StsdItem::new(&atom.read_data(atoms.reader)?, atom.name)?)
        }
        Err(e) => log!(err@"#[stsd] {e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      number_of_entries,
      sample_description_table,
    })
  }
}

#[derive(Debug)]
pub struct StsdItem {
  pub data_format: Str<4>,
  pub dref_index: u16,
  pub extra_data: Vec<u8>,
}

impl StsdItem {
  pub fn new(data: &[u8], data_format: Str<4>) -> AtomResult<Self> {
    let dref_index = u16::from_be_bytes((&data[6..8]).try_into()?);
    let extra_data = (&data[8..]).into();

    Ok(Self {
      data_format,
      dref_index,
      extra_data,
    })
  }
}

#[derive(Debug)]
pub struct SttsAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub time_to_sample_table: Vec<SttsItem>,
}

impl AtomDecoder for SttsAtom {
  const NAME: [u8; 4] = *b"stts";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    let time_to_sample_table = data[8..]
      .chunks(8)
      .map(SttsItem::from_bytes)
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
      time_to_sample_table,
    })
  }
}

#[derive(Debug)]
pub struct SttsItem {
  pub sample_count: u32,
  pub sample_duration: u32,
}

impl SttsItem {
  pub fn from_bytes(data: &[u8]) -> AtomResult<Self> {
    let sample_count = u32::from_be_bytes((&data[..4]).try_into()?);
    let sample_duration = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      sample_count,
      sample_duration,
    })
  }
}
