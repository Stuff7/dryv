use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct MetaAtom {
  pub ilst: EncodedAtom<IlstAtom>,
  pub hdlr: EncodedAtom<MetaHdlrAtom>,
  pub keys: EncodedAtom<KeysAtom>,
}

impl AtomDecoder for MetaAtom {
  const NAME: [u8; 4] = *b"meta";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    // atom.offset += 4; // TODO Handle isom ftyp
    let mut meta = Self::default();
    for atom in atom.atoms(reader) {
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

#[derive(Debug, Default)]
pub struct MetaHdlrAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub handler_type: Str<4>,
  pub name: String,
}

impl AtomDecoder for MetaHdlrAtom {
  const NAME: [u8; 4] = *b"hdlr";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    // __reserved__ (4 bytes)
    let handler_type = Str::try_from(&data[8..12])?;
    // __reserved__ (12 bytes)
    // __name_len__ (1 byte)
    let name = String::from_utf8_lossy(&data[25..]).to_string();

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
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let entry_count = u32::from_be_bytes((&data[4..8]).try_into()?);
    atom.offset += 8;
    let mut atoms = atom.atoms(reader);
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

#[derive(Debug, Default)]
pub struct KeyValueAtom {
  pub namespace: Str<4>,
  pub value: String,
}

impl AtomDecoder for KeyValueAtom {
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    Ok(Self {
      namespace: atom.name,
      value: String::from_utf8_lossy(&data).to_string(),
    })
  }
}

#[derive(Debug, Default)]
pub struct IlstAtom {
  pub items: Vec<IlstItem>,
}

impl AtomDecoder for IlstAtom {
  const NAME: [u8; 4] = *b"ilst";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut items = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => items.push(IlstItem::new(atom, atoms.reader)?), // TODO: use decode
        Err(e) => log!(err@"#[ilst] {e}"),
      }
    }
    Ok(Self { items })
  }
}

#[derive(Debug, Default)]
pub struct IlstItem {
  pub index: u32,
  pub data: Vec<DataAtom>,
}

impl IlstItem {
  fn new<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut data = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => data.push(DataAtom::decode(atom, &mut atoms.reader)?),
        Err(e) => log!(err@"#[data] {e}"),
      }
    }
    Ok(Self {
      index: u32::from_be_bytes(*atom.name),
      data,
    })
  }
}

#[derive(Debug)]
pub struct DataAtom {
  pub type_indicator: [u8; 4],
  pub locale_indicator: [u8; 4],
  pub value: String,
}

impl AtomDecoder for DataAtom {
  const NAME: [u8; 4] = *b"data";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;
    let type_indicator = (&data[..4]).try_into()?;
    let locale_indicator = (&data[4..8]).try_into()?;
    let value = String::from_utf8_lossy(&data[8..]).to_string();

    Ok(Self {
      type_indicator,
      locale_indicator,
      value,
    })
  }
}
