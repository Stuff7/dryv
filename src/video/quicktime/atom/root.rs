use super::*;
use crate::{ascii::LogDisplay, log};
use std::io::{Read, Seek};

#[derive(Debug, Default)]
pub struct RootAtom {
  pub ftyp: FtypAtom,
  pub mdat: MdatAtom,
  pub moov: MoovAtom,
  pub rest: Vec<Atom>,
}

impl RootAtom {
  pub fn new<R: Read + Seek>(reader: &mut R, size: u32) -> AtomResult<Self> {
    let mut ftyp = None;
    let mut mdat = None;
    let mut moov = None;
    let mut rest = Vec::new();
    let mut atoms = AtomIter::new(reader, 0, size);

    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"ftyp" => ftyp = Some(FtypAtom::decode_unchecked(atom, atoms.reader)?),
          b"mdat" => mdat = Some(MdatAtom::decode_unchecked(atom, atoms.reader)?),
          b"moov" => moov = Some(MoovAtom::decode_unchecked(atom, atoms.reader)?),
          _ => rest.push(atom),
        },
        Err(e) => log!(err@"#[root] {e}"),
      }
    }

    Ok(Self {
      ftyp: ftyp.ok_or(AtomError::AtomNotFound(*b"ftyp"))?,
      mdat: mdat.ok_or(AtomError::AtomNotFound(*b"mdat"))?,
      moov: moov.ok_or(AtomError::AtomNotFound(*b"moov"))?,
      rest,
    })
  }
}

#[derive(Debug, Default)]
pub struct FtypAtom {
  pub atom: Atom,
  pub compatible_brands: Vec<Str<4>>,
  pub major_brand: Str<4>,
  pub minor_version: u32,
}

impl AtomDecoder for FtypAtom {
  const NAME: [u8; 4] = *b"ftyp";
  fn decode_unchecked<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let data = atom.read_data(reader)?;

    let major_brand = Str::try_from(&data[..4])?;
    let minor_version = u32::from_be_bytes((&data[4..8]).try_into()?);
    let compatible_brands: Vec<Str<4>> = data[8..]
      .chunks_exact(4)
      .map(Str::<4>::try_from)
      .collect::<Result<_, _>>()?;

    Ok(Self {
      atom,
      compatible_brands,
      major_brand,
      minor_version,
    })
  }
}
