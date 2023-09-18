use super::*;
use crate::ascii::LogDisplay;
use crate::log;

#[derive(Debug)]
pub struct MinfAtom {
  pub mhd: MhdAtomKind,
  pub hdlr: EncodedAtom,
  pub dinf: EncodedAtom<DinfAtom>,
  pub stbl: EncodedAtom<StblAtom>,
}

impl AtomDecoder for MinfAtom {
  const NAME: [u8; 4] = *b"minf";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut mhd = None;
    let mut hdlr = EncodedAtom::Required;
    let mut dinf = EncodedAtom::Required;
    let mut stbl = EncodedAtom::Required;
    let mut atoms = atom.atoms(decoder);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"vmhd" => {
            mhd = Some(MhdAtomKind::Vmhd(VmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"smhd" => {
            mhd = Some(MhdAtomKind::Smhd(SmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"gmhd" => {
            mhd = Some(MhdAtomKind::Gmhd(GmhdAtom::decode_unchecked(
              atom,
              atoms.reader,
            )?))
          }
          b"hdlr" => hdlr = EncodedAtom::Encoded(atom),
          b"dinf" => dinf = EncodedAtom::Encoded(atom),
          b"stbl" => stbl = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[minf] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[minf] {e}"),
      }
    }

    Ok(Self {
      mhd: mhd.ok_or(AtomError::NoMinfHandler)?,
      hdlr,
      dinf,
      stbl,
    })
  }
}

#[derive(Debug)]
pub enum MhdAtomKind {
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
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      version: data.version(),
      flags: data.flags(),
      graphics_mode: data.next_into()?,
      opcolor: [data.next_into()?, data.next_into()?, data.next_into()?],
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
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      version: data.version(),
      flags: data.flags(),
      balance: data.next_into()?,
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
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut gmhd = Self::default();
    for atom in atom.atoms(decoder) {
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
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut dinf = Self::default();
    for atom in atom.atoms(decoder) {
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
  pub data_references: Box<[DrefItem]>,
}

impl AtomDecoder for DrefAtom {
  const NAME: [u8; 4] = *b"dref";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
      data_references: data
        .atoms()
        .filter_map(|atom| match atom {
          Ok((atom, data)) => Some(DrefItem::new(atom.name, AtomData::new(data, atom.offset))),
          Err(e) => {
            log!(err@"#[dref] {e}");
            None
          }
        })
        .collect::<AtomResult<_>>()?,
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
  pub fn new(atom_type: Str<4>, mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      atom_type,
      version: data.version(),
      flags: data.flags(),
      data: String::from_utf8_lossy(&data).to_string(),
    })
  }
}
