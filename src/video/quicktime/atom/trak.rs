use super::*;
use crate::log;
use crate::{ascii::LogDisplay, math::Matrix3x3};

#[derive(Debug, Default)]
pub struct TrakAtom {
  pub atom: Atom,
  pub prfl: EncodedAtom,
  pub tkhd: EncodedAtom<TkhdAtom>,
  pub tapt: EncodedAtom,
  pub clip: EncodedAtom,
  pub matt: EncodedAtom,
  pub edts: Option<EncodedAtom<EdtsAtom>>,
  pub tref: EncodedAtom,
  pub txas: EncodedAtom,
  pub load: EncodedAtom,
  pub imap: EncodedAtom,
  pub mdia: EncodedAtom<MdiaAtom>,
  pub udta: EncodedAtom,
}

impl AtomDecoder for TrakAtom {
  const NAME: [u8; 4] = *b"trak";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut trak = Self {
      atom,
      ..Default::default()
    };

    for atom in trak.atom.atoms(decoder) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"prfl" => trak.prfl = EncodedAtom::Encoded(atom),
          b"tkhd" => trak.tkhd = EncodedAtom::Encoded(atom),
          b"tapt" => trak.tapt = EncodedAtom::Encoded(atom),
          b"clip" => trak.clip = EncodedAtom::Encoded(atom),
          b"matt" => trak.matt = EncodedAtom::Encoded(atom),
          b"edts" => trak.edts = Some(EncodedAtom::Encoded(atom)),
          b"tref" => trak.tref = EncodedAtom::Encoded(atom),
          b"txas" => trak.txas = EncodedAtom::Encoded(atom),
          b"load" => trak.load = EncodedAtom::Encoded(atom),
          b"imap" => trak.imap = EncodedAtom::Encoded(atom),
          b"mdia" => trak.mdia = EncodedAtom::Encoded(atom),
          b"udta" => trak.udta = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[trak] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[trak] {e}"),
      }
    }

    Ok(trak)
  }
}

#[derive(Debug)]
pub struct TkhdAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub track_id: u32,
  pub duration: u32,
  pub layer: u16,
  pub alternate_group: u16,
  pub volume: f32,
  pub matrix: Matrix3x3,
  pub width: f32,
  pub height: f32,
}

impl AtomDecoder for TkhdAtom {
  const NAME: [u8; 4] = *b"tkhd";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      creation_time: data.next_into()?,
      modification_time: data.next_into()?,
      track_id: data.next_into()?,
      duration: data.reserved(4).next_into()?,
      layer: data.reserved(8).next_into()?,
      alternate_group: data.next_into()?,
      volume: data.fixed_point_8()?,
      matrix: data.reserved(2).next_into()?,
      width: data.fixed_point_16()?,
      height: data.fixed_point_16()?,
    })
  }
}
