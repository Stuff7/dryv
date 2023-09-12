use super::*;
use crate::ascii::LogDisplay;
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
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let creation_time = u32::from_be_bytes((&data[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&data[8..12]).try_into()?);
    let timescale = u32::from_be_bytes((&data[12..16]).try_into()?);
    let duration = u32::from_be_bytes((&data[16..20]).try_into()?);
    let language = Str(unpack_language_code(&data[20..22])?);
    let quality = u16::from_be_bytes((&data[22..24]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      creation_time,
      timescale,
      modification_time,
      duration,
      language,
      quality,
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
  pub component_name: String,
}

impl AtomDecoder for HdlrAtom {
  const NAME: [u8; 4] = *b"hdlr";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let component_type = Str::try_from(&data[4..8])?;
    let component_subtype = Str::try_from(&data[8..12])?;
    let component_manufacturer = Str::try_from(&data[12..16])?;
    let component_flags = (&data[16..20]).try_into()?;
    let component_flags_mask = (&data[20..24]).try_into()?;
    // 24th byte is the size of the string
    let component_name = String::from_utf8_lossy(&data[25..]).to_string();

    Ok(Self {
      atom,
      version,
      flags,
      component_type,
      component_subtype,
      component_manufacturer,
      component_flags,
      component_flags_mask,
      component_name,
    })
  }
}
