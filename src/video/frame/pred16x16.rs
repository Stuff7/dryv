use std::ops::{Index, IndexMut};

use super::{transform::chroma_quantization_parameters, Frame};
use crate::{
  math::{clamp, inverse_raster_scan},
  video::slice::{
    macroblock::{Macroblock, MbPosition},
    Slice,
  },
};

impl Frame {
  /// 8.3.3 Intra_16x16 prediction process for luma samples
  pub fn intra16x16_prediction(&mut self, slice: &mut Slice, is_luma: bool) {
    const REFERENCE_COORDINATE_X: [isize; 33] = [
      -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
      8, 9, 10, 11, 12, 13, 14, 15,
    ];
    const REFERENCE_COORDINATE_Y: [isize; 33] = [
      -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, -1, -1, -1, -1, -1, -1, -1, -1, -1,
      -1, -1, -1, -1, -1, -1, -1,
    ];

    let mut p = [-1; 17 * 17];
    for i in 0..33 {
      let max_w;
      let max_h;
      if is_luma {
        max_w = 16;
        max_h = 16;
      } else {
        max_h = slice.mb_height_c as isize;
        max_w = slice.mb_width_c as isize;
      }

      let x = REFERENCE_COORDINATE_X[i];
      let y = REFERENCE_COORDINATE_Y[i];

      let pos_n = MbPosition::from_coords(x, y, max_w, max_h);
      let mb_n = pos_n
        .map(|pos| slice.mb_nb_p(pos, 0))
        .unwrap_or_else(|| Macroblock::unavailable(0));
      let (xW, yW) = pos_n
        .map(|pos| pos.coords(max_w, max_h))
        .unwrap_or((-1, -1));
      let mbaddr_n = mb_n.index(&slice.macroblocks) as usize;

      if mb_n.mb_type.is_unavailable()
        || (mb_n.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
        || (slice.mb().mb_type.is_si() && slice.pps.constrained_intra_pred_flag)
      {
        *p.p(x, y) = -1;
      } else {
        let x_m =
          inverse_raster_scan(mbaddr_n, 16, 16, slice.pic_width_in_samples_l as usize, 0) as isize;
        let y_m =
          inverse_raster_scan(mbaddr_n, 16, 16, slice.pic_width_in_samples_l as usize, 1) as isize;

        *p.p(x, y) = self.luma_data[(x_m + xW) as usize][(y_m + yW) as usize] as i16;
      }
    }

    let intra16x16_pred_mode = slice.mb().mb_type.intra16x16_pred_mode();
    if intra16x16_pred_mode == 0 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(8, -1) >= 0
        && *p.p(9, -1) >= 0
        && *p.p(10, -1) >= 0
        && *p.p(11, -1) >= 0
        && *p.p(12, -1) >= 0
        && *p.p(13, -1) >= 0
        && *p.p(14, -1) >= 0
        && *p.p(15, -1) >= 0
      {
        for y in 0..16 {
          for x in 0..16 {
            slice.mb_mut().luma16x16_pred_samples[x][y] = *p.p(x as isize, -1);
          }
        }
      }
    } else if intra16x16_pred_mode == 1 {
      if *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
        && *p.p(-1, 8) >= 0
        && *p.p(-1, 9) >= 0
        && *p.p(-1, 10) >= 0
        && *p.p(-1, 11) >= 0
        && *p.p(-1, 12) >= 0
        && *p.p(-1, 13) >= 0
        && *p.p(-1, 14) >= 0
        && *p.p(-1, 15) >= 0
      {
        for y in 0..16 {
          for x in 0..16 {
            slice.mb_mut().luma16x16_pred_samples[x][y] = *p.p(-1, y as isize);
          }
        }
      }
    } else if intra16x16_pred_mode == 2 {
      let val;

      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(8, -1) >= 0
        && *p.p(9, -1) >= 0
        && *p.p(10, -1) >= 0
        && *p.p(11, -1) >= 0
        && *p.p(12, -1) >= 0
        && *p.p(13, -1) >= 0
        && *p.p(14, -1) >= 0
        && *p.p(15, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
        && *p.p(-1, 8) >= 0
        && *p.p(-1, 9) >= 0
        && *p.p(-1, 10) >= 0
        && *p.p(-1, 11) >= 0
        && *p.p(-1, 12) >= 0
        && *p.p(-1, 13) >= 0
        && *p.p(-1, 14) >= 0
        && *p.p(-1, 15) >= 0
      {
        val = (*p.p(0, -1)
          + *p.p(1, -1)
          + *p.p(2, -1)
          + *p.p(3, -1)
          + *p.p(4, -1)
          + *p.p(5, -1)
          + *p.p(6, -1)
          + *p.p(7, -1)
          + *p.p(8, -1)
          + *p.p(9, -1)
          + *p.p(10, -1)
          + *p.p(11, -1)
          + *p.p(12, -1)
          + *p.p(13, -1)
          + *p.p(14, -1)
          + *p.p(15, -1)
          + *p.p(-1, 0)
          + *p.p(-1, 1)
          + *p.p(-1, 2)
          + *p.p(-1, 3)
          + *p.p(-1, 4)
          + *p.p(-1, 5)
          + *p.p(-1, 6)
          + *p.p(-1, 7)
          + *p.p(-1, 8)
          + *p.p(-1, 9)
          + *p.p(-1, 10)
          + *p.p(-1, 11)
          + *p.p(-1, 12)
          + *p.p(-1, 13)
          + *p.p(-1, 14)
          + *p.p(-1, 15)
          + 16)
          >> 5;
      } else if !(*p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(8, -1) >= 0
        && *p.p(9, -1) >= 0
        && *p.p(10, -1) >= 0
        && *p.p(11, -1) >= 0
        && *p.p(12, -1) >= 0
        && *p.p(13, -1) >= 0
        && *p.p(14, -1) >= 0
        && *p.p(15, -1) >= 0)
        && (*p.p(-1, 0) >= 0
          && *p.p(-1, 1) >= 0
          && *p.p(-1, 2) >= 0
          && *p.p(-1, 3) >= 0
          && *p.p(-1, 4) >= 0
          && *p.p(-1, 5) >= 0
          && *p.p(-1, 6) >= 0
          && *p.p(-1, 7) >= 0
          && *p.p(-1, 8) >= 0
          && *p.p(-1, 9) >= 0
          && *p.p(-1, 10) >= 0
          && *p.p(-1, 11) >= 0
          && *p.p(-1, 12) >= 0
          && *p.p(-1, 13) >= 0
          && *p.p(-1, 14) >= 0
          && *p.p(-1, 15) >= 0)
      {
        val = (*p.p(-1, 0)
          + *p.p(-1, 1)
          + *p.p(-1, 2)
          + *p.p(-1, 3)
          + *p.p(-1, 4)
          + *p.p(-1, 5)
          + *p.p(-1, 6)
          + *p.p(-1, 7)
          + *p.p(-1, 8)
          + *p.p(-1, 9)
          + *p.p(-1, 10)
          + *p.p(-1, 11)
          + *p.p(-1, 12)
          + *p.p(-1, 13)
          + *p.p(-1, 14)
          + *p.p(-1, 15)
          + 8)
          >> 4;
      } else if (*p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(8, -1) >= 0
        && *p.p(9, -1) >= 0
        && *p.p(10, -1) >= 0
        && *p.p(11, -1) >= 0
        && *p.p(12, -1) >= 0
        && *p.p(13, -1) >= 0
        && *p.p(14, -1) >= 0
        && *p.p(15, -1) >= 0)
        && !(*p.p(-1, 0) >= 0
          && *p.p(-1, 1) >= 0
          && *p.p(-1, 2) >= 0
          && *p.p(-1, 3) >= 0
          && *p.p(-1, 4) >= 0
          && *p.p(-1, 5) >= 0
          && *p.p(-1, 6) >= 0
          && *p.p(-1, 7) >= 0
          && *p.p(-1, 8) >= 0
          && *p.p(-1, 9) >= 0
          && *p.p(-1, 10) >= 0
          && *p.p(-1, 11) >= 0
          && *p.p(-1, 12) >= 0
          && *p.p(-1, 13) >= 0
          && *p.p(-1, 14) >= 0
          && *p.p(-1, 15) >= 0)
      {
        val = (*p.p(0, -1)
          + *p.p(1, -1)
          + *p.p(2, -1)
          + *p.p(3, -1)
          + *p.p(4, -1)
          + *p.p(5, -1)
          + *p.p(6, -1)
          + *p.p(7, -1)
          + *p.p(8, -1)
          + *p.p(9, -1)
          + *p.p(10, -1)
          + *p.p(11, -1)
          + *p.p(12, -1)
          + *p.p(13, -1)
          + *p.p(14, -1)
          + *p.p(15, -1)
          + 8)
          >> 4;
      } else {
        val = 1 << (slice.bit_depth_y - 1);
      }

      for x in 0..16 {
        for y in 0..16 {
          slice.mb_mut().luma16x16_pred_samples[x][y] = val;
        }
      }
    } else if intra16x16_pred_mode == 3 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(8, -1) >= 0
        && *p.p(9, -1) >= 0
        && *p.p(10, -1) >= 0
        && *p.p(11, -1) >= 0
        && *p.p(12, -1) >= 0
        && *p.p(13, -1) >= 0
        && *p.p(14, -1) >= 0
        && *p.p(15, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
        && *p.p(-1, 8) >= 0
        && *p.p(-1, 9) >= 0
        && *p.p(-1, 10) >= 0
        && *p.p(-1, 11) >= 0
        && *p.p(-1, 12) >= 0
        && *p.p(-1, 13) >= 0
        && *p.p(-1, 14) >= 0
        && *p.p(-1, 15) >= 0
      {
        let mut h = 0;
        let mut v = 0;

        for x in 0..=7 {
          h += (x + 1) * (*p.p(8 + x, -1) - *p.p(6 - x, -1)) as isize;
        }

        for y in 0..=7 {
          v += (y + 1) * (*p.p(-1, 8 + y) - *p.p(-1, 6 - y)) as isize;
        }

        let a = 16 * (*p.p(-1, 15) + *p.p(15, -1)) as isize;
        let b = (5 * h + 32) >> 6;
        let c = (5 * v + 32) >> 6;

        for y in 0..16 {
          for x in 0..16 {
            slice.mb_mut().luma16x16_pred_samples[x][y] = clamp(
              (a + b * (x as isize - 7) + c * (y as isize - 7) + 16) >> 5,
              0,
              (1 << slice.bit_depth_y as isize) - 1,
            ) as i16;
          }
        }
      }
    }
  }

  /// 8.5.10 Scaling and transformation process for DC transform coefficients for Intra_16x16 macroblock type
  pub fn transform_intra16x16_dc(
    &mut self,
    slice: &mut Slice,
    c: &[[i16; 4]; 4],
    is_luma: bool,
    is_chroma_cb: bool,
  ) -> [[i16; 4]; 4] {
    let mut dc_y = [[0; 4]; 4];
    let q_p = if is_luma {
      slice.mb().qp1y
    } else {
      chroma_quantization_parameters(slice, is_chroma_cb);
      slice.mb().qp1c
    };

    if slice.mb().transform_bypass_mode_flag {
      dc_y.copy_from_slice(c);
    } else {
      const A: [[i16; 4]; 4] = [[1, 1, 1, 1], [1, 1, -1, -1], [1, -1, -1, 1], [1, -1, 1, -1]];

      let mut g = [[0; 4]; 4];
      let mut f = [[0; 4]; 4];
      for i in 0..4 {
        for j in 0..4 {
          for k in 0..4 {
            g[i][j] += A[i][k] * c[k][j];
          }
        }
      }
      for i in 0..4 {
        for j in 0..4 {
          for k in 0..4 {
            f[i][j] += g[i][k] * A[k][j];
          }
        }
      }

      if q_p >= 36 {
        for i in 0..4 {
          for j in 0..4 {
            dc_y[i][j] = (f[i][j] * self.level_scale4x4[q_p as usize % 6][0][0]) << (q_p / 6 - 6);
          }
        }
      } else {
        for i in 0..4 {
          for j in 0..4 {
            dc_y[i][j] = (f[i][j] * self.level_scale4x4[q_p as usize % 6][0][0]
              + (1 << (5 - q_p / 6)))
              >> (6 - q_p / 6);
          }
        }
      }
    }
    dc_y
  }
}

trait SampleP: IndexMut<usize, Output = i16> + Index<usize, Output = i16> {
  fn p(&mut self, x: isize, y: isize) -> &mut i16 {
    &mut self[(((y) + 1) * 17 + ((x) + 1)) as usize]
  }
}

impl<const N: usize> SampleP for [i16; N] {}
