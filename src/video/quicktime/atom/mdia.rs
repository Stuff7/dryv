use super::*;
use crate::ascii::LogDisplay;
use crate::byte::{c_string, pascal_string};
use crate::log;

#[derive(Debug, Default)]
pub struct MdiaAtom {
  pub atom: Atom,
  pub mdhd: EncodedAtom<MdhdAtom>,
  pub hdlr: EncodedAtom<HdlrAtom>,
  pub minf: EncodedAtom<MinfAtom>,
}

impl AtomDecoder for MdiaAtom {
  const NAME: [u8; 4] = *b"mdia";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut mdia = Self {
      atom,
      ..Default::default()
    };
    for atom in mdia.atom.atoms(decoder) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"mdhd" => mdia.mdhd = EncodedAtom::Encoded(atom),
          b"hdlr" => mdia.hdlr = EncodedAtom::Encoded(atom),
          b"minf" => mdia.minf = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[mdia] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[mdia] {e}"),
      }
    }

    Ok(mdia)
  }
}

#[derive(Debug, Default)]
pub struct MdhdAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub timescale: u32,
  pub duration: u32,
  pub language: Str<3>,
  pub quality: u16,
}

impl AtomDecoder for MdhdAtom {
  const NAME: [u8; 4] = *b"mdhd";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      creation_time: data.next_into()?,
      modification_time: data.next_into()?,
      timescale: data.next_into()?,
      duration: data.next_into()?,
      language: Str(unpack_language_code(data.next(2))?),
      quality: data.next_into()?,
    })
  }
}

#[derive(Debug, Default)]
pub struct HdlrAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub component_type: Str<4>,
  pub component_subtype: Str<4>,
  pub component_manufacturer: Str<4>,
  pub component_flags: [u8; 4],
  pub component_flags_mask: [u8; 4],
  pub component_name: Box<str>,
}

impl AtomDecoder for HdlrAtom {
  const NAME: [u8; 4] = *b"hdlr";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    let component_manufacturer;

    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      component_type: data.next_into()?,
      component_subtype: data.next_into()?,
      component_manufacturer: {
        component_manufacturer = data.next_into()?;
        component_manufacturer
      },
      component_flags: data.next_into()?,
      component_flags_mask: data.next_into()?,
      component_name: match &data {
        slice if &*component_manufacturer == b"appl" => pascal_string(slice),
        slice => c_string(slice),
      },
    })
  }
}
