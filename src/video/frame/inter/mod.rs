use crate::{
  math::inverse_raster_scan,
  video::slice::{dpb::DecodedPictureBuffer, Slice},
};

use super::Frame;

impl Frame {
  /// 8.4 Inter prediction process
  pub fn inter_prediction(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
    is_skip: bool,
  ) {
    let num_mb_part = if slice.mb().mb_type.is_b_skip() || slice.mb().mb_type.is_b_direct_16x16() {
      4
    } else {
      slice.mb().mb_type.num_mb_part()
    };

    let x_m = inverse_raster_scan(
      slice.curr_mb_addr,
      16,
      16,
      slice.pic_width_in_samples_l as isize,
      0,
    );
    let y_m = inverse_raster_scan(
      slice.curr_mb_addr,
      16,
      16,
      slice.pic_width_in_samples_l as isize,
      1,
    );

    for mb_part_idx in 0..num_mb_part {
      let x_p = inverse_raster_scan(
        mb_part_idx as isize,
        slice.mb().mb_part_width(),
        slice.mb().mb_part_height(),
        16,
        0,
      );
      let y_p = inverse_raster_scan(
        mb_part_idx as isize,
        slice.mb().mb_part_width(),
        slice.mb().mb_part_height(),
        16,
        1,
      );

      let part_width;
      let part_height;
      let part_width_c;
      let part_height_c;
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
        if slice.mb().mb_type.is_p8x8()
          || slice.mb().mb_type.is_p_8x8ref0()
          || slice.mb().mb_type.is_b8x8()
        {
          x_s = inverse_raster_scan(sub_mb_part_idx as isize, part_width, part_height, 8, 0);
          y_s = inverse_raster_scan(sub_mb_part_idx as isize, part_width, part_height, 8, 1);
        } else {
          x_s = inverse_raster_scan(sub_mb_part_idx as isize, 4, 4, 8, 0);
          y_s = inverse_raster_scan(sub_mb_part_idx as isize, 4, 4, 8, 1);
        }

        let mut mv_cnt = 0;

        let mv_l0 = [0; 2];
        let mv_l1 = [0; 2];
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
          &mv_l0,
          &mv_l1,
          &mut mv_cl0,
          &mut mv_cl1,
          &mut ref_idxl0,
          &mut ref_idxl1,
          &mut pred_flagl0,
          &mut pred_flagl1,
          &mut sub_mv_cnt,
        );

        mv_cnt += sub_mv_cnt;

        let log_wdl = 0;
        let w0_l = 1;
        let w1_l = 1;
        let o0_l = 0;
        let o1_l = 0;
        let log_wdcb = 0;
        let w0_cb = 1;
        let w1_cb = 1;
        let o0_cb = 0;
        let o1_cb = 0;
        let log_wdcr = 0;
        let w0_cr = 1;
        let w1_cr = 1;
        let o0_cr = 0;
        let o1_cr = 0;

        if slice.pps.weighted_pred_flag && (slice.slice_type.is_predictive())
          || (slice.pps.weighted_bipred_idc > 0 && slice.slice_type.is_bidirectional())
        {
          todo!("Derivation process for prediction weights");
        }

        let x_al = x_m + x_p + x_s;
        let y_al = y_m + y_p + y_s;

        todo!("Decoding process for Inter prediction samples");

        slice.mb_mut().mv_l0[mb_part_idx][sub_mb_part_idx][0] = mv_l0[0];
        slice.mb_mut().mv_l0[mb_part_idx][sub_mb_part_idx][1] = mv_l0[1];

        slice.mb_mut().mv_l1[mb_part_idx][sub_mb_part_idx][0] = mv_l1[0];
        slice.mb_mut().mv_l1[mb_part_idx][sub_mb_part_idx][1] = mv_l1[1];
        slice.mb_mut().ref_idxl0[mb_part_idx] = ref_idxl0;
        slice.mb_mut().ref_idxl1[mb_part_idx] = ref_idxl1;

        slice.mb_mut().pred_flagl0[mb_part_idx] = pred_flagl0;
        slice.mb_mut().pred_flagl1[mb_part_idx] = pred_flagl1;

        if is_skip {
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
                let dx =
                  (x_m / slice.sub_width_c + x_p / slice.sub_width_c + x_s / slice.sub_width_c + x)
                    as usize;
                let dy = (y_m / slice.sub_height_c
                  + y_p / slice.sub_height_c
                  + y_s / slice.sub_height_c
                  + y) as usize;
                let px = (x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize;
                let py = (y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize;
                self.chroma_cb_data[dx][dy] = pred_part_cb[px][py];

                let dx =
                  (x_m / slice.sub_width_c + x_p / slice.sub_width_c + x_s / slice.sub_width_c + x)
                    as usize;
                let dy = (y_m / slice.sub_height_c
                  + y_p / slice.sub_height_c
                  + y_s / slice.sub_height_c
                  + y) as usize;
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
    mv_l0: &[isize; 2],
    mv_l1: &[isize; 2],
    mv_cl0: &mut [isize; 2],
    mv_cl1: &mut [isize; 2],
    ref_idxl0: &mut isize,
    ref_idxl1: &mut isize,
    pred_flag_l0: &mut isize,
    pred_flagl1: &mut isize,
    sub_mv_cnt: &mut isize,
  ) {
    let mbaddr_a = -1;
    let mbaddr_b = -1;
    let mbaddr_c = -1;

    let mv_l0_a = [0; 2];
    let mv_l0_b = [0; 2];
    let mv_l0_c = [0; 2];
    let ref_idxl0_a = 0;
    let ref_idxl0_b = 0;
    let ref_idxl0_c = 0;

    let mut curr_sub_mb_type = -1;

    if slice.mb().mb_type.is_p_skip() {
      *ref_idxl0 = 0;

      todo!("Derivation process for motion data of neighbouring partitions");

      if mbaddr_a == -1
        || mbaddr_b == -1
        || (ref_idxl0_a == 0 && mv_l0_a[0] == 0 && mv_l0_a[1] == 0)
        || (ref_idxl0_b == 0 && mv_l0_b[0] == 0 && mv_l0_b[1] == 0)
      {
        mv_l0[0] = 0;
        mv_l0[1] = 0;
      } else {
        todo!("Derivation_process_for_luma_motion_vector_prediction");
      }

      *pred_flag_l0 = 1;
      *pred_flagl1 = 0;
      mv_l1[0] = -1;
      mv_l1[1] = -1;
      *sub_mv_cnt = 1;
    } else if slice.mb().mb_type.is_b_skip()
      || slice.mb().mb_type.is_b_direct_16x16()
      || slice.mb().sub_mb_type[mb_part_idx].is_b_direct8x8()
    {
      todo!(
        "Derivation process for luma motion vectors for B Skip or B Direct 16x16 or B Direct 8x8"
      );
    } else {
      let mvp_l0 = [0; 2];
      let mvp_l1 = [0; 2];
      let mode = slice.mb().mb_type.inter_mode(mb_part_idx);
      let sub_mode = &slice.mb().sub_mb_type[mb_part_idx].sub_mb_part_pred_mode;

      if mode.is_predl0() || sub_mode.is_predl0() {
        *ref_idxl0 = slice.mb().ref_idxl0[mb_part_idx];
        *pred_flag_l0 = 1;
      } else {
        *ref_idxl0 = -1;
        *pred_flag_l0 = 0;
      }

      if mode.is_predl1() || sub_mode.is_predl1() {
        *ref_idxl1 = slice.mb().ref_idxl1[mb_part_idx];
        *pred_flagl1 = 1;
      } else {
        *ref_idxl1 = -1;
        *pred_flagl1 = 0;
      }

      *sub_mv_cnt = *pred_flag_l0 + *pred_flagl1;

      if slice.mb().mb_type.is_b8x8() {
        curr_sub_mb_type = *slice.mb().sub_mb_type[mb_part_idx] as isize;
      } else {
        curr_sub_mb_type = -1;
      }

      if *pred_flag_l0 == 1 {
        todo!("Derivation process for luma motion vector prediction");

        mv_l0[0] = mvp_l0[0] + slice.mb().mvd[0][mb_part_idx * sub_mb_part_idx][0];
        mv_l0[1] = mvp_l0[1] + slice.mb().mvd[0][mb_part_idx * sub_mb_part_idx][1];
      }

      if *pred_flagl1 == 1 {
        todo!("Derivation process for luma motion vector prediction");

        mv_l1[0] = mvp_l1[0] + slice.mb().mvd[1][mb_part_idx * sub_mb_part_idx][0];
        mv_l1[1] = mvp_l1[1] + slice.mb().mvd[1][mb_part_idx * sub_mb_part_idx][1];
      }
    }

    if slice.chroma_array_type != 0 {
      if *pred_flag_l0 == 1 {
        mv_cl0[0] = mv_l0[0];
        mv_cl0[1] = mv_l0[1];
      }

      if *pred_flagl1 == 1 {
        mv_cl1[0] = mv_l1[0];
        mv_cl1[1] = mv_l1[1];
      }
    }
  }
}
