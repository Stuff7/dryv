use super::super::Frame;
use crate::{
  math::clamp,
  video::slice::{
    dpb::{DecodedPictureBuffer, Picture},
    Slice,
  },
};

impl Frame {
  /// 8.4.2.2 Fractional sample interpolation process
  pub fn fractional_sample_interpolation(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
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
    pred_part_lxl: &mut [u8],
    pred_part_lxcb: &mut [u8],
    pred_part_lxcr: &mut [u8],
  ) {
    for y_l in 0..part_height {
      for x_l in 0..part_width {
        let x_int_l = x_al + (mv_lx[0] >> 2) + x_l as isize;
        let y_int_l = y_al + (mv_lx[1] >> 2) + y_l as isize;
        let x_frac_l = mv_lx[0] & 3;
        let y_frac_l = mv_lx[1] & 3;

        pred_part_lxl[y_l * part_width + x_l] = self.luma_sample_interpolation(slice, dpb, x_int_l, y_int_l, x_frac_l, y_frac_l, ref_pic);
      }
    }

    if slice.chroma_array_type != 0 {
      let mut x_int_c;
      let mut y_int_c;
      let mut x_frac_c;
      let mut y_frac_c;
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
            x_frac_c = mv_clx[0] & 3;
            y_frac_c = mv_clx[1] & 3;
          }

          if slice.chroma_array_type != 3 {
            pred_part_lxcb[(y_c * part_width_c as isize + x_c) as usize] =
              self.chroma_sample_interpolation(slice, dpb, x_int_c, y_int_c, x_frac_c, y_frac_c, ref_pic, true);
            pred_part_lxcr[(y_c * part_width_c as isize + x_c) as usize] =
              self.chroma_sample_interpolation(slice, dpb, x_int_c, y_int_c, x_frac_c, y_frac_c, ref_pic, false);
          } else {
            pred_part_lxcb[(y_c * part_width_c as isize + x_c) as usize] =
              self.luma_sample_interpolation(slice, dpb, x_int_c, y_int_c, x_frac_c, y_frac_c, ref_pic);
            pred_part_lxcr[(y_c * part_width_c as isize + x_c) as usize] =
              self.luma_sample_interpolation(slice, dpb, x_int_c, y_int_c, x_frac_c, y_frac_c, ref_pic);
          }
        }
      }
    }
  }

  /// 8.4.2.2.1 Luma sample interpolation process
  pub fn luma_sample_interpolation(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    x_int_l: isize,
    y_int_l: isize,
    x_frac_l: isize,
    y_frac_l: isize,
    ref_pic: &Picture,
  ) -> u8 {
    if dpb.pic_order_cnt == 16 && slice.mb_x == 3 && slice.mb_y == 10 {
      let mut view = vec![0; ref_pic.frame.width_l * ref_pic.frame.height_l];
      for y in 0..slice.pic_height_in_samples_l {
        for x in 0..slice.pic_width_in_samples_l {
          let l = ref_pic.frame.luma_data[x][y];
          view[y * slice.pic_width_in_samples_l + x] = l;
        }
      }
    }

    let l_a = get_luma_sample(ref_pic, 0, -2, x_int_l, y_int_l);
    let l_b = get_luma_sample(ref_pic, 1, -2, x_int_l, y_int_l);
    let l_c = get_luma_sample(ref_pic, 0, -1, x_int_l, y_int_l);
    let l_d = get_luma_sample(ref_pic, 1, -1, x_int_l, y_int_l);
    let l_e = get_luma_sample(ref_pic, -2, 0, x_int_l, y_int_l);
    let l_f = get_luma_sample(ref_pic, -1, 0, x_int_l, y_int_l);
    let l_g = get_luma_sample(ref_pic, 0, 0, x_int_l, y_int_l);
    let l_h = get_luma_sample(ref_pic, 1, 0, x_int_l, y_int_l);
    let l_i = get_luma_sample(ref_pic, 2, 0, x_int_l, y_int_l);
    let l_j = get_luma_sample(ref_pic, 3, 0, x_int_l, y_int_l);
    let l_k = get_luma_sample(ref_pic, -2, 1, x_int_l, y_int_l);
    let l_l = get_luma_sample(ref_pic, -1, 1, x_int_l, y_int_l);
    let l_m = get_luma_sample(ref_pic, 0, 1, x_int_l, y_int_l);
    let l_n = get_luma_sample(ref_pic, 1, 1, x_int_l, y_int_l);
    let l_p = get_luma_sample(ref_pic, 2, 1, x_int_l, y_int_l);
    let l_q = get_luma_sample(ref_pic, 3, 1, x_int_l, y_int_l);
    let l_r = get_luma_sample(ref_pic, 0, 2, x_int_l, y_int_l);
    let l_s = get_luma_sample(ref_pic, 1, 2, x_int_l, y_int_l);
    let l_t = get_luma_sample(ref_pic, 0, 3, x_int_l, y_int_l);
    let l_u = get_luma_sample(ref_pic, 1, 3, x_int_l, y_int_l);

    let x11 = get_luma_sample(ref_pic, -2, -2, x_int_l, y_int_l);
    let x12 = get_luma_sample(ref_pic, -1, -2, x_int_l, y_int_l);
    let x13 = get_luma_sample(ref_pic, 2, -2, x_int_l, y_int_l);

    let x14 = get_luma_sample(ref_pic, 3, -2, x_int_l, y_int_l);
    let x21 = get_luma_sample(ref_pic, -2, -1, x_int_l, y_int_l);
    let x22 = get_luma_sample(ref_pic, -1, -1, x_int_l, y_int_l);
    let x23 = get_luma_sample(ref_pic, 2, -1, x_int_l, y_int_l);
    let x24 = get_luma_sample(ref_pic, 3, -1, x_int_l, y_int_l);

    let x31 = get_luma_sample(ref_pic, -2, 2, x_int_l, y_int_l);
    let x32 = get_luma_sample(ref_pic, -1, 2, x_int_l, y_int_l);
    let x33 = get_luma_sample(ref_pic, 2, 2, x_int_l, y_int_l);
    let x34 = get_luma_sample(ref_pic, 3, 2, x_int_l, y_int_l);

    let x41 = get_luma_sample(ref_pic, -2, 3, x_int_l, y_int_l);
    let x42 = get_luma_sample(ref_pic, -1, 3, x_int_l, y_int_l);
    let x43 = get_luma_sample(ref_pic, 2, 3, x_int_l, y_int_l);
    let x44 = get_luma_sample(ref_pic, 3, 3, x_int_l, y_int_l);

    let b1 = tap_filter(l_e, l_f, l_g, l_h, l_i, l_j);
    let h1 = tap_filter(l_a, l_c, l_g, l_m, l_r, l_t);
    let s1 = tap_filter(l_k, l_l, l_m, l_n, l_p, l_q);
    let m1 = tap_filter(l_b, l_d, l_h, l_n, l_s, l_u);

    let b = clip_1y(slice, (b1 + 16) >> 5);
    let h = clip_1y(slice, (h1 + 16) >> 5);
    let s = clip_1y(slice, (s1 + 16) >> 5);
    let m = clip_1y(slice, (m1 + 16) >> 5);

    // let aa = tap_filter(x11, x12, l_a, l_b, x13, x14);
    // let bb = tap_filter(x21, x22, l_c, l_d, x23, x24);
    // let gg = tap_filter(x31, x32, l_r, l_s, x33, x34);
    // let hh = tap_filter(x41, x42, l_t, l_u, x43, x44);

    let cc = tap_filter(x11, x21, l_e, l_k, x31, x41);
    let dd = tap_filter(x12, x22, l_f, l_l, x32, x42);
    let ee = tap_filter(x13, x23, l_i, l_p, x33, x43);
    let ff = tap_filter(x14, x24, l_j, l_q, x34, x44);

    let j1 = tap_filter(cc, dd, h1, m1, ee, ff);

    let j = clip_1y(slice, (j1 + 512) >> 10);

    let a = (l_g + b + 1) >> 1;
    let c = (l_h + b + 1) >> 1;
    let d = (l_g + h + 1) >> 1;
    let n = (l_m + h + 1) >> 1;
    let f = (b + j + 1) >> 1;
    let i = (h + j + 1) >> 1;
    let k = (j + m + 1) >> 1;
    let q = (j + s + 1) >> 1;

    let e = (b + h + 1) >> 1;
    let g = (b + m + 1) >> 1;
    let p = (h + s + 1) >> 1;
    let r = (m + s + 1) >> 1;

    let pred_part_lxl = [[l_g, d, h, n], [a, e, i, p], [b, f, j, q], [c, g, k, r]];

    pred_part_lxl[x_frac_l as usize][y_frac_l as usize] as u8
  }

  /// 8.4.2.2.2 Chroma sample interpolation process
  pub fn chroma_sample_interpolation(
    &mut self,
    slice: &mut Slice,
    dpb: &DecodedPictureBuffer,
    x_int_c: isize,
    y_int_c: isize,
    x_frac_c: isize,
    y_frac_c: isize,
    ref_pic: &Picture,
    is_chroma_cb: bool,
  ) -> u8 {
    let ref_pic_height_effective_c = slice.pic_height_in_samples_c as isize;

    let x_ac = clamp(x_int_c, 0, ref_pic.frame.width_c as isize - 1);
    let x_bc = clamp(x_int_c + 1, 0, ref_pic.frame.width_c as isize - 1);
    let x_cc = clamp(x_int_c, 0, ref_pic.frame.width_c as isize - 1);
    let x_dc = clamp(x_int_c + 1, 0, ref_pic.frame.width_c as isize - 1);

    let y_ac = clamp(y_int_c, 0, ref_pic_height_effective_c - 1);
    let y_bc = clamp(y_int_c, 0, ref_pic_height_effective_c - 1);
    let y_cc = clamp(y_int_c + 1, 0, ref_pic_height_effective_c - 1);
    let y_dc = clamp(y_int_c + 1, 0, ref_pic_height_effective_c - 1);

    let buffer = if is_chroma_cb {
      &ref_pic.frame.chroma_cb_data
    } else {
      &ref_pic.frame.chroma_cr_data
    };

    let a = buffer[x_ac as usize][y_ac as usize] as isize;
    let b = buffer[x_bc as usize][y_bc as usize] as isize;
    let c = buffer[x_cc as usize][y_cc as usize] as isize;
    let d = buffer[x_dc as usize][y_dc as usize] as isize;

    (((8 - x_frac_c) * (8 - y_frac_c) * a + x_frac_c * (8 - y_frac_c) * b + (8 - x_frac_c) * y_frac_c * c + x_frac_c * y_frac_c * d + 32) >> 6) as u8
  }
}

fn get_luma_sample(ref_pic: &Picture, x_dzl: isize, y_dzl: isize, x_int_l: isize, y_int_l: isize) -> isize {
  ref_pic.frame.luma_data[clamp(x_int_l + x_dzl, 0, ref_pic.frame.width_l as isize - 1) as usize]
    [clamp(y_int_l + y_dzl, 0, ref_pic.frame.height_l as isize - 1) as usize] as isize
}

fn tap_filter(v1: isize, v2: isize, v3: isize, v4: isize, v5: isize, v6: isize) -> isize {
  v1 - 5 * v2 + 20 * v3 + 20 * v4 - 5 * v5 + v6
}

fn clip_1y(slice: &Slice, x: isize) -> isize {
  clamp(x, 0, (1 << slice.bit_depth_y) - 1)
}
