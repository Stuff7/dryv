use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct StblBox {
  pub stsd: Option<StsdBox>,
}

impl StblBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut stsd = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Stsd(atom) => stsd = Some(atom),
          _ => log!(warn@"#[STBL] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[STBL] {e}"),
      }
    }

    Ok(Self { stsd })
  }
}
#[derive(Debug)]
pub struct StsdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sample_description_table: Vec<StsdEntry>,
}

impl StsdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 8];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let number_of_entries = u32::from_be_bytes((&buffer[4..8]).try_into()?);

    let sample_description_table = BoxHeaderIter::new(reader, offset + 8, offset + size)
      .filter_map(|res| match res {
        Ok(header) => Some(StsdEntry::new(header)),
        Err(e) => {
          log!(err@"#[STSD] {e}");
          None
        }
      })
      .collect::<BoxResult<_>>()?;

    Ok(Self {
      version,
      flags,
      number_of_entries,
      sample_description_table,
    })
  }
}

#[derive(Debug)]
pub struct StsdEntry {
  pub data_format: String,
  pub dref_index: u16,
  pub extra_data: Vec<u8>,
}

impl StsdEntry {
  pub fn new(header: BoxHeader) -> BoxResult<Self> {
    let dref_index = u16::from_be_bytes((&header.data[6..8]).try_into()?);
    let extra_data = (&header.data[8..]).into();

    Ok(Self {
      data_format: header.name,
      dref_index,
      extra_data,
    })
  }
}
