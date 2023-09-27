mod header;

pub use header::*;

use super::NALUnit;
use crate::{
  byte::BitData,
  video::quicktime::atom::{PictureParameterSet, SequenceParameterSet},
};

#[derive(Debug)]
pub struct Slice {
  pub header: SliceHeader,
}

impl Slice {
  pub fn new(
    data: &mut BitData,
    nal: &NALUnit,
    sps: &SequenceParameterSet,
    pps: &PictureParameterSet,
  ) -> Self {
    Self {
      header: SliceHeader::new(data, nal, sps, pps),
    }
  }
}
