pub mod header;

use std::ops::Deref;

use crate::{
  byte::BitData,
  video::atom::{PictureParameterSet, SequenceParameterSet},
  video::sample::NALUnit,
};
use header::*;

#[derive(Debug)]
pub struct Slice<'a> {
  pub header: SliceHeader,
  pub sps: &'a SequenceParameterSet,
  pub pps: &'a PictureParameterSet,
  pub stream: BitData<'a>,
  pub sliceqpy: i16,
  pub cabac_init_mode: usize,
}

impl<'a> Slice<'a> {
  pub fn new(
    data: &'a [u8],
    nal: &NALUnit,
    sps: &'a SequenceParameterSet,
    pps: &'a PictureParameterSet,
  ) -> Self {
    let mut stream = BitData::new(data);
    let header = SliceHeader::new(&mut stream, nal, sps, pps);
    Self {
      sliceqpy: 26 + pps.pic_init_qp_minus26 + header.slice_qp_delta,
      cabac_init_mode: header.cabac_init_idc.map(|idc| idc + 1).unwrap_or(0) as usize,
      header,
      sps,
      pps,
      stream,
    }
  }
}

impl<'a> Deref for Slice<'a> {
  type Target = SliceHeader;
  fn deref(&self) -> &Self::Target {
    &self.header
  }
}
