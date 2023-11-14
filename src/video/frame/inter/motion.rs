use crate::{
  math::median,
  video::slice::{macroblock::SubMbType, neighbor::MbNeighbors, Slice},
};

use super::super::Frame;

impl Frame {
  /// 8.4.1.3.2 Derivation process for motion data of neighbouring partitions
  pub fn motion_data_of_neighboring_partitions<'a>(
    &mut self,
    slice: &'a Slice,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    curr_sub_mb_type: SubMbType,
    list_suffix_flag: bool,
    mv_lxa: &mut [isize; 2],
    mv_lxb: &mut [isize; 2],
    mv_lxc: &mut [isize; 2],
    ref_idx_lxa: &mut isize,
    ref_idx_lxb: &mut isize,
    ref_idx_lxc: &mut isize,
  ) -> MbNeighbors<'a> {
    let mut nb = slice.neighboring_partitions(mb_part_idx, sub_mb_part_idx, curr_sub_mb_type);
    if nb.c.is_unavailable() && nb.c.mb_part_idx == -1 && nb.c.sub_mb_part_idx == -1 {
      nb.c.mb = nb.d.mb;
      nb.c.mb_part_idx = nb.d.mb_part_idx;
      nb.c.sub_mb_part_idx = nb.d.sub_mb_part_idx;
    }

    if nb.a.is_unavailable()
      || nb.a.mode().is_inter_mode()
      || (!list_suffix_flag && nb.a.pred_flagl0[nb.a.mb_part_idx as usize] == 0)
      || (list_suffix_flag && nb.a.pred_flagl1[nb.a.mb_part_idx as usize] == 0)
    {
      mv_lxa[0] = 0;
      mv_lxa[1] = 0;
      *ref_idx_lxa = -1;
    } else if list_suffix_flag {
      mv_lxa[0] = nb.a.mv_l1[nb.a.mb_part_idx as usize][nb.a.sub_mb_part_idx as usize][0];
      mv_lxa[1] = nb.a.mv_l1[nb.a.mb_part_idx as usize][nb.a.sub_mb_part_idx as usize][1];
      *ref_idx_lxa = nb.a.ref_idxl1[nb.a.mb_part_idx as usize];
    } else {
      mv_lxa[0] = nb.a.mv_l0[nb.a.mb_part_idx as usize][nb.a.sub_mb_part_idx as usize][0];
      mv_lxa[1] = nb.a.mv_l0[nb.a.mb_part_idx as usize][nb.a.sub_mb_part_idx as usize][1];
      *ref_idx_lxa = nb.a.ref_idxl0[nb.a.mb_part_idx as usize];
    }

    if nb.b.is_unavailable()
      || nb.b.mode().is_inter_mode()
      || (!list_suffix_flag && nb.b.pred_flagl0[nb.b.mb_part_idx as usize] == 0)
      || (list_suffix_flag && nb.b.pred_flagl1[nb.b.mb_part_idx as usize] == 0)
    {
      mv_lxb[0] = 0;
      mv_lxb[1] = 0;
      *ref_idx_lxb = -1;
    } else if list_suffix_flag {
      mv_lxb[0] = nb.b.mv_l1[nb.b.mb_part_idx as usize][nb.b.sub_mb_part_idx as usize][0];
      mv_lxb[1] = nb.b.mv_l1[nb.b.mb_part_idx as usize][nb.b.sub_mb_part_idx as usize][1];
      *ref_idx_lxb = nb.b.ref_idxl1[nb.b.mb_part_idx as usize];
    } else {
      mv_lxb[0] = nb.b.mv_l0[nb.b.mb_part_idx as usize][nb.b.sub_mb_part_idx as usize][0];
      mv_lxb[1] = nb.b.mv_l0[nb.b.mb_part_idx as usize][nb.b.sub_mb_part_idx as usize][1];
      *ref_idx_lxb = nb.b.ref_idxl0[nb.b.mb_part_idx as usize];
    }

    if nb.c.is_unavailable()
      || nb.c.mode().is_inter_mode()
      || (!list_suffix_flag && nb.c.pred_flagl0[nb.c.mb_part_idx as usize] == 0)
      || (list_suffix_flag && nb.c.pred_flagl1[nb.c.mb_part_idx as usize] == 0)
    {
      mv_lxc[0] = 0;
      mv_lxc[1] = 0;
      *ref_idx_lxc = -1;
    } else if list_suffix_flag {
      mv_lxc[0] = nb.c.mv_l1[nb.c.mb_part_idx as usize][nb.c.sub_mb_part_idx as usize][0];
      mv_lxc[1] = nb.c.mv_l1[nb.c.mb_part_idx as usize][nb.c.sub_mb_part_idx as usize][1];
      *ref_idx_lxc = nb.c.ref_idxl1[nb.c.mb_part_idx as usize];
    } else {
      mv_lxc[0] = nb.c.mv_l0[nb.c.mb_part_idx as usize][nb.c.sub_mb_part_idx as usize][0];
      mv_lxc[1] = nb.c.mv_l0[nb.c.mb_part_idx as usize][nb.c.sub_mb_part_idx as usize][1];
      *ref_idx_lxc = nb.c.ref_idxl0[nb.c.mb_part_idx as usize];
    }
    nb
  }

  /// 8.4.1.3 Derivation process for luma motion vector prediction
  pub fn luma_motion_vector_prediction<'a>(
    &'a mut self,
    slice: &'a Slice,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    curr_sub_mb_type: SubMbType,
    list_suffix_flag: bool,
    ref_idx_lx: isize,
    mvp_lx: &mut [isize; 2],
  ) -> MbNeighbors<'a> {
    let mut mv_lxa = [0; 2];
    let mut mv_lxb = [0; 2];
    let mut mv_lxc = [0; 2];
    let mut ref_idx_lxa = 0;
    let mut ref_idx_lxb = 0;
    let mut ref_idx_lxc = 0;

    let mb_part_width = slice.mb().mb_part_width();
    let mb_part_height = slice.mb().mb_part_height();

    let nb = self.motion_data_of_neighboring_partitions(
      slice,
      mb_part_idx,
      sub_mb_part_idx,
      curr_sub_mb_type,
      list_suffix_flag,
      &mut mv_lxa,
      &mut mv_lxb,
      &mut mv_lxc,
      &mut ref_idx_lxa,
      &mut ref_idx_lxb,
      &mut ref_idx_lxc,
    );

    if mb_part_width == 16 && mb_part_height == 8 && mb_part_idx == 0 && ref_idx_lxb == ref_idx_lx {
      mvp_lx[0] = mv_lxb[0];
      mvp_lx[1] = mv_lxb[1];
    } else if (mb_part_width == 16
      && mb_part_height == 8
      && mb_part_idx == 1
      && ref_idx_lxa == ref_idx_lx)
      || (mb_part_width == 8
        && mb_part_height == 16
        && mb_part_idx == 0
        && ref_idx_lxa == ref_idx_lx)
    {
      mvp_lx[0] = mv_lxa[0];
      mvp_lx[1] = mv_lxa[1];
    } else if mb_part_width == 8
      && mb_part_height == 16
      && mb_part_idx == 1
      && ref_idx_lxc == ref_idx_lx
    {
      mvp_lx[0] = mv_lxc[0];
      mvp_lx[1] = mv_lxc[1];
    } else {
      if nb.b.is_unavailable() && nb.c.is_unavailable() && nb.a.is_available() {
        mv_lxb[0] = mv_lxa[0];
        mv_lxb[1] = mv_lxa[1];
        mv_lxc[0] = mv_lxa[0];
        mv_lxc[1] = mv_lxa[1];
        ref_idx_lxb = ref_idx_lxa;
        ref_idx_lxc = ref_idx_lxa;
      }

      if ref_idx_lxa == ref_idx_lx && ref_idx_lxb != ref_idx_lx && ref_idx_lxc != ref_idx_lx {
        mvp_lx[0] = mv_lxa[0];
        mvp_lx[1] = mv_lxa[1];
      } else if ref_idx_lxa != ref_idx_lx && ref_idx_lxb == ref_idx_lx && ref_idx_lxc != ref_idx_lx
      {
        mvp_lx[0] = mv_lxb[0];
        mvp_lx[1] = mv_lxb[1];
      } else if ref_idx_lxa != ref_idx_lx && ref_idx_lxb != ref_idx_lx && ref_idx_lxc == ref_idx_lx
      {
        mvp_lx[0] = mv_lxc[0];
        mvp_lx[1] = mv_lxc[1];
      } else {
        mvp_lx[0] = median(mv_lxa[0], mv_lxb[0], mv_lxc[0]);
        mvp_lx[1] = median(mv_lxa[1], mv_lxb[1], mv_lxc[1]);
      }
    }
    nb
  }
}
