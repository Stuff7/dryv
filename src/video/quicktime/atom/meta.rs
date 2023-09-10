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
  items: Vec<DataBox>,
}

impl IlstBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let items = BoxHeaderIter::new(reader, offset, offset + size)
      .filter_map(|res| match res {
        Ok(header) => Some(DataBox::new(header)),
        Err(e) => {
          log!(err@"#[ILST] {e}");
          None
        }
      })
      .collect::<BoxResult<_>>()?;
    Ok(Self { items })
  }
}

/// ©too Encoder tag
#[derive(Debug)]
pub struct DataBox {
  pub name: Str<4>,
  pub data: Vec<DataItem>,
}

impl DataBox {
  pub fn new(header: BoxHeader) -> BoxResult<Self> {
    let mut data = Vec::new();
    let mut offset = 0;
    while offset < header.size as usize {
      let (size, name) = decode_header(&header.data)?;
      let size = size as usize;
      data.push(DataItem::new(
        &header.data[offset + 8..size],
        Str::try_from(name)?,
      )?);
      offset += size;
    }
    Ok(Self {
      name: header.name,
      data,
    })
  }
}

#[derive(Debug)]
pub struct DataItem {
  pub name: Str<4>,
  pub type_indicator: [u8; 4],
  pub locale_indicator: [u8; 4],
  pub value: String,
}

impl DataItem {
  pub fn new(data: &[u8], name: Str<4>) -> BoxResult<Self> {
    let type_indicator = (&data[..4]).try_into()?;
    let locale_indicator = (&data[4..8]).try_into()?;
    let value = String::from_utf8_lossy(&data[8..]).to_string();

    Ok(Self {
      name,
      type_indicator,
      locale_indicator,
      value,
    })
  }
}
