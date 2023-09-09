use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct MdiaBox {
  mdhd: Option<MdhdBox>,
  hdlr: Option<HdlrBox>,
  minf: Option<MinfBox>,
}

impl MdiaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut mdhd = None;
    let mut hdlr = None;
    let mut minf = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Mdhd(atom) => mdhd = Some(atom),
          AtomBox::Hdlr(atom) => hdlr = Some(atom),
          AtomBox::Minf(atom) => minf = Some(atom),
          _ => log!(warn@"#[MDIA] {atom:#?}"),
        },
        Err(e) => log!(err@"#[MDIA] {e}"),
      }
    }

    Ok(Self { mdhd, hdlr, minf })
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
