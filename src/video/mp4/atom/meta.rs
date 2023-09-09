use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

// Metadata Box
#[derive(Debug)]
pub struct MetaBox {
  pub ilst: Option<IlstBox>,
  pub hdlr: Option<HdlrBox>,
}

impl MetaBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + 4 + size);
    atoms.offset = offset + 4;
    let mut ilst = None;
    let mut hdlr = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Ilst(atom) => ilst = Some(atom),
          AtomBox::Hdlr(atom) => hdlr = Some(atom),
          _ => log!(warn@"#[META] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[META] {e}"),
      }
    }

    Ok(Self { ilst, hdlr })
  }
}

#[derive(Debug)]
pub struct IlstBox {
  items: Vec<AtomBox>,
}

impl IlstBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut items = Vec::new();
    for atom in atoms {
      match atom {
        Ok(atom) => items.push(atom),
        Err(e) => log!(err@"#[ILST] Misplaced atom {e}"),
      }
    }

    Ok(Self { items })
  }
}

/// ©too Encoder tag
#[derive(Debug)]
pub struct ToolBox {
  pub data: Option<DataBox>,
}

impl ToolBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut data = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Data(atom) => data = Some(atom),
          _ => log!(warn@"#[TOOL] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[TOOL] {e}"),
      }
    }

    Ok(Self { data })
  }
}

#[derive(Debug)]
pub struct DataBox {
  pub type_indicator: [u8; 4],
  pub locale_indicator: [u8; 4],
  pub value: String,
}

impl DataBox {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> BoxResult<Self> {
    let mut buffer = vec![0; size as usize];
    reader.read_exact(&mut buffer)?;
    let type_indicator = (&buffer[..4]).try_into()?;
    let locale_indicator = (&buffer[4..8]).try_into()?;
    let value = String::from_utf8_lossy(&buffer[8..]).to_string();

    Ok(Self {
      type_indicator,
      locale_indicator,
      value,
    })
  }
}
