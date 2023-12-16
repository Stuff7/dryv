mod interpolation;
mod motion;
mod pred;
mod weighted;

use super::Frame;
use crate::{
  math::{clamp, inverse_raster_scan},
  video::slice::{dpb::DecodedPictureBuffer, header::DEFAULT_PWT, macroblock::SubMbType, Slice},
};

impl Frame {
  /// 8.4 Inter prediction process
  pub fn inter_prediction(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
  ) {
    let num_mb_part = if slice.mb().mb_type.is_b_skip() || slice.mb().mb_type.is_b_direct_16x16() {
      4
    } else {
      slice.mb().mb_type.num_mb_part()
    };

    let x_m = inverse_raster_scan(slice.curr_mb_addr, 16, 16, slice.pic_width_in_samples_l as isize, 0);
    let y_m = inverse_raster_scan(slice.curr_mb_addr, 16, 16, slice.pic_width_in_samples_l as isize, 1);

    for mb_part_idx in 0..num_mb_part {
      let x_p = inverse_raster_scan(mb_part_idx as isize, slice.mb().mb_part_width(), slice.mb().mb_part_height(), 16, 0);
      let y_p = inverse_raster_scan(mb_part_idx as isize, slice.mb().mb_part_width(), slice.mb().mb_part_height(), 16, 1);

      let part_width;
      let part_height;
      let mut part_width_c = 0;
      let mut part_height_c = 0;
      let num_sub_mb_part;

      if !slice.mb().mb_type.is_p8x8()
        && !slice.mb().mb_type.is_p_8x8ref0()
        && !slice.mb().mb_type.is_b_skip()
        && !slice.mb().mb_type.is_b_direct_16x16()
        && !slice.mb().mb_type.is_b8x8()
      {
        num_sub_mb_part = 1;
        part_width = slice.mb().mb_part_width();
        part_height = slice.mb().mb_part_height();
      } else if slice.mb().mb_type.is_p8x8()
        || slice.mb().mb_type.is_p_8x8ref0()
        || (slice.mb().mb_type.is_b8x8() && !slice.mb().sub_mb_type[mb_part_idx].is_b_direct8x8())
      {
        num_sub_mb_part = slice.mb().sub_mb_type[mb_part_idx].num_sub_mb_part;

        part_width = slice.mb().sub_mb_type[mb_part_idx].sub_mb_part_width as isize;
        part_height = slice.mb().sub_mb_type[mb_part_idx].sub_mb_part_height as isize;
      } else {
        num_sub_mb_part = 4;

        part_width = 4;
        part_height = 4;
      }

      if slice.chroma_array_type != 0 {
        part_width_c = part_width / slice.sub_width_c;
        part_height_c = part_height / slice.sub_height_c;
      }

      for sub_mb_part_idx in 0..num_sub_mb_part {
        let x_s;
        let y_s;
        if slice.mb().mb_type.is_p8x8() || slice.mb().mb_type.is_p_8x8ref0() || slice.mb().mb_type.is_b8x8() {
          x_s = inverse_raster_scan(sub_mb_part_idx as isize, part_width, part_height, 8, 0);
          y_s = inverse_raster_scan(sub_mb_part_idx as isize, part_width, part_height, 8, 1);
        } else {
          x_s = inverse_raster_scan(sub_mb_part_idx as isize, 4, 4, 8, 0);
          y_s = inverse_raster_scan(sub_mb_part_idx as isize, 4, 4, 8, 1);
        }

        let mut mv_l0 = [0; 2];
        let mut mv_l1 = [0; 2];
        let mut mv_cl0 = [0; 2];
        let mut mv_cl1 = [0; 2];
        let mut ref_idxl0 = 0;
        let mut ref_idxl1 = 0;
        let mut pred_flagl0 = 0;
        let mut pred_flagl1 = 0;
        let mut sub_mv_cnt = 0;

        self.mv_components_and_ref_indices(
          slice,
          dpb,
          mb_part_idx,
          sub_mb_part_idx,
          &mut mv_l0,
          &mut mv_l1,
          &mut mv_cl0,
          &mut mv_cl1,
          &mut ref_idxl0,
          &mut ref_idxl1,
          &mut pred_flagl0,
          &mut pred_flagl1,
          &mut sub_mv_cnt,
        );

        let mut log_wdl = 0;
        let mut w0_l = 1;
        let mut w1_l = 1;
        let mut o0_l = 0;
        let mut o1_l = 0;
        let mut log_wdcb = 0;
        let mut w0_cb = 1;
        let mut w1_cb = 1;
        let mut o0_cb = 0;
        let mut o1_cb = 0;
        let mut log_wdcr = 0;
        let mut w0_cr = 1;
        let mut w1_cr = 1;
        let mut o0_cr = 0;
        let mut o1_cr = 0;

        if slice.pps.weighted_pred_flag && (slice.slice_type.is_predictive())
          || (slice.pps.weighted_bipred_idc > 0 && slice.slice_type.is_bidirectional())
        {
          self.prediction_weights(
            slice,
            dpb,
            ref_idxl0 as usize,
            ref_idxl1 as usize,
            pred_flagl0,
            pred_flagl1,
            &mut log_wdl,
            &mut w0_l,
            &mut w1_l,
            &mut o0_l,
            &mut o1_l,
            &mut log_wdcb,
            &mut w0_cb,
            &mut w1_cb,
            &mut o0_cb,
            &mut o1_cb,
            &mut log_wdcr,
            &mut w0_cr,
            &mut w1_cr,
            &mut o0_cr,
            &mut o1_cr,
          );
        }

        let x_al = x_m + x_p + x_s;
        let y_al = y_m + y_p + y_s;

        self.inter_prediction_samples(
          slice,
          log_wdl,
          w0_l,
          w1_l,
          o0_l,
          o1_l,
          log_wdcb,
          w0_cb,
          w1_cb,
          o0_cb,
          o1_cb,
          log_wdcr,
          w0_cr,
          w1_cr,
          o0_cr,
          o1_cr,
          dpb,
          x_al,
          y_al,
          x_p,
          x_s,
          y_p,
          y_s,
          mb_part_idx as isize,
          sub_mb_part_idx as isize,
          part_width as usize,
          part_height as usize,
          part_width_c as usize,
          part_height_c as usize,
          &mv_l0,
          &mv_l1,
          &mv_cl0,
          &mv_cl1,
          ref_idxl0 as usize,
          ref_idxl1 as usize,
          pred_flagl0,
          pred_flagl1,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        );

        slice.mb_mut().mv_l0[mb_part_idx][sub_mb_part_idx][0] = mv_l0[0];
        slice.mb_mut().mv_l0[mb_part_idx][sub_mb_part_idx][1] = mv_l0[1];

        slice.mb_mut().mv_l1[mb_part_idx][sub_mb_part_idx][0] = mv_l1[0];
        slice.mb_mut().mv_l1[mb_part_idx][sub_mb_part_idx][1] = mv_l1[1];
        slice.mb_mut().ref_idxl0[mb_part_idx] = ref_idxl0;
        slice.mb_mut().ref_idxl1[mb_part_idx] = ref_idxl1;

        slice.mb_mut().pred_flagl0[mb_part_idx] = pred_flagl0;
        slice.mb_mut().pred_flagl1[mb_part_idx] = pred_flagl1;

        if slice.mb().is_skip() {
          for y in 0..part_height {
            for x in 0..part_width {
              let lx = (x_m + x_p + x_s + x) as usize;
              let ly = (y_m + y_p + y_s + y) as usize;
              let px = (x_p + x_s + x) as usize;
              let py = (y_p + y_s + y) as usize;
              self.luma_data[lx][ly] = pred_part_l[px][py];
            }
          }

          if slice.chroma_array_type != 0 {
            for y in 0..part_height_c {
              for x in 0..part_width_c {
                let dx = (x_m / slice.sub_width_c + x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize;
                let dy = (y_m / slice.sub_height_c + y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize;
                let px = (x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize;
                let py = (y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize;
                self.chroma_cb_data[dx][dy] = pred_part_cb[px][py];

                let dx = (x_m / slice.sub_width_c + x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize;
                let dy = (y_m / slice.sub_height_c + y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize;
                let px = (x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize;
                let py = (y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize;
                self.chroma_cr_data[dx][dy] = pred_part_cr[px][py];
              }
            }
          }
        }
      }
    }
  }

  /// 8.4.1 Derivation process for motion vector components and reference indices
  pub fn mv_components_and_ref_indices(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    mv_l0: &mut [isize; 2],
    mv_l1: &mut [isize; 2],
    mv_cl0: &mut [isize; 2],
    mv_cl1: &mut [isize; 2],
    ref_idxl0: &mut isize,
    ref_idxl1: &mut isize,
    pred_flagl0: &mut isize,
    pred_flagl1: &mut isize,
    sub_mv_cnt: &mut isize,
  ) {
    let mut mv_l0_a = [0; 2];
    let mut mv_l0_b = [0; 2];
    let mut mv_l0_c = [0; 2];
    let mut ref_idxl0_a = 0;
    let mut ref_idxl0_b = 0;
    let mut ref_idxl0_c = 0;

    let mut curr_sub_mb_type = SubMbType::none();

    if slice.mb().mb_type.is_p_skip() {
      *ref_idxl0 = 0;

      let nb = self.motion_data_of_neighboring_partitions(
        slice,
        mb_part_idx,
        sub_mb_part_idx,
        curr_sub_mb_type,
        false,
        &mut mv_l0_a,
        &mut mv_l0_b,
        &mut mv_l0_c,
        &mut ref_idxl0_a,
        &mut ref_idxl0_b,
        &mut ref_idxl0_c,
      );

      if nb.a.is_unavailable()
        || nb.b.is_unavailable()
        || (ref_idxl0_a == 0 && mv_l0_a[0] == 0 && mv_l0_a[1] == 0)
        || (ref_idxl0_b == 0 && mv_l0_b[0] == 0 && mv_l0_b[1] == 0)
      {
        mv_l0[0] = 0;
        mv_l0[1] = 0;
      } else {
        self.luma_motion_vector_prediction(slice, mb_part_idx, sub_mb_part_idx, curr_sub_mb_type, false, *ref_idxl0, mv_l0);
      }

      *pred_flagl0 = 1;
      *pred_flagl1 = 0;
      mv_l1[0] = -1;
      mv_l1[1] = -1;
      *sub_mv_cnt = 1;
    } else if slice.mb().mb_type.is_b_skip() || slice.mb().mb_type.is_b_direct_16x16() || slice.mb().sub_mb_type[mb_part_idx].is_b_direct8x8() {
      self.luma_motion_vectors_for_b_blocks(
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
      let mut mvp_l0 = [0; 2];
      let mut mvp_l1 = [0; 2];
      let mode = slice.mb().mb_type.inter_mode(mb_part_idx);
      let sub_mode = &slice.mb().sub_mb_type[mb_part_idx].sub_mb_part_pred_mode;

      if mode.is_predl0() || sub_mode.is_predl0() {
        *ref_idxl0 = slice.mb().ref_idxlx(0, mb_part_idx);
        *pred_flagl0 = 1;
      } else {
        *ref_idxl0 = -1;
        *pred_flagl0 = 0;
      }

      if mode.is_predl1() || sub_mode.is_predl1() {
        *ref_idxl1 = slice.mb().ref_idxlx(1, mb_part_idx);
        *pred_flagl1 = 1;
      } else {
        *ref_idxl1 = -1;
        *pred_flagl1 = 0;
      }

      *sub_mv_cnt = *pred_flagl0 + *pred_flagl1;

      if slice.mb().mb_type.is_b8x8() {
        curr_sub_mb_type = slice.mb().sub_mb_type[mb_part_idx];
      } else {
        curr_sub_mb_type = SubMbType::none();
      }

      if *pred_flagl0 == 1 {
        self.luma_motion_vector_prediction(slice, mb_part_idx, sub_mb_part_idx, curr_sub_mb_type, false, *ref_idxl0, &mut mvp_l0);

        mv_l0[0] = mvp_l0[0] + slice.mb().mvd_lx(0, mb_part_idx, sub_mb_part_idx, 0);
        mv_l0[1] = mvp_l0[1] + slice.mb().mvd_lx(0, mb_part_idx, sub_mb_part_idx, 1);
      }

      if *pred_flagl1 == 1 {
        self.luma_motion_vector_prediction(slice, mb_part_idx, sub_mb_part_idx, curr_sub_mb_type, true, *ref_idxl1, &mut mvp_l1);

        mv_l1[0] = mvp_l1[0] + slice.mb().mvd_lx(1, mb_part_idx, sub_mb_part_idx, 0);
        mv_l1[1] = mvp_l1[1] + slice.mb().mvd_lx(1, mb_part_idx, sub_mb_part_idx, 1);
      }
    }

    if slice.chroma_array_type != 0 {
      if *pred_flagl0 == 1 {
        mv_cl0[0] = mv_l0[0];
        mv_cl0[1] = mv_l0[1];
      }

      if *pred_flagl1 == 1 {
        mv_cl1[0] = mv_l1[0];
        mv_cl1[1] = mv_l1[1];
      }
    }
  }

  /// 8.4.3 Derivation process for prediction weights
  pub fn prediction_weights(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    ref_idxl0: usize,
    ref_idxl1: usize,
    pred_flagl0: isize,
    pred_flagl1: isize,
    log_wdl: &mut isize,
    w0_l: &mut isize,
    w1_l: &mut isize,
    o0_l: &mut isize,
    o1_l: &mut isize,
    log_wdcb: &mut isize,
    w0_cb: &mut isize,
    w1_cb: &mut isize,
    o0_cb: &mut isize,
    o1_cb: &mut isize,
    log_wdcr: &mut isize,
    w0_cr: &mut isize,
    w1_cr: &mut isize,
    o0_cr: &mut isize,
    o1_cr: &mut isize,
  ) {
    let implicit_mode_flag;
    let explicit_mode_flag;

    if slice.pps.weighted_bipred_idc == 2 && slice.slice_type.is_bidirectional() && pred_flagl0 == 1 && pred_flagl1 == 1 {
      implicit_mode_flag = 1;
      explicit_mode_flag = 0;
    } else if (slice.pps.weighted_bipred_idc == 1
      && slice.slice_type.is_bidirectional()
      && (pred_flagl0 + pred_flagl1 == 1 || pred_flagl0 + pred_flagl1 == 2))
      || (slice.pps.weighted_pred_flag && (slice.slice_type.is_non_switching_p() || slice.slice_type.is_switching_p()) && pred_flagl0 == 1)
    {
      implicit_mode_flag = 0;
      explicit_mode_flag = 1;
    } else {
      implicit_mode_flag = 0;
      explicit_mode_flag = 0;
    }

    if implicit_mode_flag == 1 {
      *log_wdl = 5;
      *o0_l = 0;
      *o1_l = 0;
      if slice.chroma_array_type != 0 {
        *log_wdcb = 5;
        *o0_cb = 0;
        *o1_cb = 0;

        *log_wdcr = 5;
        *o0_cr = 0;
        *o1_cr = 0;
      }
      let pic0 = dpb.ref_pic_list0(ref_idxl0);
      let pic1 = dpb.ref_pic_list1(ref_idxl1);

      let curr_pic_order_cnt = std::cmp::min(dpb.top_field_order_cnt, dpb.bottom_field_order_cnt);
      let pic0_pic_order_cnt = std::cmp::min(pic0.top_field_order_cnt, pic0.bottom_field_order_cnt);
      let pic1_pic_order_cnt = std::cmp::min(pic1.top_field_order_cnt, pic1.bottom_field_order_cnt);

      let tb = clamp(curr_pic_order_cnt - pic0_pic_order_cnt, -128, 127);
      let td = clamp(pic1_pic_order_cnt - pic0_pic_order_cnt, -128, 127);
      let tx = (16384 + (td / 2).abs()) / td;
      let dist_scale_factor = clamp((tb * tx + 32) >> 6, -1024, 1023);

      if pic1_pic_order_cnt - pic0_pic_order_cnt == 0
        || pic0.reference_marked_type.is_long_term_reference()
        || pic1.reference_marked_type.is_long_term_reference()
        || (dist_scale_factor >> 2) < -64
        || (dist_scale_factor >> 2) > 128
      {
        *w0_l = 32;
        *w1_l = 32;

        if slice.chroma_array_type != 0 {
          *w0_cb = 32;
          *w1_cb = 32;
          *w0_cr = 32;
          *w1_cr = 32;
        }
      } else {
        *w0_l = 64 - (dist_scale_factor >> 2);
        *w1_l = dist_scale_factor >> 2;

        if slice.chroma_array_type != 0 {
          *w0_cb = 64 - (dist_scale_factor >> 2);
          *w1_cb = dist_scale_factor >> 2;
          *w0_cr = 64 - (dist_scale_factor >> 2);
          *w1_cr = dist_scale_factor >> 2;
        }
      }
    } else if explicit_mode_flag == 1 {
      let ref_idx_l0_wp = ref_idxl0;
      let ref_idx_l1_wp = ref_idxl1;
      if let Some(pwt) = &slice.pred_weight_table {
        *log_wdl = pwt.luma_log2_weight_denom;
        let l0 = pwt.l0.get(ref_idx_l0_wp).unwrap_or(&DEFAULT_PWT);
        let l1 = pwt.l1.get(ref_idx_l1_wp).unwrap_or(&DEFAULT_PWT);
        *w0_l = l0.luma_weight;
        *w1_l = l1.luma_weight;
        *o0_l = l0.luma_offset * (1 << (slice.bit_depth_y - 8));
        *o1_l = l1.luma_offset * (1 << (slice.bit_depth_y - 8));

        if slice.chroma_array_type != 0 {
          *log_wdcb = pwt.chroma_log2_weight_denom;
          *w0_cb = l0.chroma_weight[0];
          *w1_cb = l1.chroma_weight[0];
          *o0_cb = l0.chroma_offset[0] * (1 << (slice.bit_depth_c - 8));
          *o1_cb = l1.chroma_offset[0] * (1 << (slice.bit_depth_c - 8));

          *log_wdcr = pwt.chroma_log2_weight_denom;
          *w0_cr = l0.chroma_weight[1];
          *w1_cr = l1.chroma_weight[1];
          *o0_cr = l0.chroma_offset[1] * (1 << (slice.bit_depth_c - 8));
          *o1_cr = l1.chroma_offset[1] * (1 << (slice.bit_depth_c - 8));
        }
      }
    }

    if explicit_mode_flag == 1 && pred_flagl0 == 1 && pred_flagl1 == 1 {
      if !(-128 <= *w0_l + *w1_l && *w0_l + *w1_l <= (if *log_wdl == 7 { 127 } else { 128 })) {
        panic!("w0_l + w1_l must be greater than or equal to -128, less than or equal to 127, or 128");
      }

      if slice.chroma_array_type != 0 {
        if !(-128 <= *w0_cb + *w1_cb && *w0_cb + *w1_cb <= (if *log_wdcb == 7 { 127 } else { 128 })) {
          panic!("w0_l + w1_l must be greater than or equal to -128, less than or equal to 127, or 128");
        }
        if !(-128 <= *w0_cr + *w1_cr && *w0_cb + *w1_cr <= (if *log_wdcr == 7 { 127 } else { 128 })) {
          panic!("w0_l + w1_l must be greater than or equal to -128, less than or equal to 127, or 128");
        }
      }
    }
  }

  /// 8.4.2 Decoding process for inter prediction samples
  pub fn inter_prediction_samples(
    &mut self,
    slice: &mut Slice,
    log_wdl: isize,
    w0_l: isize,
    w1_l: isize,
    o0_l: isize,
    o1_l: isize,
    log_wdcb: isize,
    w0_cb: isize,
    w1_cb: isize,
    o0_cb: isize,
    o1_cb: isize,
    log_wdcr: isize,
    w0_cr: isize,
    w1_cr: isize,
    o0_cr: isize,
    o1_cr: isize,
    dpb: &DecodedPictureBuffer,
    x_al: isize,
    y_al: isize,
    x_p: isize,
    x_s: isize,
    y_p: isize,
    y_s: isize,
    mb_part_idx: isize,
    sub_mb_part_idx: isize,
    part_width: usize,
    part_height: usize,
    part_width_c: usize,
    part_height_c: usize,
    mv_l0: &[isize; 2],
    mv_l1: &[isize; 2],
    mv_cl0: &[isize; 2],
    mv_cl1: &[isize; 2],
    ref_idxl0: usize,
    ref_idxl1: usize,
    pred_flagl0: isize,
    pred_flagl1: isize,
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
  ) {
    let mut pred_part_l0l = vec![0; part_width * part_height];
    let mut pred_part_l1l = vec![0; part_width * part_height];

    let mut pred_part_l0cb = vec![0; part_width_c * part_height_c];
    let mut pred_part_l1cb = vec![0; part_width_c * part_height_c];
    let mut pred_part_l0cr = vec![0; part_width_c * part_height_c];
    let mut pred_part_l1cr = vec![0; part_width_c * part_height_c];

    if pred_flagl0 == 1 {
      let ref_pic = dpb.ref_pic_list0(ref_idxl0);
      self.fractional_sample_interpolation(
        slice,
        dpb,
        x_al,
        y_al,
        mb_part_idx,
        sub_mb_part_idx,
        part_width,
        part_height,
        part_width_c,
        part_height_c,
        mv_l0,
        mv_cl0,
        ref_pic,
        &mut pred_part_l0l,
        &mut pred_part_l0cb,
        &mut pred_part_l0cr,
      );
    }

    if pred_flagl1 == 1 {
      let ref_pic = dpb.ref_pic_list1(ref_idxl1);
      self.fractional_sample_interpolation(
        slice,
        dpb,
        x_al,
        y_al,
        mb_part_idx,
        sub_mb_part_idx,
        part_width,
        part_height,
        part_width_c,
        part_height_c,
        mv_l1,
        mv_cl1,
        ref_pic,
        &mut pred_part_l1l,
        &mut pred_part_l1cb,
        &mut pred_part_l1cr,
      );
    }

    self.weighted_sample_prediction(
      slice,
      log_wdl,
      w0_l,
      w1_l,
      o0_l,
      o1_l,
      log_wdcb,
      w0_cb,
      w1_cb,
      o0_cb,
      o1_cb,
      log_wdcr,
      w0_cr,
      w1_cr,
      o0_cr,
      o1_cr,
      x_p,
      x_s,
      y_p,
      y_s,
      part_width as isize,
      part_height as isize,
      part_width_c as isize,
      part_height_c as isize,
      pred_flagl0,
      pred_flagl1,
      &pred_part_l0l,
      &pred_part_l0cb,
      &pred_part_l0cr,
      &pred_part_l1l,
      &pred_part_l1cb,
      &pred_part_l1cr,
      pred_part_l,
      pred_part_cb,
      pred_part_cr,
    );
  }
}
