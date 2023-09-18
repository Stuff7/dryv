use std::collections::HashMap;
use std::rc::Rc;

use super::*;
use crate::ascii::LogDisplay;
use crate::byte::pascal_string;
use crate::log;

#[derive(Debug, Default)]
pub struct MetaAtom {
  pub atom: Atom,
  pub hdlr: MetaHdlrAtom,
  pub keys: Option<KeysAtom>,
  pub ilst: Option<IlstAtom>,
}

impl MetaAtom {
  pub fn new(atom: Atom, data: AtomData) -> AtomResult<Self> {
    let mut content = (None, None, None);
    for atom in data.atoms() {
      match atom {
        Ok((atom, data)) => match &*atom.name {
          b"hdlr" => content.0 = Some(MetaHdlrAtom::new(atom, AtomData::new(data, atom.offset))?),
          b"keys" => content.1 = Some(KeysAtom::new(atom, AtomData::new(data, atom.offset))?),
          b"ilst" => content.2 = Some(IlstAtom::new(AtomData::new(data, atom.offset))?),
          _ => log!(warn@"#[meta] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[meta] {e}"),
      }
    }

    Ok(Self {
      atom,
      hdlr: content.0.ok_or(AtomError::NoMetaHandler)?,
      keys: content.1,
      ilst: content.2,
    })
  }
}

impl MetaAtom {
  pub fn tags(&mut self) -> AtomResult<HashMap<Rc<str>, Rc<str>>> {
    match &*self.hdlr.handler_type {
      b"mdta" => {
        let Some(keys) = &mut self.keys else {
          return Ok(HashMap::new())
        };
        let Some(values) = &mut self.ilst else {
          return Ok(HashMap::new())
        };
        values
          .items
          .iter()
          .map(|item| {
            let index = (item.index - 1) as usize;
            keys
              .key_values
              .get(index)
              .ok_or_else(|| AtomError::MetaKeyValue(index, format!("{:?}", keys.key_values)))
              .map(|key| (key.value.clone(), item.data.value.clone()))
          })
          .collect::<AtomResult<_>>()
      }
      b"mdir" => {
        let Some(values) = &mut self.ilst else {
          return Ok(HashMap::new())
        };
        Ok(
          values
            .items
            .iter()
            .map(|item| (item.atom.name.into(), item.data.value.clone()))
            .collect(),
        )
      }
      hdlr => Err(AtomError::MetaHandler(Str(*hdlr))),
    }
  }
}

#[derive(Debug, Default)]
pub struct MetaHdlrAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub handler_type: Str<4>,
  pub name: Box<str>,
}

impl MetaHdlrAtom {
  fn new(atom: Atom, mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      handler_type: data.reserved(4).next_into()?,
      name: pascal_string(data.reserved(12)),
    })
  }
}

#[derive(Debug, Default)]
pub struct KeysAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub entry_count: u32,
  pub key_values: Box<[KeyValueAtom]>,
}

impl KeysAtom {
  fn new(atom: Atom, mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      entry_count: data.next_into()?,
      key_values: data
        .atoms()
        .filter_map(|atom| match atom {
          Ok((atom, data)) => Some(KeyValueAtom::new(atom, data)),
          Err(e) => {
            log!(err@"#[keys] {e}");
            None
          }
        })
        .collect(),
    })
  }
}

#[derive(Debug)]
pub struct KeyValueAtom {
  pub namespace: Str<4>,
  pub value: Rc<str>,
}

impl Default for KeyValueAtom {
  fn default() -> Self {
    Self {
      namespace: Str::default(),
      value: "".into(),
    }
  }
}

impl KeyValueAtom {
  fn new(atom: Atom, data: &[u8]) -> Self {
    Self {
      namespace: atom.name,
      value: std::str::from_utf8(data).unwrap_or_default().into(),
    }
  }
}

#[derive(Debug, Default)]
pub struct IlstAtom {
  pub items: Box<[IlstItem]>,
}

impl IlstAtom {
  fn new(data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      items: data
        .atoms()
        .filter_map(|atom| match atom {
          Ok((atom, data)) => Some(IlstItem::new(atom, AtomData::new(data, atom.offset))),
          Err(e) => {
            log!(err@"#[ilst] {e}");
            None
          }
        })
        .collect::<AtomResult<_>>()?,
    })
  }
}

#[derive(Debug, Default)]
pub struct IlstItem {
  pub atom: Atom,
  pub index: u32,
  pub data: DataAtom,
}

impl IlstItem {
  fn new(atom: Atom, data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      atom,
      index: u32::from_be_bytes(*atom.name),
      data: DataAtom::new(AtomData::new(
        data.atoms().next().ok_or(AtomError::IlstData)??.1,
        atom.offset,
      ))?,
    })
  }
}

#[derive(Debug)]
pub struct DataAtom {
  pub type_indicator: [u8; 4],
  pub locale_indicator: [u8; 4],
  pub value: Rc<str>,
}

impl Default for DataAtom {
  fn default() -> Self {
    Self {
      type_indicator: [0; 4],
      locale_indicator: [0; 4],
      value: "".into(),
    }
  }
}

impl DataAtom {
  fn new(mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      type_indicator: data.next_into()?,
      locale_indicator: data.next_into()?,
      value: std::str::from_utf8(&data).unwrap_or_default().into(),
    })
  }
}
