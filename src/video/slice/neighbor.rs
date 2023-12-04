use std::ops::Deref;

use super::{
  consts::{MB_TYPE_I_PCM, MB_UNAVAILABLE_INTER},
  macroblock::{Macroblock, MbPosition, SubMbType},
  Slice,
};
use crate::{log, math::inverse_raster_scan};

impl<'a> Slice<'a> {
  /// 6.4.11.7 Derivation process for neighbouring partitions
  pub fn neighboring_partitions(&'a self, mb_part_idx: usize, sub_mb_part_idx: usize, curr_sub_mb_type: SubMbType) -> MbNeighbors<'a> {
    let mb_part_width = self.mb().mb_part_width();
    let mb_part_height = self.mb().mb_part_height();
    let sub_mb_part_width = self.mb().sub_mb_type[mb_part_idx].sub_mb_part_width as isize;
    let sub_mb_part_height = self.mb().sub_mb_type[mb_part_idx].sub_mb_part_height as isize;

    let x = inverse_raster_scan(mb_part_idx as isize, mb_part_width, mb_part_height, 16, 0);
    let y = inverse_raster_scan(mb_part_idx as isize, mb_part_width, mb_part_height, 16, 1);

    let x_s;
    let y_s;
    if self.mb().mb_type.is_p8x8() || self.mb().mb_type.is_p_8x8ref0() || self.mb().mb_type.is_b8x8() {
      x_s = inverse_raster_scan(sub_mb_part_idx as isize, sub_mb_part_width, sub_mb_part_height, 8, 0);
      y_s = inverse_raster_scan(sub_mb_part_idx as isize, sub_mb_part_width, sub_mb_part_height, 8, 1);
    } else {
      x_s = 0;
      y_s = 0;
    }

    let pred_part_width;
    if self.mb().mb_type.is_p_skip() || self.mb().mb_type.is_b_skip() || self.mb().mb_type.is_b_direct_16x16() {
      pred_part_width = 16;
    } else if self.mb().mb_type.is_b8x8() {
      if curr_sub_mb_type.is_b_direct8x8() {
        pred_part_width = 16;
      } else {
        pred_part_width = sub_mb_part_width;
      }
    } else if self.mb().mb_type.is_p8x8() || self.mb().mb_type.is_p_8x8ref0() {
      pred_part_width = sub_mb_part_width;
    } else {
      pred_part_width = mb_part_width;
    }

    let mut neighbors = MbNeighbors::new();
    neighbors.a.mb = MbPosition::from_coords(x + x_s - 1, y + y_s, 16, 16)
      .map(|pos| self.mb_nb_p(pos, 0))
      .unwrap_or(&MB_UNAVAILABLE_INTER);
    let (x_w, y_w) = MbPosition::coords(x + x_s - 1, y + y_s, 16, 16);

    if neighbors.a.is_available() {
      self.mb_and_submb_partition_indices(&mut neighbors.a, x_w, y_w);
    }

    neighbors.b.mb = MbPosition::from_coords(x + x_s, y + y_s - 1, 16, 16)
      .map(|pos| self.mb_nb_p(pos, 0))
      .unwrap_or(&MB_UNAVAILABLE_INTER);
    let (x_w, y_w) = MbPosition::coords(x + x_s, y + y_s - 1, 16, 16);

    if neighbors.b.is_available() {
      self.mb_and_submb_partition_indices(&mut neighbors.b, x_w, y_w);
    }

    neighbors.c.mb = MbPosition::from_coords(x + x_s + pred_part_width, y + y_s - 1, 16, 16)
      .map(|pos| self.mb_nb_p(pos, 0))
      .unwrap_or(&MB_UNAVAILABLE_INTER);
    let (x_w, y_w) = MbPosition::coords(x + x_s + pred_part_width, y + y_s - 1, 16, 16);

    if neighbors.c.is_available() {
      self.mb_and_submb_partition_indices(&mut neighbors.c, x_w, y_w);
    }

    neighbors.d.mb = MbPosition::from_coords(x + x_s - 1, y + y_s - 1, 16, 16)
      .map(|pos| self.mb_nb_p(pos, 0))
      .unwrap_or(&MB_UNAVAILABLE_INTER);
    let (x_w, y_w) = MbPosition::coords(x + x_s - 1, y + y_s - 1, 16, 16);

    if neighbors.d.is_available() {
      self.mb_and_submb_partition_indices(&mut neighbors.d, x_w, y_w);
    }

    neighbors
  }

  /// 6.4.13.4 Derivation process for macroblock and sub-macroblock partition indices
  pub fn mb_and_submb_partition_indices(&self, mb: &mut MbNeighbor, x_w: isize, y_w: isize) {
    let mb_type = &mb.mb.mb_type;

    if **mb_type <= MB_TYPE_I_PCM {
      mb.mb_part_idx = 0;
    } else {
      let mb_part_width = mb.mb_part_width();
      let mb_part_height = mb.mb_part_height();
      mb.mb_part_idx = (16 / mb_part_width) * (y_w / mb_part_height) + (x_w / mb_part_width);
    }

    if !mb_type.is_p8x8() && !mb_type.is_p_8x8ref0() && !mb_type.is_b8x8() && !mb_type.is_b_skip() && !mb_type.is_b_direct_16x16() {
      mb.sub_mb_part_idx = 0;
    } else if mb_type.is_b_skip() || mb_type.is_b_direct_16x16() {
      mb.sub_mb_part_idx = 2 * ((y_w % 8) / 4) + ((x_w % 8) / 4);
    } else {
      let mb_part_width = mb.sub_mb_type[mb.mb_part_idx as usize].sub_mb_part_width as isize;
      let mb_part_height = mb.sub_mb_type[mb.mb_part_idx as usize].sub_mb_part_height as isize;
      mb.sub_mb_part_idx = (8 / mb_part_width) * ((y_w % 8) / mb_part_height) + ((x_w % 8) / mb_part_width);
    }
  }
}

pub struct MbNeighbors<'a> {
  pub a: MbNeighbor<'a>,
  pub b: MbNeighbor<'a>,
  pub c: MbNeighbor<'a>,
  pub d: MbNeighbor<'a>,
}

impl<'a> MbNeighbors<'a> {
  pub fn new() -> Self {
    Self {
      a: MbNeighbor::new(&MB_UNAVAILABLE_INTER),
      b: MbNeighbor::new(&MB_UNAVAILABLE_INTER),
      c: MbNeighbor::new(&MB_UNAVAILABLE_INTER),
      d: MbNeighbor::new(&MB_UNAVAILABLE_INTER),
    }
  }
}

pub struct MbNeighbor<'a> {
  pub mb: &'a Macroblock,
  pub mb_part_idx: isize,
  pub sub_mb_part_idx: isize,
}

impl<'a> MbNeighbor<'a> {
  pub fn new(mb: &'a Macroblock) -> Self {
    Self {
      mb,
      mb_part_idx: -1,
      sub_mb_part_idx: -1,
    }
  }
}

impl<'a> Deref for MbNeighbor<'a> {
  type Target = &'a Macroblock;

  fn deref(&self) -> &Self::Target {
    &self.mb
  }
}
