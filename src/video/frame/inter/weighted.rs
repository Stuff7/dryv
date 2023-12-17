use super::super::Frame;
use crate::{math::clamp, video::slice::Slice};

impl Frame {
  /// 8.4.2.3 Weighted sample prediction process
  pub fn weighted_sample_prediction(
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
    x_p: isize,
    x_s: isize,
    y_p: isize,
    y_s: isize,
    part_width: isize,
    part_height: isize,
    part_width_c: isize,
    part_height_c: isize,
    pred_flag_l0: isize,
    pred_flag_l1: isize,
    pred_part_l0l: &[u8],
    pred_part_l0cb: &[u8],
    pred_part_l0cr: &[u8],
    pred_part_l1l: &[u8],
    pred_part_l1cb: &[u8],
    pred_part_l1cr: &[u8],
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
  ) {
    if pred_flag_l0 == 1 && slice.slice_type.is_predictive() {
      if slice.pps.weighted_pred_flag {
        self.implicit_weighted_sample_prediction(
          slice,
          x_p,
          x_s,
          y_p,
          y_s,
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
          part_width,
          part_height,
          part_width_c,
          part_height_c,
          pred_flag_l0,
          pred_flag_l1,
          pred_part_l0l,
          pred_part_l0cb,
          pred_part_l0cr,
          pred_part_l1l,
          pred_part_l1cb,
          pred_part_l1cr,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        );
      } else {
        self.default_weighted_sample_prediction(
          slice,
          x_p,
          x_s,
          y_p,
          y_s,
          part_width,
          part_height,
          part_width_c,
          part_height_c,
          pred_flag_l0,
          pred_flag_l1,
          pred_part_l0l,
          pred_part_l0cb,
          pred_part_l0cr,
          pred_part_l1l,
          pred_part_l1cb,
          pred_part_l1cr,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        )
      }
    } else if (pred_flag_l0 == 1 || pred_flag_l1 == 1) && slice.slice_type.is_bidirectional() {
      if slice.pps.weighted_bipred_idc == 0 {
        self.default_weighted_sample_prediction(
          slice,
          x_p,
          x_s,
          y_p,
          y_s,
          part_width,
          part_height,
          part_width_c,
          part_height_c,
          pred_flag_l0,
          pred_flag_l1,
          pred_part_l0l,
          pred_part_l0cb,
          pred_part_l0cr,
          pred_part_l1l,
          pred_part_l1cb,
          pred_part_l1cr,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        )
      } else if slice.pps.weighted_bipred_idc == 1 || (pred_flag_l0 == 1 && pred_flag_l1 == 1) {
        self.implicit_weighted_sample_prediction(
          slice,
          x_p,
          x_s,
          y_p,
          y_s,
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
          part_width,
          part_height,
          part_width_c,
          part_height_c,
          pred_flag_l0,
          pred_flag_l1,
          pred_part_l0l,
          pred_part_l0cb,
          pred_part_l0cr,
          pred_part_l1l,
          pred_part_l1cb,
          pred_part_l1cr,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        );
      } else {
        self.default_weighted_sample_prediction(
          slice,
          x_p,
          x_s,
          y_p,
          y_s,
          part_width,
          part_height,
          part_width_c,
          part_height_c,
          pred_flag_l0,
          pred_flag_l1,
          pred_part_l0l,
          pred_part_l0cb,
          pred_part_l0cr,
          pred_part_l1l,
          pred_part_l1cb,
          pred_part_l1cr,
          pred_part_l,
          pred_part_cb,
          pred_part_cr,
        )
      }
    }
  }

  /// 8.4.2.3.1 Default weighted sample prediction process
  pub fn default_weighted_sample_prediction(
    &mut self,
    slice: &mut Slice,
    x_p: isize,
    x_s: isize,
    y_p: isize,
    y_s: isize,
    part_width: isize,
    part_height: isize,
    part_width_c: isize,
    part_height_c: isize,
    pred_flag_l0: isize,
    pred_flag_l1: isize,
    pred_part_l0l: &[u8],
    pred_part_l0cb: &[u8],
    pred_part_l0cr: &[u8],
    pred_part_l1l: &[u8],
    pred_part_l1cb: &[u8],
    pred_part_l1cr: &[u8],
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
  ) {
    if pred_flag_l0 == 1 && pred_flag_l1 == 0 {
      for y in 0..part_height {
        for x in 0..part_width {
          pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = pred_part_l0l[(y * part_width + x) as usize];
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = pred_part_l0cb[(y * part_width_c + x) as usize];
            pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = pred_part_l0cr[(y * part_width_c + x) as usize];
          }
        }
      }
    } else if pred_flag_l0 == 0 && pred_flag_l1 == 1 {
      for y in 0..part_height {
        for x in 0..part_width {
          pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = pred_part_l1l[(y * part_width + x) as usize];
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = pred_part_l1cb[(y * part_width_c + x) as usize];
            pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = pred_part_l1cr[(y * part_width_c + x) as usize];
          }
        }
      }
    } else {
      for y in 0..part_height {
        for x in 0..part_width {
          pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] =
            (pred_part_l0l[(y * part_width + x) as usize] + pred_part_l1l[(y * part_width + x) as usize] + 1) >> 1;
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] =
              (pred_part_l0cb[(y * part_width_c + x) as usize] + pred_part_l1cb[(y * part_width_c + x) as usize] + 1) >> 1;
            pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] =
              (pred_part_l0cr[(y * part_width_c + x) as usize] + pred_part_l1cr[(y * part_width_c + x) as usize] + 1) >> 1;
          }
        }
      }
    }
  }

  /// 8.4.2.3.2 Weighted sample prediction process (Implicit)
  pub fn implicit_weighted_sample_prediction(
    &mut self,
    slice: &mut Slice,
    x_p: isize,
    x_s: isize,
    y_p: isize,
    y_s: isize,
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
    part_width: isize,
    part_height: isize,
    part_width_c: isize,
    part_height_c: isize,
    pred_flag_l0: isize,
    pred_flag_l1: isize,
    pred_part_l0l: &[u8],
    pred_part_l0cb: &[u8],
    pred_part_l0cr: &[u8],
    pred_part_l1l: &[u8],
    pred_part_l1cb: &[u8],
    pred_part_l1cr: &[u8],
    pred_part_l: &mut [[u8; 16]; 16],
    pred_part_cb: &mut [[u8; 16]; 16],
    pred_part_cr: &mut [[u8; 16]; 16],
  ) {
    if pred_flag_l0 == 1 && pred_flag_l1 == 0 {
      for y in 0..part_height {
        for x in 0..part_width {
          if log_wdl >= 1 {
            pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = clamp(
              ((pred_part_l0l[(y * part_width + x) as usize] as isize * w0_l + (1 << (log_wdl - 1))) >> log_wdl) + o0_l,
              0,
              (1 << slice.bit_depth_y) - 1,
            ) as u8;
          } else {
            pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = clamp(
              pred_part_l0l[(y * part_width + x) as usize] as isize * w0_l + o0_l,
              0,
              (1 << slice.bit_depth_y) - 1,
            ) as u8;
          }
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            if log_wdcb >= 1 {
              pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                ((pred_part_l0cb[(y * part_width_c + x) as usize] as isize * w0_cb + (1 << (log_wdcb - 1))) >> log_wdcb) + o0_cb,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            } else {
              pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                pred_part_l0cb[(y * part_width_c + x) as usize] as isize * w0_cb + o0_cb,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            }

            if log_wdcr >= 1 {
              pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                ((pred_part_l0cr[(y * part_width_c + x) as usize] as isize * w0_cr + (1 << (log_wdcr - 1))) >> log_wdcr) + o0_cr,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            } else {
              pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                pred_part_l0cr[(y * part_width_c + x) as usize] as isize * w0_cr + o0_cr,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            }
          }
        }
      }
    } else if pred_flag_l0 == 0 && pred_flag_l1 == 1 {
      for y in 0..part_height {
        for x in 0..part_width {
          if log_wdl >= 1 {
            pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = clamp(
              ((pred_part_l1l[(y * part_width + x) as usize] as isize * w0_l + (1 << (log_wdl - 1))) >> log_wdl) + o0_l,
              0,
              (1 << slice.bit_depth_y) - 1,
            ) as u8;
          } else {
            pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = clamp(
              pred_part_l1l[(y * part_width + x) as usize] as isize * w0_l + o0_l,
              0,
              (1 << slice.bit_depth_y) - 1,
            ) as u8;
          }
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            if log_wdcb >= 1 {
              pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                ((pred_part_l1cb[(y * part_width_c + x) as usize] as isize * w1_cb + (1 << (log_wdcb - 1))) >> log_wdcb) + o1_cb,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            } else {
              pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                pred_part_l1cb[(y * part_width_c + x) as usize] as isize * w1_cb + o1_cb,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            }

            if log_wdcr >= 1 {
              pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                ((pred_part_l1cr[(y * part_width_c + x) as usize] as isize * w1_cr + (1 << (log_wdcr - 1))) >> log_wdcr) + o1_cr,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            } else {
              pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
                [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
                pred_part_l1cr[(y * part_width_c + x) as usize] as isize * w1_cr + o1_cr,
                0,
                (1 << slice.bit_depth_c) - 1,
              ) as u8;
            }
          }
        }
      }
    } else if pred_flag_l0 == 1 && pred_flag_l1 == 1 {
      for y in 0..part_height {
        for x in 0..part_width {
          pred_part_l[(x_p + x_s + x) as usize][(y_p + y_s + y) as usize] = clamp(
            ((pred_part_l0l[(y * part_width + x) as usize] as isize * w0_l
              + pred_part_l1l[(y * part_width + x) as usize] as isize * w1_l
              + (1 << (log_wdl)))
              >> (log_wdl + 1))
              + ((o0_l + o1_l + 1) >> 1),
            0,
            (1 << slice.bit_depth_y) - 1,
          ) as u8;
        }
      }

      if slice.chroma_array_type != 0 {
        for y in 0..part_height_c {
          for x in 0..part_width_c {
            pred_part_cb[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
              ((pred_part_l0cb[(y * part_width_c + x) as usize] as isize * w0_cb
                + pred_part_l1cb[(y * part_width_c + x) as usize] as isize * w1_cb
                + (1 << (log_wdcb)))
                >> (log_wdcb + 1))
                + ((o0_cb + o1_cb + 1) >> 1),
              0,
              (1 << slice.bit_depth_c) - 1,
            ) as u8;

            pred_part_cr[(x_p / slice.sub_width_c + x_s / slice.sub_width_c + x) as usize]
              [(y_p / slice.sub_height_c + y_s / slice.sub_height_c + y) as usize] = clamp(
              ((pred_part_l0cr[(y * part_width_c + x) as usize] as isize * w0_cr
                + pred_part_l1cr[(y * part_width_c + x) as usize] as isize * w1_cr
                + (1 << (log_wdcr)))
                >> (log_wdcr + 1))
                + ((o0_cr + o1_cr + 1) >> 1),
              0,
              (1 << slice.bit_depth_c) - 1,
            ) as u8;
          }
        }
      }
    }
  }
}
