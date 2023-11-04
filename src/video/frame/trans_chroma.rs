use std::ops::{Index, IndexMut};

use super::{inverse_scanner4x4, transform::chroma_quantization_parameters, BlockType, Frame};
use crate::{
  math::{clamp, inverse_raster_scan},
  video::slice::{
    macroblock::{Macroblock, MbPosition},
    Slice,
  },
};

impl Frame {
  /// 8.5.4 Specification of transform decoding process for chroma samples
  fn transform_chroma_samples(&mut self, slice: &mut Slice, is_chroma_cb: bool) {
    if slice.chroma_array_type == 0 {
      panic!("Transform decoding process for chroma samples invoked for chroma_array_type 0");
    }

    if slice.chroma_array_type == 3 {
      todo!("8.5.5 Transform decoding process for chroma samples with ChromaArrayType equal to 3");
    } else {
      let mb_width_c = slice.mb_width_c as usize;
      let mb_height_c = slice.mb_height_c as usize;
      let num_chroma4x4_blks = (mb_width_c / 4) * (mb_height_c / 4);
      let i_cb_cr = if is_chroma_cb { 0 } else { 1 };

      let mut dc_c = [[0; 2]; 4];
      if slice.chroma_array_type == 1 {
        let mut c = [[0; 2]; 2];

        c[0][0] = slice.mb().block_chroma_dc[i_cb_cr][0];
        c[0][1] = slice.mb().block_chroma_dc[i_cb_cr][1];
        c[1][0] = slice.mb().block_chroma_dc[i_cb_cr][2];
        c[1][1] = slice.mb().block_chroma_dc[i_cb_cr][3];
        dc_c = self.transform_chroma_dc(slice, &c, mb_width_c, mb_height_c, is_chroma_cb);
      } else if slice.chroma_array_type == 2 {
        let mut c = [[0; 2]; 4];
        c[0][0] = slice.mb().block_chroma_dc[i_cb_cr][0];
        c[0][1] = slice.mb().block_chroma_dc[i_cb_cr][1];
        c[1][0] = slice.mb().block_chroma_dc[i_cb_cr][2];
        c[1][1] = slice.mb().block_chroma_dc[i_cb_cr][3];
        c[2][0] = slice.mb().block_chroma_dc[i_cb_cr][4];
        c[2][1] = slice.mb().block_chroma_dc[i_cb_cr][5];
        c[3][0] = slice.mb().block_chroma_dc[i_cb_cr][6];
        c[3][1] = slice.mb().block_chroma_dc[i_cb_cr][7];
        dc_c = self.transform_chroma_dc(slice, &c, mb_width_c, mb_height_c, is_chroma_cb);
      }

      let dc_cto_chroma = [
        dc_c[0][0], dc_c[0][1], dc_c[1][0], dc_c[1][1], dc_c[2][0], dc_c[2][1], dc_c[3][0],
        dc_c[3][1],
      ];

      let mut r_mb = [[0; 16]; 8];

      for chroma4x4_blk_idx in 0..num_chroma4x4_blks as usize {
        let mut chroma_list = [0; 16];
        chroma_list[0] = dc_cto_chroma[chroma4x4_blk_idx];

        for k in 1..16 {
          chroma_list[k] = slice.mb().block_chroma_ac[i_cb_cr][chroma4x4_blk_idx][k - 1];
        }

        let c = inverse_scanner4x4(&chroma_list);
        let r = self.scaling_and_transform4x4(slice, &c, false, is_chroma_cb);

        let x_o = inverse_raster_scan(chroma4x4_blk_idx, 4, 4, 8, 0);
        let y_o = inverse_raster_scan(chroma4x4_blk_idx, 4, 4, 8, 1);

        for i in 0..4 {
          for j in 0..4 {
            r_mb[x_o + j][y_o + i] = r[i][j];
          }
        }
      }

      if slice.mb().transform_bypass_mode_flag {
        todo!("Bypass conversion");
      }
      self.intra_chroma_prediction(slice, is_chroma_cb);

      let mut u = vec![0; mb_width_c * mb_height_c];
      for i in 0..mb_width_c {
        for j in 0..mb_height_c {
          u[i * mb_width_c + j] = clamp(
            slice.mb().chroma_pred_samples[j][i] + r_mb[j][i],
            0,
            (1 << slice.bit_depth_c) - 1,
          );
        }
      }

      self.picture_construction(slice, &u, BlockType::B4x4, 0, false, is_chroma_cb);
    }
  }

  pub fn intra_chroma_prediction(&mut self, slice: &mut Slice, is_chroma_cb: bool) {
    if slice.chroma_array_type == 3 {
      todo!("Intra chroma prediction for chroma_array_type 3");
    } else {
      let mb_width_c = slice.mb_width_c as isize;
      let mb_height_c = slice.mb_height_c as isize;

      let max_samples_val = mb_width_c + mb_height_c + 1;
      let mut reference_coordinate_x = vec![0i16; max_samples_val as usize];
      let mut reference_coordinate_y = vec![0i16; max_samples_val as usize];

      for i in -1..mb_height_c as i16 {
        reference_coordinate_x[(i + 1) as usize] = -1;
        reference_coordinate_y[(i + 1) as usize] = i;
      }

      for i in 0..mb_width_c as i16 {
        reference_coordinate_x[(mb_height_c as i16 + 1 + i) as usize] = i;
        reference_coordinate_y[(mb_height_c as i16 + 1 + i) as usize] = -1;
      }

      let mut samples = vec![-1; ((mb_width_c + 1) * (mb_height_c + 1)) as usize];

      for i in 0..max_samples_val as usize {
        let x = reference_coordinate_x[i] as isize;
        let y = reference_coordinate_y[i] as isize;

        let pos_a = MbPosition::from_coords(x, y, mb_width_c, mb_height_c);
        let mb_n = pos_a
          .map(|pos| slice.mb_nb_p(pos, 0))
          .unwrap_or_else(|| Macroblock::unavailable(0));
        let (x_w, y_w) = pos_a
          .map(|pos| pos.coords(mb_width_c, mb_height_c))
          .unwrap_or((-1, -1));
        let mbaddr_n = mb_n.index(&slice.macroblocks) as usize;

        if mb_n.mb_type.is_unavailable()
          || (mb_n.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
          || (mb_n.mb_type.is_si()
            && slice.pps.constrained_intra_pred_flag
            && !slice.mb().mb_type.is_si())
        {
          *samples.p(x, y) = -1;
        } else {
          let x_l = inverse_raster_scan(mbaddr_n, 16, 16, slice.pic_width_in_samples_l as usize, 0);
          let y_l = inverse_raster_scan(mbaddr_n, 16, 16, slice.pic_width_in_samples_l as usize, 1);

          let x_m = (x_l >> 4) * mb_width_c as usize;
          let y_m = ((y_l >> 4) * mb_height_c as usize) + (y_l % 2);

          let p_x = (x_m as isize + x_w) as usize;
          let p_y = (y_m as isize + y_w) as usize;
          if is_chroma_cb {
            *samples.p(x, y) = self.chroma_cb_data[p_x][p_y] as i16;
          } else {
            *samples.p(x, y) = self.chroma_cr_data[p_x][p_y] as i16;
          }
        }
      }

      let intra_chroma_pred_mode = slice.mb().intra_chroma_pred_mode;

      if intra_chroma_pred_mode == 0 {
        for chroma4x4_blk_idx in 0..(1 << (slice.chroma_array_type + 1)) {
          let x_o = inverse_raster_scan(chroma4x4_blk_idx, 4, 4, 8, 0) as isize;
          let y_o = inverse_raster_scan(chroma4x4_blk_idx, 4, 4, 8, 1) as isize;

          let mut val = 0;
          if (x_o == 0 && y_o == 0) || (x_o > 0 && y_o > 0) {
            if *samples.p(x_o, -1) >= 0
              && *samples.p(1 + x_o, -1) >= 0
              && *samples.p(2 + x_o, -1) >= 0
              && *samples.p(3 + x_o, -1) >= 0
              && *samples.p(-1, y_o) >= 0
              && *samples.p(-1, 1 + y_o) >= 0
              && *samples.p(-1, 2 + y_o) >= 0
              && *samples.p(-1, 3 + y_o) >= 0
            {
              val = (*samples.p(x_o, -1)
                + *samples.p(1 + x_o, -1)
                + *samples.p(2 + x_o, -1)
                + *samples.p(3 + x_o, -1)
                + *samples.p(-1, y_o)
                + *samples.p(-1, 1 + y_o)
                + *samples.p(-1, 2 + y_o)
                + *samples.p(-1, 3 + y_o)
                + 4)
                >> 3;
            } else if !(*samples.p(x_o, -1) >= 0
              && *samples.p(1 + x_o, -1) >= 0
              && *samples.p(2 + x_o, -1) >= 0
              && *samples.p(3 + x_o, -1) >= 0)
              && (*samples.p(-1, y_o) >= 0
                && *samples.p(-1, 1 + y_o) >= 0
                && *samples.p(-1, 2 + y_o) >= 0
                && *samples.p(-1, 3 + y_o) >= 0)
            {
              val = (*samples.p(-1, y_o)
                + *samples.p(-1, 1 + y_o)
                + *samples.p(-1, 2 + y_o)
                + *samples.p(-1, 3 + y_o)
                + 2)
                >> 2;
            } else if (*samples.p(x_o, -1) > 0
              && *samples.p(1 + x_o, -1) > 0
              && *samples.p(2 + x_o, -1) > 0
              && *samples.p(3 + x_o, -1) > 0)
              && !(*samples.p(-1, y_o) > 0
                && *samples.p(-1, 1 + y_o) > 0
                && *samples.p(-1, 2 + y_o) > 0
                && *samples.p(-1, 3 + y_o) > 0)
            {
              val = (*samples.p(x_o, -1)
                + *samples.p(1 + x_o, -1)
                + *samples.p(2 + x_o, -1)
                + *samples.p(3 + x_o, -1)
                + 2)
                >> 2;
            } else {
              val = 1 << (slice.bit_depth_c - 1);
            }
          } else if x_o > 0 && y_o == 0 {
            if *samples.p(x_o, -1) >= 0
              && *samples.p(1 + x_o, -1) >= 0
              && *samples.p(2 + x_o, -1) >= 0
              && *samples.p(3 + x_o, -1) >= 0
            {
              val = (*samples.p(x_o, -1)
                + *samples.p(1 + x_o, -1)
                + *samples.p(2 + x_o, -1)
                + *samples.p(3 + x_o, -1)
                + 2)
                >> 2;
            } else if *samples.p(-1, y_o) >= 0
              && *samples.p(-1, 1 + y_o) >= 0
              && *samples.p(-1, 2 + y_o) >= 0
              && *samples.p(-1, 3 + y_o) > 0
            {
              val = (*samples.p(-1, y_o)
                + *samples.p(-1, 1 + y_o)
                + *samples.p(-1, 2 + y_o)
                + *samples.p(-1, 3 + y_o)
                + 2)
                >> 2;
            } else {
              val = 1 << (slice.bit_depth_c - 1);
            }
          } else if x_o == 0 && y_o > 0 {
            if *samples.p(-1, y_o) >= 0
              && *samples.p(-1, 1 + y_o) >= 0
              && *samples.p(-1, 2 + y_o) >= 0
              && *samples.p(-1, 3 + y_o) > 0
            {
              val = (*samples.p(-1, y_o)
                + *samples.p(-1, 1 + y_o)
                + *samples.p(-1, 2 + y_o)
                + *samples.p(-1, 3 + y_o)
                + 2)
                >> 2;
            } else if *samples.p(x_o, -1) >= 0
              && *samples.p(1 + x_o, -1) >= 0
              && *samples.p(2 + x_o, -1) >= 0
              && *samples.p(3 + x_o, -1) > 0
            {
              val = (*samples.p(x_o, -1)
                + *samples.p(1 + x_o, -1)
                + *samples.p(2 + x_o, -1)
                + *samples.p(3 + x_o, -1)
                + 2)
                >> 2;
            } else {
              val = 1 << (slice.bit_depth_c - 1);
            }
          }

          for y in 0..4 {
            for x in 0..4 {
              slice.mb_mut().chroma_pred_samples[(x + x_o) as usize][(y + y_o) as usize] = val;
            }
          }
        }
      } else if intra_chroma_pred_mode == 1 {
        let mut flag = true;

        for y in 0..mb_height_c {
          if *samples.p(-1, y) < 0 {
            flag = false;
            break;
          }
        }

        if flag {
          for y in 0..mb_height_c {
            for x in 0..mb_width_c {
              slice.mb_mut().chroma_pred_samples[x as usize][y as usize] = *samples.p(-1, y);
            }
          }
        }
      } else if intra_chroma_pred_mode == 2 {
        let mut flag = true;
        for x in 0..mb_width_c {
          if *samples.p(x, -1) < 0 {
            flag = false;
            break;
          }
        }
        if flag {
          for y in 0..mb_height_c {
            for x in 0..mb_width_c {
              slice.mb_mut().chroma_pred_samples[x as usize][y as usize] = *samples.p(x, -1);
            }
          }
        }
      } else if intra_chroma_pred_mode == 3 {
        let mut flag = true;
        for x in 0..mb_width_c {
          if *samples.p(x, -1) < 0 {
            flag = false;
            break;
          }
        }

        for y in -1..mb_height_c {
          if *samples.p(-1, y) < 0 {
            flag = false;
            break;
          }
        }

        if flag {
          let x_cf = if slice.chroma_array_type == 3 { 4 } else { 0 };
          let y_cf = if slice.chroma_array_type != 1 { 4 } else { 0 };

          let mut h = 0;
          let mut v = 0;

          for x1 in 0..=3 + x_cf {
            h += (x1 as i16 + 1) * (*samples.p(4 + x_cf + x1, -1) - *samples.p(2 + x_cf - x1, -1));
          }

          for y1 in 0..=3 + y_cf {
            v += (y1 as i16 + 1) * (*samples.p(-1, 4 + y_cf + y1) - *samples.p(-1, 2 + y_cf - y1));
          }

          let a = 16 * (*samples.p(-1, mb_height_c - 1) + *samples.p(mb_width_c - 1, -1));
          let b = ((34 - 29 * (slice.chroma_array_type == 3) as i16) * h + 32) >> 6;
          let c = ((34 - 29 * (slice.chroma_array_type != 1) as i16) * v + 32) >> 6;

          for y in 0..mb_height_c {
            for x in 0..mb_width_c {
              slice.mb_mut().chroma_pred_samples[x as usize][y as usize] = clamp(
                (a + b * (x as i16 - 3 - x_cf as i16) + c * (y as i16 - 3 - y_cf as i16) + 16) >> 5,
                0,
                (1 << slice.bit_depth_c) - 1,
              );
            }
          }
        }
      }
    }
  }

  /// 8.5.11.1 Transformation process for chroma DC transform coefficients
  pub fn transform_chroma_dc(
    &mut self,
    slice: &mut Slice,
    c: &[[i16; 2]],
    mb_width_c: usize,
    mb_height_c: usize,
    is_chroma_cb: bool,
  ) -> [[i16; 2]; 4] {
    let mut dc_c = [[0; 2]; 4];
    let bit_depth = slice.bit_depth_c;

    chroma_quantization_parameters(slice, is_chroma_cb);

    let q_p = slice.mb().qp1c as usize;

    if slice.mb().transform_bypass_mode_flag {
      for i in 0..mb_width_c / 4 {
        for j in 0..mb_height_c / 4 {
          dc_c[i][j] = c[i][j];
        }
      }
    } else if slice.chroma_array_type == 1 {
      let a = [[1, 1], [1, -1]];

      let mut g = [[0; 2]; 2];
      let mut f = [[0; 2]; 2];

      for i in 0..2 {
        for j in 0..2 {
          for k in 0..2 {
            g[i][j] += a[i][k] * c[k][j];
          }
        }
      }

      for i in 0..2 {
        for j in 0..2 {
          for k in 0..2 {
            f[i][j] += g[i][k] * a[k][j];
          }
        }
      }

      for i in 0..2 {
        for j in 0..2 {
          dc_c[i][j] = ((f[i][j] * self.level_scale4x4[q_p % 6][0][0]) << (q_p / 6)) >> 5;
        }
      }
    } else if slice.chroma_array_type == 2 {
      let a = [[1, 1, 1, 1], [1, 1, -1, -1], [1, -1, -1, 1], [1, -1, 1, -1]];
      let b = [[1, 1], [1, -1]];

      let mut g = [[0; 2]; 4];
      let mut f = [[0; 2]; 4];
      for i in 0..4 {
        for j in 0..2 {
          for k in 0..4 {
            g[i][j] += a[i][k] * c[k][j];
          }
        }
      }

      for i in 0..4 {
        for j in 0..2 {
          for k in 0..2 {
            f[i][j] += g[i][k] * b[k][j];
          }
        }
      }

      let q_pdc = q_p + 3;
      if q_pdc >= 36 {
        for i in 0..4 {
          for j in 0..2 {
            dc_c[i][j] = (f[i][j] * self.level_scale4x4[q_pdc % 6][0][0]) << (q_pdc / 6 - 6);
          }
        }
      } else {
        for i in 0..4 {
          for j in 0..2 {
            dc_c[i][j] = (f[i][j] * self.level_scale4x4[q_pdc % 6][0][0] + (1 << (5 - q_pdc / 6)))
              >> (6 - q_p / 6);
          }
        }
      }
    }

    dc_c
  }
}

trait SampleP: IndexMut<usize, Output = i16> + Index<usize, Output = i16> {
  fn p(&mut self, x: isize, y: isize) -> &mut i16 {
    &mut self[(((y) + 1) * 9 + ((x) + 1)) as usize]
  }
}

impl SampleP for Vec<i16> {}
