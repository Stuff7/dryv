use super::*;
use crate::log;
use crate::{ascii::LogDisplay, math::Matrix3x3};
use std::fs::File;

#[derive(Debug)]
pub struct MoovAtom {
  pub atom: Atom,
  pub mvhd: EncodedAtom<MvhdAtom>,
  pub trak: Box<[EncodedAtom<TrakAtom>]>,
  pub udta: Option<EncodedAtom<UdtaAtom>>,
  pub meta: Option<MetaAtom>,
}

impl MoovAtom {
  pub fn new(atom: Atom, reader: &mut File) -> AtomResult<Self> {
    let (mut mvhd, mut udta, mut meta) = (EncodedAtom::Required, None, None);
    let mut reader_clone = reader.try_clone()?;
    let trak = atom
      .atoms(&mut reader_clone)
      .filter_map(|atom| {
        match atom {
          Ok(mut atom) => match &*atom.name {
            b"mvhd" => mvhd = EncodedAtom::Encoded(atom),
            b"trak" => return Some(EncodedAtom::<TrakAtom>::Encoded(atom)),
            b"udta" => udta = Some(EncodedAtom::Encoded(atom)),
            b"meta" => {
              meta = Some(
                atom
                  .read_data(reader)
                  .and_then(|data| MetaAtom::new(atom, data)),
              )
            }
            _ => log!(warn@"#[moov] Unused atom {atom:#?}"),
          },
          Err(e) => log!(err@"#[moov] {e}"),
        }
        None
      })
      .collect();

    Ok(Self {
      atom,
      mvhd,
      trak,
      udta,
      meta: meta.transpose()?,
    })
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
      rate: data.fixed_point_16()?,
      volume: data.fixed_point_8()?,
      matrix: data.reserved(2).reserved(8).next_into()?,
      next_track_id: data.reserved(24).next_into()?,
    })
  }
}

#[derive(Debug, Default)]
pub struct UdtaAtom {
  pub metas: Box<[MetaAtom]>,
}

impl AtomDecoder for UdtaAtom {
  const NAME: [u8; 4] = *b"udta";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;
    Ok(Self {
      metas: data
        .atoms()
        .filter_map(|atom| {
          match atom {
            Ok((atom, data)) => match &*atom.name {
              b"meta" => {
                let mut data = AtomData::new(data, atom.offset);
                if decoder.brand.is_isom() {
                  data.reserved(4);
                }
                return Some(MetaAtom::new(atom, data));
              }
              _ => log!(warn@"#[udta] Unused atom {atom:#?}"),
            },
            Err(e) => log!(err@"#[udta] {e}"),
          }
          None
        })
        .collect::<AtomResult<_>>()?,
    })
  }
}
