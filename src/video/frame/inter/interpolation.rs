use super::super::Frame;
use crate::{
  math::clamp,
  video::slice::{dpb::Picture, Slice},
};

impl Frame {
  /// 8.4.2.2 Fractional sample interpolation process
  pub fn fraction_sample_interpolation(
    &mut self,
    slice: &mut Slice,
    x_al: isize,
    y_al: isize,
    mb_part_idx: isize,
    sub_mb_part_idx: isize,
    part_width: usize,
    part_height: usize,
    part_width_c: usize,
    part_height_c: usize,
    mv_lx: &[isize; 2],
    mv_clx: &[isize; 2],
    ref_pic: &Picture,
    pred_part_lxl: &[isize],
    pred_part_lxcb: &[isize],
    pred_part_lxcr: &[isize],
  ) {
    for y_l in 0..part_height {
      for x_l in 0..part_width {
        let x_int_l = x_al + (mv_lx[0] >> 2) + x_l as isize;
        let y_int_l = y_al + (mv_lx[1] >> 2) + y_l as isize;
        let x_frac_l = mv_lx[0] & 3;
        let y_frac_l = mv_lx[1] & 3;

        pred_part_lxl[y_l * part_width + x_l] = todo!("Luma sample interpolation process");
      }
    }

    if slice.chroma_array_type != 0 {
      let x_int_c;
      let y_int_c;
      let x_frac_c;
      let y_frac_c;
      for y_c in 0..part_height_c as isize {
        for x_c in 0..part_width_c as isize {
          if slice.chroma_array_type == 1 {
            x_int_c = (x_al / slice.sub_width_c) + (mv_clx[0] >> 3) + x_c;
            y_int_c = (y_al / slice.sub_height_c) + (mv_clx[1] >> 3) + y_c;
            x_frac_c = mv_clx[0] & 7;
            y_frac_c = mv_clx[1] & 7;
          } else if slice.chroma_array_type == 2 {
            x_int_c = (x_al / slice.sub_width_c) + (mv_clx[0] >> 3) + x_c;
            y_int_c = (y_al / slice.sub_height_c) + (mv_clx[1] >> 2) + y_c;
            x_frac_c = mv_clx[0] & 7;
            y_frac_c = (mv_clx[1] & 3) << 1;
          } else {
            x_int_c = x_al + (mv_lx[0] >> 2) + x_c;
            y_int_c = y_al + (mv_lx[1] >> 2) + y_c;
            x_frac_c = (mv_clx[0] & 3);
            y_frac_c = (mv_clx[1] & 3);
          }

          if slice.chroma_array_type != 3 {
            pred_part_lxcb[(y_c * part_width_c as isize + x_c) as usize] =
              todo!("Chroma sample interpolation process");
            pred_part_lxcr[(y_c * part_width_c as isize + x_c) as usize] =
              todo!("Chroma sample interpolation process");
          } else {
            pred_part_lxcb[(y_c * part_width_c as isize + x_c) as usize] =
              todo!("Luma sample interpolation process");
            pred_part_lxcr[(y_c * part_width_c as isize + x_c) as usize] =
              todo!("Luma sample interpolation process");
          }
        }
      }
    }
  }
}

pub fn get_luma_sample(
  ref_pic: &Picture,
  x_dzl: usize,
  y_dzl: usize,
  x_int_l: usize,
  y_int_l: usize,
) -> u8 {
  ref_pic.frame.luma_data[clamp(x_int_l + x_dzl, 0, ref_pic.frame.width_l - 1)]
    [clamp(y_int_l + y_dzl, 0, ref_pic.frame.height_l - 1)]
}
