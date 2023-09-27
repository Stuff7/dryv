mod pps;
mod sps;
mod vui;

pub use pps::*;
pub use sps::*;
pub use vui::*;

use super::*;
use crate::byte::BitData;

#[derive(Debug)]
pub struct AvcCAtom {
  pub configuration_version: u8,
  pub profile_indication: u8,
  pub profile_compatibility: u8,
  pub level_indication: u8,
  pub nal_length_size_minus_one: u8,
  pub num_sps: u8,
  pub sps: SequenceParameterSet,
  pub num_pps: u8,
  pub pps: PictureParameterSet,
}

impl AvcCAtom {
  pub const TYPE: [u8; 4] = *b"avcC";
  pub fn decode(mut data: AtomData) -> Self {
    let mut bit_data;
    let chroma_format_idc;
    Self {
      configuration_version: data.byte(),
      profile_indication: data.byte(),
      profile_compatibility: data.byte(),
      level_indication: data.byte(),
      nal_length_size_minus_one: data.byte() & 0b0000_0011,
      num_sps: data.byte() & 0b0001_1111,
      sps: {
        bit_data = BitData::new(&data);
        let sps = SequenceParameterSet::decode(&mut bit_data);
        chroma_format_idc = sps.chroma_format_idc;
        sps
      },
      num_pps: bit_data.byte(),
      pps: PictureParameterSet::decode(&mut bit_data, chroma_format_idc),
    }
  }
}
