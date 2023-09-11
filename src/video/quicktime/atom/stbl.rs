use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct StblAtom {
  pub stsd: EncodedAtom<StsdAtom>,
  pub stts: EncodedAtom<SttsAtom>,
  pub stss: Option<EncodedAtom<StssAtom>>,
  pub ctts: Option<EncodedAtom<CttsAtom>>,
  pub stsc: EncodedAtom<StscAtom>,
  pub stsz: EncodedAtom<StszAtom>,
  pub stco: EncodedAtom<StcoAtom>,
  pub sgpd: Option<EncodedAtom<SgpdAtom>>,
  pub sbgp: Option<EncodedAtom<SbgpAtom>>,
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
          b"stss" => stbl.stss = Some(EncodedAtom::Encoded(atom)),
          b"ctts" => stbl.ctts = Some(EncodedAtom::Encoded(atom)),
          b"stsc" => stbl.stsc = EncodedAtom::Encoded(atom),
          b"stsz" => stbl.stsz = EncodedAtom::Encoded(atom),
          b"stco" => stbl.stco = EncodedAtom::Encoded(atom),
          b"sgpd" => stbl.sgpd = Some(EncodedAtom::Encoded(atom)),
          b"sbgp" => stbl.sbgp = Some(EncodedAtom::Encoded(atom)),
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

#[derive(Debug)]
pub struct StssAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sync_sample_table: Vec<u32>,
}

impl AtomDecoder for StssAtom {
  const NAME: [u8; 4] = *b"stss";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    let sync_sample_table = data[8..]
      .chunks(4)
      .map(|b| {
        b.try_into()
          .map(u32::from_be_bytes)
          .map_err(AtomError::from)
      })
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
      sync_sample_table,
    })
  }
}

#[derive(Debug)]
pub struct CttsAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub entry_count: u32,
}

impl AtomDecoder for CttsAtom {
  const NAME: [u8; 4] = *b"ctts";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let entry_count = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      entry_count,
    })
  }
}

#[derive(Debug)]
pub struct StscAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sample_to_chunk_table: Vec<StscItem>,
}

impl AtomDecoder for StscAtom {
  const NAME: [u8; 4] = *b"stsc";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    let sample_to_chunk_table = data[8..]
      .chunks(8)
      .map(StscItem::from_bytes)
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
      sample_to_chunk_table,
    })
  }
}

#[derive(Debug)]
pub struct StscItem {
  pub first_chunk: u32,
  pub samples_per_chunk: u32,
  pub sample_description_id: u32,
}

impl StscItem {
  pub fn from_bytes(data: &[u8]) -> AtomResult<Self> {
    let first_chunk = u32::from_be_bytes((&data[..4]).try_into()?);
    let samples_per_chunk = u32::from_be_bytes((&data[..4]).try_into()?);
    let sample_description_id = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      first_chunk,
      samples_per_chunk,
      sample_description_id,
    })
  }
}

#[derive(Debug)]
pub struct StszAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub sample_size: u32,
  pub number_of_entries: u32,
  pub sample_size_table: Vec<u32>,
}

impl AtomDecoder for StszAtom {
  const NAME: [u8; 4] = *b"stsz";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let sample_size = u32::from_be_bytes((&data[4..8]).try_into()?);
    let number_of_entries = u32::from_be_bytes((&data[8..12]).try_into()?);

    let sample_size_table = data[12..]
      .chunks(4)
      .map(|b| {
        b.try_into()
          .map(u32::from_be_bytes)
          .map_err(AtomError::from)
      })
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      sample_size,
      number_of_entries,
      sample_size_table,
    })
  }
}

#[derive(Debug)]
pub struct StcoAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub chunk_offset_table: Vec<u32>,
}

impl AtomDecoder for StcoAtom {
  const NAME: [u8; 4] = *b"stco";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    let chunk_offset_table = data[8..]
      .chunks(4)
      .map(|b| {
        b.try_into()
          .map(u32::from_be_bytes)
          .map_err(AtomError::from)
      })
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
      chunk_offset_table,
    })
  }
}

#[derive(Debug)]
pub struct SgpdAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub grouping_table: u32,
  pub default_length: u32,
  pub entry_count: u32,
  pub payload_data: Vec<u16>,
}

impl AtomDecoder for SgpdAtom {
  const NAME: [u8; 4] = *b"sgpd";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let grouping_table = u32::from_be_bytes((&data[4..8]).try_into()?);
    let default_length = u32::from_be_bytes((&data[8..12]).try_into()?);
    let entry_count = u32::from_be_bytes((&data[12..16]).try_into()?);

    let payload_data = data[16..]
      .chunks(2)
      .map(|b| {
        b.try_into()
          .map(u16::from_be_bytes)
          .map_err(AtomError::from)
      })
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      grouping_table,
      default_length,
      entry_count,
      payload_data,
    })
  }
}

#[derive(Debug)]
pub struct SbgpAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub grouping_type: Str<4>,
  pub entry_count: u32,
  pub sample_count: u32,
  pub group_description_index: u32,
}

impl AtomDecoder for SbgpAtom {
  const NAME: [u8; 4] = *b"sbgp";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let grouping_type = Str::try_from(&data[4..8])?;
    let entry_count = u32::from_be_bytes((&data[8..12]).try_into()?);
    let sample_count = u32::from_be_bytes((&data[12..16]).try_into()?);

    let group_description_index = u32::from_be_bytes((&data[16..20]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      grouping_type,
      entry_count,
      sample_count,
      group_description_index,
    })
  }
}
