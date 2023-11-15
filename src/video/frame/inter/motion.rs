use crate::{
  math::{clamp, inverse_raster_scan, median, min_positive},
  video::slice::{
    dpb::{DecodedPictureBuffer, Picture},
    macroblock::SubMbType,
    neighbor::{MbNeighbor, MbNeighbors},
    Slice,
  },
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

  /// 8.4.1.2 Derivation process for luma motion vectors for B_Skip, B_Direct_16x16, and B_Direct_8x8
  pub fn luma_motion_vectors_for_b_blocks(
    &mut self,
    slice: &Slice,
    dpb: &DecodedPictureBuffer,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    ref_idxl0: &mut isize,
    ref_idxl1: &mut isize,
    mv_l0: &mut [isize; 2],
    mv_l1: &mut [isize; 2],
    pred_flagl0: &mut isize,
    pred_flagl1: &mut isize,
    sub_mv_cnt: &mut isize,
  ) {
    if slice.direct_spatial_mv_pred_flag {
      self.spatial_direct_luma_motion_vector_and_ref_idx_pred_mode(
        slice,
        dpb,
        mb_part_idx,
        sub_mb_part_idx,
        ref_idxl0,
        ref_idxl1,
        mv_l0,
        mv_l1,
        pred_flagl0,
        pred_flagl1,
        sub_mv_cnt,
      );
    } else {
      self.temporal_direct_luma_mv_and_ref_idx_pred_mode(
        slice,
        dpb,
        mb_part_idx,
        sub_mb_part_idx,
        ref_idxl0,
        ref_idxl1,
        mv_l0,
        mv_l1,
        pred_flagl0,
        pred_flagl1,
      );

      if sub_mb_part_idx == 0 {
        *sub_mv_cnt = 2;
      } else {
        *sub_mv_cnt = 0;
      }
    }
  }

  /// 8.4.1.2.2 Derivation process for spatial direct luma motion vector and reference index prediction mode
  pub fn spatial_direct_luma_motion_vector_and_ref_idx_pred_mode(
    &mut self,
    slice: &Slice,
    dpb: &DecodedPictureBuffer,
    mut mb_part_idx: usize,
    mut sub_mb_part_idx: usize,
    ref_idxl0: &mut isize,
    ref_idxl1: &mut isize,
    mv_l0: &mut [isize; 2],
    mv_l1: &mut [isize; 2],
    pred_flagl0: &mut isize,
    pred_flagl1: &mut isize,
    sub_mv_cnt: &mut isize,
  ) {
    let curr_sub_mb_type = slice.mb().sub_mb_type[mb_part_idx];
    let mut mv_l0_a = [0; 2];
    let mut mv_l0_b = [0; 2];
    let mut mv_l0_c = [0; 2];
    let mut ref_idx_l0a = 0;
    let mut ref_idx_l0b = 0;
    let mut ref_idx_l0c = 0;

    self.motion_data_of_neighboring_partitions(
      slice,
      mb_part_idx,
      sub_mb_part_idx,
      curr_sub_mb_type,
      false,
      &mut mv_l0_a,
      &mut mv_l0_b,
      &mut mv_l0_c,
      &mut ref_idx_l0a,
      &mut ref_idx_l0b,
      &mut ref_idx_l0c,
    );

    let mut mv_l1a = [0; 2];
    let mut mv_l1b = [0; 2];
    let mut mv_l1c = [0; 2];
    let mut ref_idx_l1a = 0;
    let mut ref_idx_l1b = 0;
    let mut ref_idx_l1c = 0;

    self.motion_data_of_neighboring_partitions(
      slice,
      mb_part_idx,
      sub_mb_part_idx,
      curr_sub_mb_type,
      true,
      &mut mv_l1a,
      &mut mv_l1b,
      &mut mv_l1c,
      &mut ref_idx_l1a,
      &mut ref_idx_l1b,
      &mut ref_idx_l1c,
    );

    *ref_idxl0 = min_positive(ref_idx_l0a, min_positive(ref_idx_l0b, ref_idx_l0c));
    *ref_idxl1 = min_positive(ref_idx_l1a, min_positive(ref_idx_l1b, ref_idx_l1c));

    let mut direct_zero_prediction_flag = false;
    if *ref_idxl0 < 0 && *ref_idxl1 < 0 {
      *ref_idxl0 = 0;
      *ref_idxl1 = 0;
      direct_zero_prediction_flag = true;
    }

    let mut ref_idx_col = 0;
    let mut mv_col = [0; 2];

    self.co_located_4x4submb_partitions(
      slice,
      dpb,
      mb_part_idx,
      sub_mb_part_idx,
      &mut mv_col,
      &mut ref_idx_col,
    );

    let col_zero_flag = dpb
      .ref_pic_list1(0)
      .reference_marked_type
      .is_short_term_reference()
      && ref_idx_col == 0
      && (mv_col[0] >= -1 && mv_col[0] <= 1)
      && (mv_col[1] >= -1 && mv_col[1] <= 1);

    if direct_zero_prediction_flag || *ref_idxl0 < 0 || (*ref_idxl0 == 0 && col_zero_flag) {
      mv_l0[0] = 0;
      mv_l0[1] = 0;
    } else {
      mb_part_idx = 0;
      sub_mb_part_idx = 0;
      self.luma_motion_vector_prediction(
        slice,
        mb_part_idx,
        sub_mb_part_idx,
        curr_sub_mb_type,
        false,
        *ref_idxl0,
        mv_l0,
      );
    }

    if direct_zero_prediction_flag || *ref_idxl1 < 0 || (*ref_idxl1 == 0 && col_zero_flag) {
      mv_l1[0] = 0;
      mv_l1[1] = 0;
    } else {
      mb_part_idx = 0;
      sub_mb_part_idx = 0;
      self.luma_motion_vector_prediction(
        slice,
        mb_part_idx,
        sub_mb_part_idx,
        curr_sub_mb_type,
        true,
        *ref_idxl1,
        mv_l1,
      );
    }

    if *ref_idxl0 >= 0 && *ref_idxl1 >= 0 {
      *pred_flagl0 = 1;
      *pred_flagl1 = 1;
    } else if *ref_idxl0 >= 0 && *ref_idxl1 < 0 {
      *pred_flagl0 = 1;
      *pred_flagl1 = 0;
    } else if *ref_idxl0 < 0 && *ref_idxl1 >= 0 {
      *pred_flagl0 = 0;
      *pred_flagl1 = 1;
    }

    if sub_mb_part_idx != 0 {
      *sub_mv_cnt = 0;
    } else {
      *sub_mv_cnt = *pred_flagl0 + *pred_flagl1;
    }
  }

  /// 8.4.1.2.3 Derivation process for temporal direct luma motion vector and reference index prediction mode
  pub fn temporal_direct_luma_mv_and_ref_idx_pred_mode(
    &mut self,
    slice: &Slice,
    dpb: &DecodedPictureBuffer,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    ref_idxl0: &mut isize,
    ref_idxl1: &mut isize,
    mv_l0: &mut [isize; 2],
    mv_l1: &mut [isize; 2],
    pred_flagl0: &mut isize,
    pred_flagl1: &mut isize,
  ) {
    let mut ref_idx_col = 0;
    let mut mv_col = [0; 2];
    let col_pic = self.co_located_4x4submb_partitions(
      slice,
      dpb,
      mb_part_idx,
      sub_mb_part_idx,
      &mut mv_col,
      &mut ref_idx_col,
    );

    let mut ref_idxl0_frm = 0;
    for i in 0..dpb.ref_pic_list0.len() {
      if dpb.ref_pic_list0(i) == col_pic {
        ref_idxl0_frm = i;
        break;
      }
    }
    *ref_idxl0 = if ref_idx_col < 0 {
      0
    } else {
      ref_idxl0_frm as isize
    };
    *ref_idxl1 = 0;

    let pic0 = dpb.ref_pic_list0(*ref_idxl0 as usize);
    let pic1 = dpb.ref_pic_list1(0);

    let curr_pic_order_cnt =
      std::cmp::min(dpb.poc.top_field_order_cnt, dpb.poc.bottom_field_order_cnt);
    let pic0_pic_order_cnt = std::cmp::min(pic0.top_field_order_cnt, pic0.bottom_field_order_cnt);
    let pic1_pic_order_cnt = std::cmp::min(pic1.top_field_order_cnt, pic1.bottom_field_order_cnt);

    if dpb
      .ref_pic_list0(*ref_idxl0 as usize)
      .reference_marked_type
      .is_long_term_reference()
      || pic1_pic_order_cnt - pic0_pic_order_cnt == 0
    {
      mv_l0[0] = mv_col[0];
      mv_l0[1] = mv_col[1];

      mv_l1[0] = 0;
      mv_l1[1] = 0;
    } else {
      let tb = clamp(curr_pic_order_cnt - pic0_pic_order_cnt, -128, 127);
      let td = clamp(pic1_pic_order_cnt - pic0_pic_order_cnt, -128, 127);
      let tx = (16384 + (td / 2).abs()) / td;

      let dist_scale_factor = clamp((tb * tx + 32) >> 6, -1024, 1023);

      mv_l0[0] = (dist_scale_factor * mv_col[0] + 128) >> 8;
      mv_l0[1] = (dist_scale_factor * mv_col[1] + 128) >> 8;
      mv_l1[0] = mv_l0[0] - mv_col[0];
      mv_l1[1] = mv_l0[1] - mv_col[1];
    }

    *pred_flagl0 = 1;
    *pred_flagl1 = 1;
  }

  /// 8.4.1.2.1 Derivation process for the co-located 4x4 sub-macroblock partitions
  pub fn co_located_4x4submb_partitions<'a>(
    &mut self,
    slice: &'a Slice,
    dpb: &'a DecodedPictureBuffer,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    mv_col: &mut [isize; 2],
    ref_idx_col: &mut isize,
  ) -> &'a Picture {
    let col_pic = dpb.ref_pic_list1(0);

    let luma4x4_blk_idx = if slice.sps.direct_8x8_inference_flag {
      5 * mb_part_idx
    } else {
      4 * mb_part_idx + sub_mb_part_idx
    } as isize;

    let x_col = inverse_raster_scan(luma4x4_blk_idx / 4, 8, 8, 16, 0)
      + inverse_raster_scan(luma4x4_blk_idx % 4, 4, 4, 8, 0);
    let y_col = inverse_raster_scan(luma4x4_blk_idx / 4, 8, 8, 16, 1)
      + inverse_raster_scan(luma4x4_blk_idx % 4, 4, 4, 8, 1);

    let y_m = y_col;
    let mut nb = MbNeighbor::new(slice.mb());
    slice.mb_and_submb_partition_indices(&mut nb, x_col, y_m);

    if nb.mb.mode().is_inter_mode() {
      mv_col[0] = 0;
      mv_col[1] = 0;
      *ref_idx_col = -1;
    } else {
      let pred_flagl0_col = nb.mb.pred_flagl0[nb.mb_part_idx as usize];
      let pred_flagl1_col = nb.mb.pred_flagl1[nb.mb_part_idx as usize];

      if pred_flagl0_col == 1 {
        mv_col[0] = nb.mb.mv_l0[nb.mb_part_idx as usize][nb.sub_mb_part_idx as usize][0];
        mv_col[1] = nb.mb.mv_l0[nb.mb_part_idx as usize][nb.sub_mb_part_idx as usize][1];
        *ref_idx_col = nb.mb.ref_idxl0[nb.mb_part_idx as usize];
      } else if pred_flagl1_col == 1 {
        mv_col[0] = nb.mb.mv_l1[nb.mb_part_idx as usize][nb.sub_mb_part_idx as usize][0];
        mv_col[1] = nb.mb.mv_l1[nb.mb_part_idx as usize][nb.sub_mb_part_idx as usize][1];
        *ref_idx_col = nb.mb.ref_idxl1[nb.mb_part_idx as usize];
      } else {
        unreachable!()
      }
    }
    col_pic
  }
}
