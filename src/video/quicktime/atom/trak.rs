use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct TrakAtom {
  pub atom: Atom,
  pub prfl: EncodedAtom,
  pub tkhd: EncodedAtom<TkhdAtom>,
  pub tapt: EncodedAtom,
  pub clip: EncodedAtom,
  pub matt: EncodedAtom,
  pub edts: EncodedAtom<EdtsAtom>,
  pub tref: EncodedAtom,
  pub txas: EncodedAtom,
  pub load: EncodedAtom,
  pub imap: EncodedAtom,
  pub mdia: EncodedAtom<MdiaAtom>,
  pub udta: EncodedAtom,
}

impl AtomDecoder for TrakAtom {
  const NAME: [u8; 4] = *b"trak";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut trak = Self {
      atom,
      ..Default::default()
    };

    for atom in trak.atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"prfl" => trak.prfl = EncodedAtom::Encoded(atom),
          b"tkhd" => trak.tkhd = EncodedAtom::Encoded(atom),
          b"tapt" => trak.tapt = EncodedAtom::Encoded(atom),
          b"clip" => trak.clip = EncodedAtom::Encoded(atom),
          b"matt" => trak.matt = EncodedAtom::Encoded(atom),
          b"edts" => trak.edts = EncodedAtom::Encoded(atom),
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
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let creation_time = u32::from_be_bytes((&data[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&data[8..12]).try_into()?);
    let track_id = u32::from_be_bytes((&data[12..16]).try_into()?);
    // __reserved__ 32 bit     (4 bytes)
    let duration = u32::from_be_bytes((&data[20..24]).try_into()?);
    // __reserved__ 32 bit [2] (8 bytes)
    let layer = u16::from_be_bytes((&data[32..34]).try_into()?);
    let alternate_group = u16::from_be_bytes((&data[34..36]).try_into()?);
    let volume = fixed_point_to_f32(i16::from_be_bytes((&data[36..38]).try_into()?) as f32, 8);
    // __reserved__ 16 bit     (2 bytes)
    let matrix = Matrix3x3::from_bytes(&data[40..])?;
    let width = fixed_point_to_f32(i32::from_be_bytes((&data[76..80]).try_into()?) as f32, 16);
    let height = fixed_point_to_f32(i32::from_be_bytes((&data[80..84]).try_into()?) as f32, 16);

    Ok(Self {
      atom,
      version,
      flags,
      creation_time,
      modification_time,
      track_id,
      duration,
      layer,
      alternate_group,
      volume,
      matrix,
      width,
      height,
    })
  }
}
