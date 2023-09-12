use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct MinfAtom {
  pub kind: Option<MdiaAtomKind>,
  pub hdlr: EncodedAtom,
  pub dinf: EncodedAtom<DinfAtom>,
  pub stbl: EncodedAtom<StblAtom>,
}

impl AtomDecoder for MinfAtom {
  const NAME: [u8; 4] = *b"minf";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut minf = Self::default();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"vmhd" => {
            minf.kind = Some(MdiaAtomKind::Vmhd(VmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"smhd" => {
            minf.kind = Some(MdiaAtomKind::Smhd(SmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"gmhd" => {
            minf.kind = Some(MdiaAtomKind::Gmhd(GmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"hdlr" => minf.hdlr = EncodedAtom::Encoded(atom),
          b"dinf" => minf.dinf = EncodedAtom::Encoded(atom),
          b"stbl" => minf.stbl = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[minf] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[minf] {e}"),
      }
    }

    Ok(minf)
  }
}

#[derive(Debug)]
pub enum MdiaAtomKind {
  Vmhd(VmhdAtom),
  Smhd(SmhdAtom),
  Gmhd(GmhdAtom),
}

#[derive(Debug)]
pub struct VmhdAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub graphics_mode: u16,
  pub opcolor: [u16; 3],
}

impl AtomDecoder for VmhdAtom {
  const NAME: [u8; 4] = *b"vmhd";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let graphics_mode = u16::from_be_bytes((&data[4..6]).try_into()?);
    let opcolor = [
      u16::from_be_bytes((&data[6..8]).try_into()?),
      u16::from_be_bytes((&data[8..10]).try_into()?),
      u16::from_be_bytes((&data[10..12]).try_into()?),
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
pub struct SmhdAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub balance: u16,
}

impl AtomDecoder for SmhdAtom {
  const NAME: [u8; 4] = *b"smhd";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let balance = u16::from_be_bytes((&data[4..6]).try_into()?);

    Ok(Self {
      version,
      flags,
      balance,
    })
  }
}

#[derive(Debug, Default)]
pub struct GmhdAtom {
  pub gmin: EncodedAtom,
  pub text: EncodedAtom,
}

impl AtomDecoder for GmhdAtom {
  const NAME: [u8; 4] = *b"gmhd";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut gmhd = Self::default();
    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"gmin" => gmhd.gmin = EncodedAtom::Encoded(atom),
          b"text" => gmhd.text = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[gmhd] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[gmhd] {e}"),
      }
    }

    Ok(gmhd)
  }
}

#[derive(Debug, Default)]
pub struct DinfAtom {
  pub dref: EncodedAtom<DrefAtom>,
}

impl AtomDecoder for DinfAtom {
  const NAME: [u8; 4] = *b"dinf";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut dinf = Self::default();
    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"dref" => dinf.dref = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[dinf] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[dinf] {e}"),
      }
    }

    Ok(dinf)
  }
}

#[derive(Debug, Default)]
pub struct DrefAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub data_references: Vec<DrefItem>,
}

impl AtomDecoder for DrefAtom {
  const NAME: [u8; 4] = *b"dref";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    atom.offset += 8;
    let mut data_references = Vec::new();
    let mut atoms = atom.atoms(reader);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(mut atom) => {
          data_references.push(DrefItem::new(&atom.read_data(atoms.reader)?, atom.name)?)
        }
        Err(e) => log!(err@"#[dref] {e}"),
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

#[derive(Debug, Default)]
pub struct DrefItem {
  pub atom_type: Str<4>,
  pub version: u8,
  pub flags: [u8; 3],
  pub data: String,
}

impl DrefItem {
  pub fn new(data: &[u8], atom_type: Str<4>) -> AtomResult<Self> {
    let (version, flags) = decode_version_flags(data);
    let data = String::from_utf8_lossy(&data[4..]).to_string();

    Ok(Self {
      atom_type,
      version,
      flags,
      data,
    })
  }
}
