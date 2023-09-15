use std::collections::HashMap;
use std::rc::Rc;

use super::*;
use crate::ascii::LogDisplay;
use crate::byte::pascal_string;
use crate::log;

#[derive(Debug, Default)]
pub struct MetaAtom {
  pub handler_type: Str<4>,
  pub ilst: EncodedAtom<IlstAtom>,
  pub hdlr: EncodedAtom<MetaHdlrAtom>,
  pub keys: EncodedAtom<KeysAtom>,
}

impl AtomDecoder for MetaAtom {
  const NAME: [u8; 4] = *b"meta";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = [0; 4];
    if decoder.brand.is_isom() {
      decoder.read_exact(&mut data)?;
      atom.offset += 4;
    }

    let mut meta = Self {
      handler_type: Str(data),
      ..Default::default()
    };
    for atom in atom.atoms(decoder) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"ilst" => meta.ilst = EncodedAtom::Encoded(atom),
          b"hdlr" => meta.hdlr = EncodedAtom::Encoded(atom),
          b"keys" => meta.keys = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[meta] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[meta] {e}"),
      }
    }

    Ok(meta)
  }
}

impl MetaAtom {
  pub fn tags(&mut self, decoder: &mut Decoder) -> AtomResult<HashMap<Rc<str>, Rc<str>>> {
    match &*self.hdlr.decode(decoder)?.handler_type {
      b"mdta" => {
        let Ok(keys) = self.keys.decode(decoder) else {
          return Ok(HashMap::new())
        };
        let Ok(values) = self.ilst.decode(decoder) else {
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
        let Ok(values) = self.ilst.decode(decoder) else {
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

impl AtomDecoder for MetaHdlrAtom {
  const NAME: [u8; 4] = *b"hdlr";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    // __reserved__ (4 bytes)
    let handler_type = Str::try_from(&data[8..12])?;
    // __reserved__ (12 bytes)
    let (name, _) = pascal_string(&data[24..]);

    Ok(Self {
      atom,
      version,
      flags,
      handler_type,
      name,
    })
  }
}

#[derive(Debug, Default)]
pub struct KeysAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub entry_count: u32,
  pub key_values: Vec<KeyValueAtom>,
}

impl AtomDecoder for KeysAtom {
  const NAME: [u8; 4] = *b"keys";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let entry_count = u32::from_be_bytes((&data[4..8]).try_into()?);
    atom.offset += 8;
    let mut atoms = atom.atoms(decoder);
    let mut key_values = Vec::new();
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => key_values.push(KeyValueAtom::decode_unchecked(atom, atoms.reader)?),
        Err(e) => log!(err@"#[keys] {e}"),
      }
    }

    Ok(Self {
      atom,
      version,
      flags,
      entry_count,
      key_values,
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

impl AtomDecoder for KeyValueAtom {
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    Ok(Self {
      namespace: atom.name,
      value: std::str::from_utf8(&data).unwrap_or_default().into(),
    })
  }
}

#[derive(Debug, Default)]
pub struct IlstAtom {
  pub items: Vec<IlstItem>,
}

impl AtomDecoder for IlstAtom {
  const NAME: [u8; 4] = *b"ilst";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut items = Vec::new();
    let mut atoms = atom.atoms(decoder);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => items.push(IlstItem::new(atom, atoms.reader)?),
        Err(e) => log!(err@"#[ilst] {e}"),
      }
    }
    Ok(Self { items })
  }
}

#[derive(Debug, Default)]
pub struct IlstItem {
  pub atom: Atom,
  pub index: u32,
  pub data: DataAtom,
}

impl IlstItem {
  fn new(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut atoms = atom.atoms(decoder);
    let data = DataAtom::decode(atoms.next().ok_or(AtomError::IlstData)??, decoder)?;
    Ok(Self {
      atom,
      index: u32::from_be_bytes(*atom.name),
      data,
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

impl AtomDecoder for DataAtom {
  const NAME: [u8; 4] = *b"data";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;
    let type_indicator = (&data[..4]).try_into()?;
    let locale_indicator = (&data[4..8]).try_into()?;
    let value = std::str::from_utf8(&data[8..]).unwrap_or_default().into();

    Ok(Self {
      type_indicator,
      locale_indicator,
      value,
    })
  }
}
