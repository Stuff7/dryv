use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct MetaAtom {
  pub ilst: EncodedAtom<IlstAtom>,
  pub hdlr: EncodedAtom<HdlrAtom>,
}

impl AtomDecoder for MetaAtom {
  const NAME: [u8; 4] = *b"meta";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    atom.offset += 4;
    let mut meta = Self::default();
    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"ilst" => meta.ilst = EncodedAtom::Encoded(atom),
          b"hdlr" => meta.hdlr = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[meta] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[meta] {e}"),
      }
    }

    Ok(meta)
  }
}

#[derive(Debug, Default)]
pub struct IlstAtom {
  pub items: Vec<DataAtom>,
}

impl AtomDecoder for IlstAtom {
  const NAME: [u8; 4] = *b"ilst";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut items = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => items.push(DataAtom::decode_unchecked(atom, atoms.reader)?), // TODO: use decode
        Err(e) => log!(err@"#[ilst] {e}"),
      }
    }
    Ok(Self { items })
  }
}

#[derive(Debug, Default)]
pub struct DataAtom {
  pub name: Str<4>,
  pub data: Vec<DataItem>,
}

impl AtomDecoder for DataAtom {
  const NAME: [u8; 4] = *b"data";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut data = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(mut atom) => data.push(DataItem::new(&atom.read_data(atoms.reader)?, atom.name)?),
        Err(e) => log!(err@"#[data] {e}"),
      }
    }
    Ok(Self {
      name: atom.name,
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
  pub fn new(data: &[u8], name: Str<4>) -> AtomResult<Self> {
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
