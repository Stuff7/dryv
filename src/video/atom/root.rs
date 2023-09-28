use super::*;
use crate::{ascii::LogDisplay, log};
use std::{
  fs::File,
  io::{Read, Seek},
};

#[derive(Debug)]
pub struct RootAtom {
  pub ftyp: FtypAtom,
  pub mdat: MdatAtom,
  pub moov: MoovAtom,
  pub rest: Box<[Atom]>,
}

impl RootAtom {
  pub fn new(reader: &mut File, size: u64) -> AtomResult<Self> {
    let mut ftyp = None;
    let mut mdat = None;
    let mut moov = None;
    let mut rest = Vec::new();
    let mut atoms = AtomIter::new(reader, 0, size);

    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"ftyp" => ftyp = Some(FtypAtom::new(atom, atoms.reader)?),
          b"mdat" => {
            mdat = {
              let mdat = MdatAtom::new(atom, atoms.reader)?;
              if mdat.extended_size > u32::MAX as u64 {
                atoms.start += mdat.extended_size + 7;
              }
              Some(mdat)
            }
          }
          b"moov" => moov = Some(MoovAtom::new(atom, atoms.reader)?),
          _ => rest.push(atom),
        },
        Err(e) => log!(err@"#[root] {e}"),
      }
    }

    Ok(Self {
      ftyp: ftyp.ok_or(AtomError::Required(*b"ftyp"))?,
      mdat: mdat.ok_or(AtomError::Required(*b"mdat"))?,
      moov: moov.ok_or(AtomError::Required(*b"moov"))?,
      rest: rest.into_boxed_slice(),
    })
  }
}

#[derive(Debug, Default)]
pub struct FtypAtom {
  pub atom: Atom,
  pub major_brand: Str<4>,
  pub minor_version: u32,
  pub compatible_brands: Box<[Str<4>]>,
}

impl FtypAtom {
  fn new<R: Read + Seek>(mut atom: Atom, reader: &mut R) -> AtomResult<Self> {
    let mut data = atom.read_data(reader)?;
    Ok(Self {
      atom,
      major_brand: data.next_into()?,
      minor_version: data.next_into()?,
      compatible_brands: data
        .chunks_exact(4)
        .map(Str::<4>::try_from)
        .collect::<Result<_, _>>()?,
    })
  }
}
