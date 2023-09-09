use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct MinfBox {
  pub media_header: Option<MediaHeader>,
  pub dinf: Option<DinfBox>,
  pub stbl: Option<StblBox>,
}

impl MinfBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut media_header = None;
    let mut dinf = None;
    let mut stbl = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Vmhd(atom) => media_header = Some(MediaHeader::Video(atom)),
          AtomBox::Smhd(atom) => media_header = Some(MediaHeader::Sound(atom)),
          AtomBox::Dinf(atom) => dinf = Some(atom),
          AtomBox::Stbl(atom) => stbl = Some(atom),
          _ => log!(warn@"#[MINF] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[MINF] {e}"),
      }
    }

    Ok(Self {
      media_header,
      dinf,
      stbl,
    })
  }
}

#[derive(Debug)]
pub enum MediaHeader {
  Video(VmhdBox),
  Sound(SmhdBox),
}

#[derive(Debug)]
pub struct VmhdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub graphics_mode: u16,
  pub opcolor: [u16; 3],
}

impl VmhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 12];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let graphics_mode = u16::from_be_bytes((&buffer[4..6]).try_into()?);
    let opcolor = [
      u16::from_be_bytes((&buffer[6..8]).try_into()?),
      u16::from_be_bytes((&buffer[8..10]).try_into()?),
      u16::from_be_bytes((&buffer[10..12]).try_into()?),
    ];

    Ok(Self {
      version,
      flags,
      graphics_mode,
      opcolor,
    })
  }
}

#[derive(Debug)]
pub struct SmhdBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub balance: u16,
}

impl SmhdBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 6];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let balance = u16::from_be_bytes((&buffer[4..6]).try_into()?);

    Ok(Self {
      version,
      flags,
      balance,
    })
  }
}

#[derive(Debug)]
pub struct DinfBox {
  pub data_ref_list: Option<DrefBox>,
}

impl DinfBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut data_ref_list = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Dref(atom) => data_ref_list = Some(atom),
          _ => log!(warn@"#[DINF] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[DINF] {e}"),
      }
    }

    Ok(Self { data_ref_list })
  }
}

#[derive(Debug)]
pub struct DrefBox {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub data_references: Vec<DrefEntry>,
}

impl DrefBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut buffer = [0; 8];
    reader.read_exact(&mut buffer)?;

    let (version, flags) = decode_version_flags(&buffer);
    let number_of_entries = u32::from_be_bytes((&buffer[4..8]).try_into()?);

    let mut atoms = AtomBoxIter::new(reader, offset + 8 + size);
    atoms.offset = offset + 8;
    let mut data_references = Vec::new();
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::DrefEntry(atom) => data_references.push(atom),
          _ => log!(warn@"#[DATA_REF_LIST] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[DATA_REF_LIST] {e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      number_of_entries,
      data_references,
    })
  }
}

#[derive(Debug)]
pub struct DrefEntry {
  pub box_type: String,
  pub version: u8,
  pub flags: [u8; 3],
  pub data: String,
}

impl DrefEntry {
  pub fn new<R: Read + Seek>(reader: &mut R, box_type: String, size: u32) -> BoxResult<Self> {
    let mut buffer = vec![0; size as usize];
    reader.read_exact(&mut buffer)?;
    let (version, flags) = decode_version_flags(&buffer);
    let data = String::from_utf8_lossy(&buffer[4..]).to_string();

    Ok(Self {
      box_type,
      version,
      flags,
      data,
    })
  }
}
