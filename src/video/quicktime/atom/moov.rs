use super::*;
use crate::log;
use crate::math::fixed_point_to_f32;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct MoovAtom {
  pub atom: Atom,
  pub prfl: EncodedAtom,
  pub mvhd: EncodedAtom<MvhdAtom>,
  pub clip: EncodedAtom,
  pub trak: Vec<EncodedAtom<TrakAtom>>,
  pub udta: Option<EncodedAtom<UdtaAtom>>,
  pub ctab: EncodedAtom,
  pub cmov: EncodedAtom,
  pub rmra: EncodedAtom,
  pub meta: Option<EncodedAtom<MetaAtom>>,
}

impl AtomDecoder for MoovAtom {
  const NAME: [u8; 4] = *b"moov";
  fn decode_unchecked<R: Read + Seek>(atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut moov = Self {
      atom,
      ..Default::default()
    };
    for atom in moov.atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"prfl" => moov.prfl = EncodedAtom::Encoded(atom),
          b"mvhd" => moov.mvhd = EncodedAtom::Encoded(atom),
          b"clip" => moov.clip = EncodedAtom::Encoded(atom),
          b"trak" => moov.trak.push(EncodedAtom::Encoded(atom)),
          b"udta" => moov.udta = Some(EncodedAtom::Encoded(atom)),
          b"ctab" => moov.ctab = EncodedAtom::Encoded(atom),
          b"cmov" => moov.cmov = EncodedAtom::Encoded(atom),
          b"rmra" => moov.rmra = EncodedAtom::Encoded(atom),
          b"meta" => moov.meta = Some(EncodedAtom::Encoded(atom)),
          _ => log!(warn@"#[moov] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[moov] {e}"),
      }
    }

    Ok(moov)
  }
}

#[derive(Debug)]
pub struct MvhdAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub creation_time: u32,
  pub modification_time: u32,
  pub timescale: u32,
  pub duration: u32,
  pub rate: f32,
  pub volume: f32,
  pub matrix: Matrix3x3,
  pub next_track_id: u32,
}

impl AtomDecoder for MvhdAtom {
  const NAME: [u8; 4] = *b"mvhd";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let creation_time = u32::from_be_bytes((&data[4..8]).try_into()?);
    let modification_time = u32::from_be_bytes((&data[8..12]).try_into()?);
    let timescale = u32::from_be_bytes((&data[12..16]).try_into()?);
    let duration = u32::from_be_bytes((&data[16..20]).try_into()?);
    let rate = fixed_point_to_f32(i32::from_be_bytes((&data[20..24]).try_into()?) as f32, 16);
    let volume = fixed_point_to_f32(i16::from_be_bytes((&data[24..26]).try_into()?) as f32, 8);
    // __reserved__    16 bit     (2 bytes)
    // __reserved__    32 bit [2] (8 bytes)
    let matrix = Matrix3x3::from_bytes(&data[36..])?;
    // __pre_defined__ 32 bit [6] (24 bytes)
    let next_track_id = u32::from_be_bytes((&data[96..100]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      creation_time,
      modification_time,
      timescale,
      duration,
      rate,
      volume,
      matrix,
      next_track_id,
    })
  }
}

#[derive(Debug)]
pub struct UdtaAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub metas: Vec<EncodedAtom<MetaAtom>>,
}

impl AtomDecoder for UdtaAtom {
  const NAME: [u8; 4] = *b"udta";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let (version, flags) = decode_version_flags(&data);
    let mut metas = Vec::new();

    for atom in atom.atoms(reader) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"meta" => metas.push(EncodedAtom::Encoded(atom)),
          _ => log!(warn@"#[udta] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[udta] {e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      metas,
    })
  }
}
