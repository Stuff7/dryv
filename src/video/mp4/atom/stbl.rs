use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::{Read, Seek};

#[derive(Debug)]
pub struct StblBox {
  pub vmhd: Option<VmhdBox>,
  pub hdlr: Option<HdlrBox>,
  pub dinf: Option<DinfBox>,
}

impl StblBox {
  pub fn new<R: Read + Seek>(reader: &mut R, offset: u32, size: u32) -> BoxResult<Self> {
    let mut atoms = AtomBoxIter::new(reader, offset + size);
    atoms.offset = offset;
    let mut vmhd = None;
    let mut hdlr = None;
    let mut dinf = None;
    for atom in atoms {
      match atom {
        Ok(atom) => match atom {
          AtomBox::Vmhd(atom) => vmhd = Some(atom),
          AtomBox::Hdlr(atom) => hdlr = Some(atom),
          AtomBox::Dinf(atom) => dinf = Some(atom),
          _ => log!(warn@"#[STBL] Misplaced atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[STBL] {e}"),
      }
    }

    Ok(Self { vmhd, hdlr, dinf })
  }
}
