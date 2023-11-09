use std::ops::{Index, IndexMut};

use crate::{
  math::{clamp, inverse_raster_scan},
  video::slice::{
    macroblock::{Macroblock, MbPosition},
    Slice,
  },
};

use super::{inverse_scanner_8x8, transform::chroma_quantization_parameters, BlockType, Frame};

impl Frame {
  pub fn transform_for_8x8_luma_residual_blocks(&mut self, slice: &mut Slice) {
    self.scaling(slice, true, false);

    for luma8x8_blk_idx in 0..4 {
      let c = inverse_scanner_8x8(&slice.mb().block_luma_8x8[0][luma8x8_blk_idx]);
      let r = [[0; 8]; 8];
      self.scaling_and_transform8x8(slice, &c, true, false);

      if slice.mb().transform_bypass_mode_flag
        && slice.mb().mb_type.mode().is_intra_8x8()
        && (slice.mb().intra4x4_pred_mode[luma8x8_blk_idx] == 0
          || slice.mb().intra4x4_pred_mode[luma8x8_blk_idx] == 1)
      {
        todo!("Bypass transform decoding");
      }

      self.intra8x8_prediction(slice, luma8x8_blk_idx, true);

      let mut u = [0; 64];

      for i in 0..8 {
        for j in 0..8 {
          u[i * 8 + j] = clamp(
            slice.mb().luma8x8_pred_samples[luma8x8_blk_idx][j][i] + r[i][j],
            0,
            (1 << slice.bit_depth_y) - 1,
          );
        }
      }

      self.picture_construction(slice, &u, BlockType::B8x8, luma8x8_blk_idx, true, false);
    }
  }

  /// 8.5.13.1 Scaling process for residual 8x8 blocks
  pub fn scaling_and_transform8x8(
    &mut self,
    slice: &mut Slice,
    c: &[[isize; 8]; 8],
    is_luma: bool,
    is_chroma_cb: bool,
  ) -> [[isize; 8]; 8] {
    let mut r = [[0; 8]; 8];
    chroma_quantization_parameters(slice, is_chroma_cb);

    let bit_depth;
    let q_p;
    if is_luma {
      bit_depth = slice.bit_depth_y;
      q_p = slice.mb().qp1y as usize;
    } else {
      bit_depth = slice.bit_depth_c;
      q_p = slice.mb().qp1c as usize;
    }
    if slice.mb().transform_bypass_mode_flag {
      r.copy_from_slice(c);
    } else {
      let mut d = [[0; 8]; 8];

      for i in 0..8 {
        for j in 0..8 {
          if q_p >= 36 {
            d[i][j] = (c[i][j] * self.level_scale8x8[q_p % 6][i][j]) << (q_p / 6 - 6);
          } else {
            d[i][j] = (c[i][j] * self.level_scale8x8[q_p % 6][i][j] + (1 << (5 - q_p / 6)))
              >> (6 - q_p / 6);
          }
        }
      }

      let mut g = [[0; 8]; 8];
      let mut m = [[0; 8]; 8];

      for i in 0..8 {
        let ei0 = d[i][0] + d[i][4];
        let ei1 = -d[i][3] + d[i][5] - d[i][7] - (d[i][7] >> 1);
        let ei2 = d[i][0] - d[i][4];
        let ei3 = d[i][1] + d[i][7] - d[i][3] - (d[i][3] >> 1);
        let ei4 = (d[i][2] >> 1) - d[i][6];
        let ei5 = -d[i][1] + d[i][7] + d[i][5] + (d[i][5] >> 1);
        let ei6 = d[i][2] + (d[i][6] >> 1);
        let ei7 = d[i][3] + d[i][5] + d[i][1] + (d[i][1] >> 1);

        let fi0 = ei0 + ei6;
        let fi1 = ei1 + (ei7 >> 2);
        let fi2 = ei2 + ei4;
        let fi3 = ei3 + (ei5 >> 2);
        let fi4 = ei2 - ei4;
        let fi5 = (ei3 >> 2) - ei5;
        let fi6 = ei0 - ei6;
        let fi7 = ei7 - (ei1 >> 2);

        g[i][0] = fi0 + fi7;
        g[i][1] = fi2 + fi5;
        g[i][2] = fi4 + fi3;
        g[i][3] = fi6 + fi1;
        g[i][4] = fi6 - fi1;
        g[i][5] = fi4 - fi3;
        g[i][6] = fi2 - fi5;
        g[i][7] = fi0 - fi7;
      }

      for j in 0..8 {
        let h0j = g[0][j] + g[4][j];
        let h1j = -g[3][j] + g[5][j] - g[7][j] - (g[7][j] >> 1);
        let h2j = g[0][j] - g[4][j];
        let h3j = g[1][j] + g[7][j] - g[3][j] - (g[3][j] >> 1);
        let h4j = (g[2][j] >> 1) - g[6][j];
        let h5j = -g[1][j] + g[7][j] + g[5][j] + (g[5][j] >> 1);
        let h6j = g[2][j] + (g[6][j] >> 1);
        let h7j = g[3][j] + g[5][j] + g[1][j] + (g[1][j] >> 1);

        let k0j = h0j + h6j;
        let k1j = h1j + (h7j >> 2);
        let k2j = h2j + h4j;
        let k3j = h3j + (h5j >> 2);
        let k4j = h2j - h4j;
        let k5j = (h3j >> 2) - h5j;
        let k6j = h0j - h6j;
        let k7j = h7j - (h1j >> 2);

        m[0][j] = k0j + k7j;
        m[1][j] = k2j + k5j;
        m[2][j] = k4j + k3j;
        m[3][j] = k6j + k1j;
        m[4][j] = k6j - k1j;
        m[5][j] = k4j - k3j;
        m[6][j] = k2j - k5j;
        m[7][j] = k0j - k7j;
      }

      for i in 0..8 {
        for j in 0..8 {
          r[i][j] = (m[i][j] + 32) >> 6;
        }
      }
    }
    r
  }

  pub fn intra8x8_prediction(&mut self, slice: &mut Slice, luma8x8_blk_idx: usize, is_luma: bool) {
    const REFERENCE_COORDINATE_X: [isize; 25] = [
      -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];
    const REFERENCE_COORDINATE_Y: [isize; 25] = [
      -1, 0, 1, 2, 3, 4, 5, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ];

    let mut p = [-1; 9 * 17];
    let mut p1 = [-1; 9 * 17];

    let x_o = inverse_raster_scan(luma8x8_blk_idx as isize, 8, 8, 16, 0);
    let y_o = inverse_raster_scan(luma8x8_blk_idx as isize, 8, 8, 16, 1);

    for i in 0..25 {
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

      let x_n = x_o as isize + x;
      let y_n = y_o as isize + y;

      let pos_n = MbPosition::from_coords(x_n, y_n, max_w, max_h);
      let mb_n = pos_n
        .map(|pos| slice.mb_nb_p(pos, 0))
        .unwrap_or_else(|| Macroblock::unavailable(0));
      let (x_w, y_w) = MbPosition::coords(x_n, y_n, max_w, max_h);
      let mbaddr_n = mb_n.index(&slice.macroblocks) as usize;

      if mb_n.mb_type.is_unavailable()
        || (mb_n.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
      {
        *p.p(x, y) = -1;
      } else {
        let x_m = inverse_raster_scan(
          mbaddr_n as isize,
          16,
          16,
          slice.pic_width_in_samples_l as isize,
          0,
        );
        let y_m = inverse_raster_scan(
          mbaddr_n as isize,
          16,
          16,
          slice.pic_width_in_samples_l as isize,
          1,
        );

        *p.p(x, y) = self.luma_data[(x_m + x_w) as usize][(y_m + y_w) as usize] as isize;
      }
    }

    if *p.p(8, -1) < 0
      && *p.p(9, -1) < 0
      && *p.p(10, -1) < 0
      && *p.p(11, -1) < 0
      && *p.p(12, -1) < 0
      && *p.p(13, -1) < 0
      && *p.p(14, -1) < 0
      && *p.p(15, -1) < 0
      && *p.p(7, -1) >= 0
    {
      *p.p(8, -1) = *p.p(7, -1);
      *p.p(9, -1) = *p.p(7, -1);
      *p.p(10, -1) = *p.p(7, -1);
      *p.p(11, -1) = *p.p(7, -1);
      *p.p(12, -1) = *p.p(7, -1);
      *p.p(13, -1) = *p.p(7, -1);
      *p.p(14, -1) = *p.p(7, -1);
      *p.p(15, -1) = *p.p(7, -1);
    }

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
      if *p.p(-1, -1) >= 0 {
        *p1.p(0, -1) = (*p.p(-1, -1) + 2 * *p.p(0, -1) + *p.p(1, -1) + 2) >> 2;
      } else {
        *p1.p(0, -1) = (3 * *p.p(0, -1) + *p.p(1, -1) + 2) >> 2;
      }

      for x in 0..15 {
        *p1.p(x, -1) = (*p.p(x - 1, -1) + 2 * *p.p(x, -1) + *p.p(x + 1, -1) + 2) >> 2;
      }

      *p1.p(15, -1) = (*p.p(14, -1) + 3 * *p.p(15, -1) + 2) >> 2;
    }

    if *p.p(-1, -1) >= 0 {
      if *p.p(0, -1) < 0 || *p.p(-1, 0) < 0 {
        if *p.p(0, -1) >= 0 {
          *p1.p(-1, -1) = (3 * *p.p(-1, -1) + *p.p(0, -1) + 2) >> 2;
        } else if *p.p(0, -1) < 0 && *p.p(-1, 0) >= 0 {
          *p1.p(-1, -1) = (3 * *p.p(-1, -1) + *p.p(-1, 0) + 2) >> 2;
        } else {
          *p1.p(-1, -1) = *p.p(-1, -1);
        }
      } else {
        *p1.p(-1, -1) = (*p.p(0, -1) + 2 * *p.p(-1, -1) + *p.p(-1, 0) + 2) >> 2;
      }
    }

    if *p.p(-1, 0) >= 0
      && *p.p(-1, 1) >= 0
      && *p.p(-1, 2) >= 0
      && *p.p(-1, 3) >= 0
      && *p.p(-1, 4) >= 0
      && *p.p(-1, 5) >= 0
      && *p.p(-1, 6) >= 0
      && *p.p(-1, 7) >= 0
    {
      if *p.p(-1, -1) >= 0 {
        *p1.p(-1, 0) = (*p.p(-1, -1) + 2 * *p.p(-1, 0) + *p.p(-1, 1) + 2) >> 2;
      } else {
        *p1.p(-1, 0) = (3 * *p.p(-1, 0) + *p.p(-1, 1) + 2) >> 2;
      }

      for y in 1..7 {
        *p1.p(-1, y) = (*p.p(-1, y - 1) + 2 * *p.p(-1, y) + *p.p(-1, y + 1) + 2) >> 2;
      }

      *p1.p(-1, 7) = (*p.p(-1, 6) + 3 * *p.p(-1, 7) + 2) >> 2;
    }

    p.copy_from_slice(&p1);

    self.intra8x8_pred_mode(slice, luma8x8_blk_idx, is_luma);

    let intra8x8_pred_mode = slice.mb().intra8x8_pred_mode[luma8x8_blk_idx];

    if intra8x8_pred_mode == 0 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
      {
        for y in 0..8 {
          for x in 0..8 {
            slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x][y] = *p.p(x as isize, -1);
          }
        }
      }
    } else if intra8x8_pred_mode == 1 {
      if *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        for y in 0..8 {
          for x in 0..8 {
            slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x][y] = *p.p(-1, y as isize);
          }
        }
      }
    } else if intra8x8_pred_mode == 2 {
      let val;
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        val = (*p.p(0, -1)
          + *p.p(1, -1)
          + *p.p(2, -1)
          + *p.p(3, -1)
          + *p.p(4, -1)
          + *p.p(5, -1)
          + *p.p(6, -1)
          + *p.p(7, -1)
          + *p.p(-1, 0)
          + *p.p(-1, 1)
          + *p.p(-1, 2)
          + *p.p(-1, 3)
          + *p.p(-1, 4)
          + *p.p(-1, 5)
          + *p.p(-1, 6)
          + *p.p(-1, 7)
          + 8)
          >> 4;
      } else if (*p.p(0, -1) < 0
        || *p.p(1, -1) < 0
        || *p.p(2, -1) < 0
        || *p.p(3, -1) < 0
        || *p.p(4, -1) < 0
        || *p.p(5, -1) < 0
        || *p.p(6, -1) < 0
        || *p.p(7, -1) < 0)
        && (*p.p(-1, 0) >= 0
          && *p.p(-1, 1) >= 0
          && *p.p(-1, 2) >= 0
          && *p.p(-1, 3) >= 0
          && *p.p(-1, 4) >= 0
          && *p.p(-1, 5) >= 0
          && *p.p(-1, 6) >= 0
          && *p.p(-1, 7) >= 0)
      {
        val = (*p.p(-1, 0)
          + *p.p(-1, 1)
          + *p.p(-1, 2)
          + *p.p(-1, 3)
          + *p.p(-1, 4)
          + *p.p(-1, 5)
          + *p.p(-1, 6)
          + *p.p(-1, 7)
          + 4)
          >> 3;
      } else if (*p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0)
        && (*p.p(-1, 0) < 0
          || *p.p(-1, 1) < 0
          || *p.p(-1, 2) < 0
          || *p.p(-1, 3) < 0
          || *p.p(-1, 4) < 0
          || *p.p(-1, 5) < 0
          || *p.p(-1, 6) < 0
          || *p.p(-1, 7) < 0)
      {
        val = (*p.p(0, -1)
          + *p.p(1, -1)
          + *p.p(2, -1)
          + *p.p(3, -1)
          + *p.p(4, -1)
          + *p.p(5, -1)
          + *p.p(6, -1)
          + *p.p(7, -1)
          + 4)
          >> 3;
      } else {
        val = 1 << (slice.bit_depth_y - 1);
      }

      for y in 0..8 {
        for x in 0..8 {
          slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x][y] = val;
        }
      }
    } else if intra8x8_pred_mode == 3 {
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
        for y in 0..8isize {
          for x in 0..8isize {
            if x == 7 && y == 7 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(14, -1) + 3 * *p.p(15, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(x + y, -1) + 2 * *p.p(x + y + 1, -1) + *p.p(x + y + 2, -1) + 2) >> 2;
            }
          }
        }
      }
    } else if intra8x8_pred_mode == 4 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(-1, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        for y in 0..8isize {
          for x in 0..8isize {
            if x > y {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(x - y - 2, -1) + 2 * *p.p(x - y - 1, -1) + *p.p(x - y, -1) + 2) >> 2;
            } else if x < y {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, y - x - 2) + 2 * *p.p(-1, y - x - 1) + *p.p(-1, y - x) + 2) >> 2;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(0, -1) + 2 * *p.p(-1, -1) + *p.p(-1, 0) + 2) >> 2;
            }
          }
        }
      }
    } else if intra8x8_pred_mode == 5 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(-1, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        for y in 0..8isize {
          for x in 0..8isize {
            let z_vr = 2 * x - y;

            if z_vr == 0
              || z_vr == 2
              || z_vr == 4
              || z_vr == 6
              || z_vr == 8
              || z_vr == 10
              || z_vr == 12
              || z_vr == 14
            {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(x - (y >> 1) - 1, -1) + *p.p(x - (y >> 1), -1) + 1) >> 1;
            } else if z_vr == 1
              || z_vr == 3
              || z_vr == 5
              || z_vr == 7
              || z_vr == 9
              || z_vr == 11
              || z_vr == 13
            {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(x - (y >> 1) - 2, -1)
                + 2 * *p.p(x - (y >> 1) - 1, -1)
                + *p.p(x - (y >> 1), -1)
                + 2)
                >> 2;
            } else if z_vr == -1 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, 0) + 2 * *p.p(-1, -1) + *p.p(0, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(-1, y - 2 * x - 1)
                + 2 * *p.p(-1, y - 2 * x - 2)
                + *p.p(-1, y - 2 * x - 3)
                + 2)
                >> 2;
            }
          }
        }
      }
    } else if intra8x8_pred_mode == 6 {
      if *p.p(0, -1) >= 0
        && *p.p(1, -1) >= 0
        && *p.p(2, -1) >= 0
        && *p.p(3, -1) >= 0
        && *p.p(4, -1) >= 0
        && *p.p(5, -1) >= 0
        && *p.p(6, -1) >= 0
        && *p.p(7, -1) >= 0
        && *p.p(-1, -1) >= 0
        && *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        for y in 0..8isize {
          for x in 0..8isize {
            let z_hd = 2 * y - x;

            if z_hd == 0
              || z_hd == 2
              || z_hd == 4
              || z_hd == 6
              || z_hd == 8
              || z_hd == 10
              || z_hd == 12
              || z_hd == 14
            {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, y - (x >> 1) - 1) + *p.p(-1, y - (x >> 1)) + 1) >> 1;
            } else if z_hd == 1
              || z_hd == 3
              || z_hd == 5
              || z_hd == 7
              || z_hd == 9
              || z_hd == 11
              || z_hd == 13
            {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(-1, y - (x >> 1) - 2)
                + 2 * *p.p(-1, y - (x >> 1) - 1)
                + *p.p(-1, y - (x >> 1))
                + 2)
                >> 2;
            } else if z_hd == -1 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, 0) + 2 * *p.p(-1, -1) + *p.p(0, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(x - 2 * y - 1, -1)
                + 2 * *p.p(x - 2 * y - 2, -1)
                + *p.p(x - 2 * y - 3, -1)
                + 2)
                >> 2;
            }
          }
        }
      }
    } else if intra8x8_pred_mode == 7 {
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
        for y in 0..8isize {
          for x in 0..8isize {
            if y == 0 || y == 2 || y == 4 || y == 6 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(x + (y >> 1), -1) + *p.p(x + (y >> 1) + 1, -1) + 1) >> 1;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(x + (y >> 1), -1)
                + 2 * *p.p(x + (y >> 1) + 1, -1)
                + *p.p(x + (y >> 1) + 2, -1)
                + 2)
                >> 2;
            }
          }
        }
      }
    } else if intra8x8_pred_mode == 8 {
      if *p.p(-1, 0) >= 0
        && *p.p(-1, 1) >= 0
        && *p.p(-1, 2) >= 0
        && *p.p(-1, 3) >= 0
        && *p.p(-1, 4) >= 0
        && *p.p(-1, 5) >= 0
        && *p.p(-1, 6) >= 0
        && *p.p(-1, 7) >= 0
      {
        for y in 0..8isize {
          for x in 0..8isize {
            let z_hu = x + 2 * y;

            if z_hu == 0
              || z_hu == 2
              || z_hu == 4
              || z_hu == 6
              || z_hu == 8
              || z_hu == 10
              || z_hu == 12
            {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, y + (x >> 1)) + *p.p(-1, y + (x >> 1) + 1) + 1) >> 1;
            } else if z_hu == 1 || z_hu == 3 || z_hu == 5 || z_hu == 7 || z_hu == 9 || z_hu == 11 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] = (*p
                .p(-1, y + (x >> 1))
                + 2 * *p.p(-1, y + (x >> 1) + 1)
                + *p.p(-1, y + (x >> 1) + 2)
                + 2)
                >> 2;
            } else if z_hu == 13 {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                (*p.p(-1, 6) + 3 * *p.p(-1, 7) + 2) >> 2;
            } else {
              slice.mb_mut().luma8x8_pred_samples[luma8x8_blk_idx][x as usize][y as usize] =
                *p.p(-1, 7);
            }
          }
        }
      }
    } else {
      panic!("Could not do 8x8 prediction");
    }
  }

  pub fn intra8x8_pred_mode(&mut self, slice: &mut Slice, luma8x8_blk_idx: usize, is_luma: bool) {
    let max_w;
    let max_h;
    if is_luma {
      max_w = 16;
      max_h = 16;
    } else {
      max_h = slice.mb_height_c as isize;
      max_w = slice.mb_width_c as isize;
    }
    let x = (luma8x8_blk_idx % 2) * 8;
    let y = (luma8x8_blk_idx / 2) * 8;

    let pos_a = MbPosition::from_coords(x as isize + (-1), y as isize, max_w, max_h);
    let mb_a = pos_a
      .map(|pos| slice.mb_nb_p(pos, 0))
      .unwrap_or_else(|| Macroblock::unavailable(0));
    let luma8x8_blk_idx_a = mb_a.blk_idx8x8(x as isize - 1, y as isize, max_w, max_h);

    let pos_b = MbPosition::from_coords(x as isize, y as isize + (-1), max_w, max_h);
    let mb_b = pos_b
      .map(|pos| slice.mb_nb_p(pos, 0))
      .unwrap_or_else(|| Macroblock::unavailable(0));
    let luma8x8_blk_idx_b = mb_b.blk_idx8x8(x as isize, y as isize - 1, max_w, max_h);

    let dc_pred_mode_predicted_flag;

    if mb_a.mb_type.is_unavailable()
      || mb_b.mb_type.is_unavailable()
      || (mb_a.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
      || (mb_b.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
    {
      dc_pred_mode_predicted_flag = true;
    } else {
      dc_pred_mode_predicted_flag = false;
    }

    let intra_mx_mpred_mode_a;
    let intra_mx_mpred_mode_b;

    if dc_pred_mode_predicted_flag
      || (!mb_a.mb_type.mode().is_intra_4x4() && !mb_a.mb_type.mode().is_intra_8x8())
    {
      intra_mx_mpred_mode_a = 2;
    } else if mb_a.mb_type.mode().is_intra_8x8() {
      intra_mx_mpred_mode_a = mb_a.intra8x8_pred_mode[luma8x8_blk_idx_a as usize];
    } else {
      let n = 1;
      intra_mx_mpred_mode_a = mb_a.intra4x4_pred_mode[luma8x8_blk_idx_a as usize * 4 + n];
    }

    if dc_pred_mode_predicted_flag
      || (!mb_b.mb_type.mode().is_intra_4x4() && !mb_b.mb_type.mode().is_intra_8x8())
    {
      intra_mx_mpred_mode_b = 2;
    } else if mb_b.mb_type.mode().is_intra_8x8() {
      intra_mx_mpred_mode_b = mb_b.intra8x8_pred_mode[luma8x8_blk_idx_b as usize];
    } else {
      let n = 2;
      intra_mx_mpred_mode_b = mb_b.intra4x4_pred_mode[luma8x8_blk_idx_b as usize * 4 + n];
    }
    let predintra8x8_pred_mode = std::cmp::min(intra_mx_mpred_mode_a, intra_mx_mpred_mode_b);

    if slice.mb().prev_intra8x8_pred_mode_flag[luma8x8_blk_idx] != 0 {
      slice.mb_mut().intra8x8_pred_mode[luma8x8_blk_idx] = predintra8x8_pred_mode;
    } else if (slice.mb().rem_intra8x8_pred_mode[luma8x8_blk_idx] as isize) < predintra8x8_pred_mode
    {
      slice.mb_mut().intra8x8_pred_mode[luma8x8_blk_idx] =
        slice.mb().rem_intra8x8_pred_mode[luma8x8_blk_idx] as isize;
    } else {
      slice.mb_mut().intra8x8_pred_mode[luma8x8_blk_idx] =
        slice.mb().rem_intra8x8_pred_mode[luma8x8_blk_idx] as isize + 1;
    }
  }
}

trait SampleP: IndexMut<usize, Output = isize> + Index<usize, Output = isize> {
  fn p(&mut self, x: isize, y: isize) -> &mut isize {
    &mut self[(((y) + 1) * 17 + ((x) + 1)) as usize]
  }
}

impl<const N: usize> SampleP for [isize; N] {}
