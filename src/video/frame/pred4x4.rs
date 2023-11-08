use super::Frame;
use crate::video::slice::{macroblock::MbPosition, Slice};
use crate::{
  math::{clamp, inverse_raster_scan},
  video::slice::macroblock::Macroblock,
};
use std::ops::{Index, IndexMut};

impl Frame {
  /// 8.3.1.2 Intra_4x4 sample prediction
  pub fn intra4x4_prediction(&mut self, slice: &mut Slice, luma4x4_blk_idx: usize, is_luma: bool) {
    const REFERENCE_COORDINATE_X: [isize; 13] = [-1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7];
    const REFERENCE_COORDINATE_Y: [isize; 13] = [-1, 0, 1, 2, 3, -1, -1, -1, -1, -1, -1, -1, -1];

    let x_o = inverse_raster_scan(luma4x4_blk_idx as isize / 4, 8, 8, 16, 0)
      + inverse_raster_scan(luma4x4_blk_idx as isize % 4, 4, 4, 8, 0);
    let y_o = inverse_raster_scan(luma4x4_blk_idx as isize / 4, 8, 8, 16, 1)
      + inverse_raster_scan(luma4x4_blk_idx as isize % 4, 4, 4, 8, 1);

    let mut samples = [-1isize; 45];

    for i in 0..13 {
      let x = REFERENCE_COORDINATE_X[i];
      let y = REFERENCE_COORDINATE_Y[i];
      let x_n = x_o + x;
      let y_n = y_o + y;

      let (max_w, max_h) = if is_luma {
        (16, 16)
      } else {
        (slice.mb_width_c as isize, slice.mb_height_c as isize)
      };

      let pos_a = MbPosition::from_coords(x_n, y_n, max_w, max_h);
      let mb_n = pos_a
        .map(|pos| slice.mb_nb_p(pos, 0))
        .unwrap_or_else(|| Macroblock::unavailable(0));
      let (x_w, y_w) = MbPosition::coords(x_n, y_n, max_w, max_h);

      if mb_n.mb_type.is_unavailable()
        || (mb_n.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
        || (slice.mb().mb_type.is_si() && slice.pps.constrained_intra_pred_flag)
        || (x > 3) && (luma4x4_blk_idx == 3 || luma4x4_blk_idx == 11)
      {
        *samples.p(x, y) = -1;
      } else {
        let mbaddr_n = mb_n.index(&slice.macroblocks) as usize;
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

        *samples.p(x, y) = self.luma_data[(x_m + x_w) as usize][(y_m + y_w) as usize] as isize;
      }
    }

    if *samples.p(4, -1) < 0
      && *samples.p(5, -1) < 0
      && *samples.p(6, -1) < 0
      && *samples.p(7, -1) < 0
      && *samples.p(3, -1) >= 0
    {
      *samples.p(4, -1) = *samples.p(3, -1);
      *samples.p(5, -1) = *samples.p(3, -1);
      *samples.p(6, -1) = *samples.p(3, -1);
      *samples.p(7, -1) = *samples.p(3, -1);
    }

    self.intra4x4_pred_mode(slice, luma4x4_blk_idx, is_luma);

    const INTRA_4X4_VERTICAL: isize = 0;
    const INTRA_4X4_HORIZONTAL: isize = 1;
    const INTRA_4X4_DC: isize = 2;
    const INTRA_4X4_DIAGONAL_DOWN_LEFT: isize = 3;
    const INTRA_4X4_DIAGONAL_DOWN_RIGHT: isize = 4;
    const INTRA_4X4_VERTICAL_RIGHT: isize = 5;
    const INTRA_4X4_HORIZONTAL_DOWN: isize = 6;
    const INTRA_4X4_VERTICAL_LEFT: isize = 7;
    const INTRA_4X4_HORIZONTAL_UP: isize = 8;

    let intra4x4_pred_mode = slice.mb().intra4x4_pred_mode[luma4x4_blk_idx];

    if intra4x4_pred_mode == INTRA_4X4_VERTICAL {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
      {
        for y in 0..4 {
          for x in 0..4 {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x][y] = *samples.p(x as isize, -1);
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_HORIZONTAL {
      if *samples.p(-1, 0) >= 0
        && *samples.p(-1, 1) >= 0
        && *samples.p(-1, 2) >= 0
        && *samples.p(-1, 3) >= 0
      {
        for y in 0..4 {
          for x in 0..4 {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x][y] = *samples.p(-1, y as isize);
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_DC {
      let val;
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(-1, 0) >= 0
        && *samples.p(-1, 1) >= 0
        && *samples.p(-1, 2) >= 0
        && *samples.p(-1, 3) >= 0
      {
        val = (*samples.p(0, -1)
          + *samples.p(1, -1)
          + *samples.p(2, -1)
          + *samples.p(3, -1)
          + *samples.p(-1, 0)
          + *samples.p(-1, 1)
          + *samples.p(-1, 2)
          + *samples.p(-1, 3)
          + 4)
          >> 3;
      } else if (*samples.p(0, -1) < 0
        || *samples.p(1, -1) < 0
        || *samples.p(2, -1) < 0
        || *samples.p(3, -1) < 0)
        && (*samples.p(-1, 0) >= 0
          && *samples.p(-1, 1) >= 0
          && *samples.p(-1, 2) >= 0
          && *samples.p(-1, 3) >= 0)
      {
        val =
          (*samples.p(-1, 0) + *samples.p(-1, 1) + *samples.p(-1, 2) + *samples.p(-1, 3) + 2) >> 2;
      } else if (*samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0)
        && (*samples.p(-1, 0) < 0
          || *samples.p(-1, 1) < 0
          || *samples.p(-1, 2) < 0
          || *samples.p(-1, 3) < 0)
      {
        val =
          (*samples.p(0, -1) + *samples.p(1, -1) + *samples.p(2, -1) + *samples.p(3, -1) + 2) >> 2;
      } else {
        val = 1 << (slice.bit_depth_y - 1);
      }

      for x in 0..4 {
        for y in 0..4 {
          slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x][y] = val;
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_DIAGONAL_DOWN_LEFT {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(4, -1) >= 0
        && *samples.p(5, -1) >= 0
        && *samples.p(6, -1) >= 0
        && *samples.p(7, -1) >= 0
      {
        for y in 0..4isize {
          for x in 0..4isize {
            if x == 3 && y == 3 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(6, -1) + 3 * *samples.p(7, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(x + y, -1)
                + 2 * *samples.p(x + y + 1, -1)
                + *samples.p(x + y + 2, -1)
                + 2)
                >> 2;
            }
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_DIAGONAL_DOWN_RIGHT {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(-1, -1) >= 0
        && *samples.p(-1, 0) >= 0
        && *samples.p(-1, 1) >= 0
        && *samples.p(-1, 2) >= 0
        && *samples.p(-1, 3) >= 0
      {
        for y in 0..=3isize {
          for x in 0..=3isize {
            if x > y {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(x - y - 2, -1)
                + 2 * *samples.p(x - y - 1, -1)
                + *samples.p(x - y, -1)
                + 2)
                >> 2;
            } else if x < y {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(-1, y - x - 2)
                + 2 * *samples.p(-1, y - x - 1)
                + *samples.p(-1, y - x)
                + 2)
                >> 2;
            } else {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(0, -1) + 2 * *samples.p(-1, -1) + *samples.p(-1, 0) + 2) >> 2;
            }
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_VERTICAL_RIGHT {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(-1, -1) >= 0
        && *samples.p(-1, 0) >= 0
        && *samples.p(-1, 1) >= 0
        && *samples.p(-1, 2) >= 0
        && *samples.p(-1, 3) >= 0
      {
        for y in 0..=3isize {
          for x in 0..=3isize {
            let z_vr = 2 * x - y;

            if z_vr == 0 || z_vr == 2 || z_vr == 4 || z_vr == 6 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(x - (y >> 1) - 1, -1) + *samples.p(x - (y >> 1), -1) + 1) >> 1;
            } else if z_vr == 1 || z_vr == 3 || z_vr == 5 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(x - (y >> 1) - 2, -1)
                + 2 * *samples.p(x - (y >> 1) - 1, -1)
                + *samples.p(x - (y >> 1), -1)
                + 2)
                >> 2;
            } else if z_vr == -1 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(-1, 0) + 2 * *samples.p(-1, -1) + *samples.p(0, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(-1, y - 1) + 2 * *samples.p(-1, y - 2) + *samples.p(-1, y - 3) + 2)
                  >> 2;
            }
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_HORIZONTAL_DOWN {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(-1, -1) >= 0
        && *samples.p(-1, 0) >= 0
        && *samples.p(-1, 1) >= 0
        && *samples.p(-1, 2) >= 0
        && *samples.p(-1, 3) >= 0
      {
        for y in 0..=3isize {
          for x in 0..=3isize {
            let z_hd = 2 * y - x;

            if z_hd == 0 || z_hd == 2 || z_hd == 4 || z_hd == 6 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(-1, y - (x >> 1) - 1) + *samples.p(-1, y - (x >> 1)) + 1) >> 1;
            } else if z_hd == 1 || z_hd == 3 || z_hd == 5 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(-1, y - (x >> 1) - 2)
                + 2 * *samples.p(-1, y - (x >> 1) - 1)
                + *samples.p(-1, y - (x >> 1))
                + 2)
                >> 2;
            } else if z_hd == -1 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(-1, 0) + 2 * *samples.p(-1, -1) + *samples.p(0, -1) + 2) >> 2;
            } else {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(x - 1, -1) + 2 * *samples.p(x - 2, -1) + *samples.p(x - 3, -1) + 2)
                  >> 2;
            }
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_VERTICAL_LEFT {
      if *samples.p(0, -1) >= 0
        && *samples.p(1, -1) >= 0
        && *samples.p(2, -1) >= 0
        && *samples.p(3, -1) >= 0
        && *samples.p(4, -1) >= 0
        && *samples.p(5, -1) >= 0
        && *samples.p(6, -1) >= 0
        && *samples.p(7, -1) >= 0
      {
        for y in 0..=3isize {
          for x in 0..=3isize {
            if y == 0 || y == 2 {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
                (*samples.p(x + (y >> 1), -1) + *samples.p(x + (y >> 1) + 1, -1) + 1) >> 1;
            } else {
              slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
                .p(x + (y >> 1), -1)
                + 2 * *samples.p(x + (y >> 1) + 1, -1)
                + *samples.p(x + (y >> 1) + 2, -1)
                + 2)
                >> 2;
            }
          }
        }
      }
    } else if intra4x4_pred_mode == INTRA_4X4_HORIZONTAL_UP
      && *samples.p(-1, 0) >= 0
      && *samples.p(-1, 1) >= 0
      && *samples.p(-1, 2) >= 0
      && *samples.p(-1, 3) >= 0
    {
      for y in 0..=3isize {
        for x in 0..=3isize {
          let z_hu = x + 2 * y;

          if z_hu == 0 || z_hu == 2 || z_hu == 4 {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
              (*samples.p(-1, y + (x >> 1)) + *samples.p(-1, y + (x >> 1) + 1) + 1) >> 1;
          } else if z_hu == 1 || z_hu == 3 {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] = (*samples
              .p(-1, y + (x >> 1))
              + 2 * *samples.p(-1, y + (x >> 1) + 1)
              + *samples.p(-1, y + (x >> 1) + 2)
              + 2)
              >> 2;
          } else if z_hu == 5 {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
              (*samples.p(-1, 2) + 3 * *samples.p(-1, 3) + 2) >> 2;
          } else {
            slice.mb_mut().luma_pred_samples[luma4x4_blk_idx][x as usize][y as usize] =
              *samples.p(-1, 3);
          }
        }
      }
    }
  }

  /// 8.3.1.1 Derivation process for intra4x4_pred_mode
  pub fn intra4x4_pred_mode(&mut self, slice: &mut Slice, luma4x4_block_idx: usize, is_luma: bool) {
    const INTRA4X4_DC: isize = 2;
    let x = inverse_raster_scan(luma4x4_block_idx as isize / 4, 8, 8, 16, 0)
      + inverse_raster_scan(luma4x4_block_idx as isize % 4, 4, 4, 8, 0);
    let y = inverse_raster_scan(luma4x4_block_idx as isize / 4, 8, 8, 16, 1)
      + inverse_raster_scan(luma4x4_block_idx as isize % 4, 4, 4, 8, 1);

    let (max_w, max_h) = if is_luma {
      (16, 16)
    } else {
      (slice.mb_width_c as isize, slice.mb_height_c as isize)
    };

    let mb_a = slice.mb_nb_p(MbPosition::A, 0);
    let luma4x4_block_idx_a = MbPosition::blk_idx4x4(x - 1, y, max_w, max_h);

    let mb_b = slice.mb_nb_p(MbPosition::B, 0);
    let luma4x4_block_idx_b = MbPosition::blk_idx4x4(x, y - 1, max_w, max_h);

    let dc_pred_mode_predicted_flag = mb_a.mb_type.is_unavailable()
      || mb_b.mb_type.is_unavailable()
      || (mb_a.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag)
      || (mb_b.mb_type.mode().is_inter_frame() && slice.pps.constrained_intra_pred_flag);

    let intra_mxm_pred_mode_a;
    let intra_mxm_pred_mode_b;

    if dc_pred_mode_predicted_flag
      || (!mb_a.mb_type.mode().is_intra_4x4() && !mb_a.mb_type.mode().is_intra_8x8())
    {
      intra_mxm_pred_mode_a = INTRA4X4_DC;
    } else if mb_a.mb_type.mode().is_intra_4x4() {
      intra_mxm_pred_mode_a = mb_a.intra4x4_pred_mode[luma4x4_block_idx_a as usize];
    } else {
      intra_mxm_pred_mode_a = mb_a.intra8x8_pred_mode[luma4x4_block_idx_a as usize >> 2];
    }

    if dc_pred_mode_predicted_flag
      || (!mb_b.mb_type.mode().is_intra_4x4() && !mb_b.mb_type.mode().is_intra_8x8())
    {
      intra_mxm_pred_mode_b = INTRA4X4_DC;
    } else if mb_b.mb_type.mode().is_intra_4x4() {
      intra_mxm_pred_mode_b = mb_b.intra4x4_pred_mode[luma4x4_block_idx_b as usize];
    } else {
      intra_mxm_pred_mode_b = mb_b.intra8x8_pred_mode[luma4x4_block_idx_b as usize >> 2];
    }

    let pred_intra4x4_pred_mode = std::cmp::min(intra_mxm_pred_mode_a, intra_mxm_pred_mode_b);

    if slice.mb().prev_intra4x4_pred_mode_flag[luma4x4_block_idx] != 0 {
      slice.mb_mut().intra4x4_pred_mode[luma4x4_block_idx] = pred_intra4x4_pred_mode as isize;
    } else if (slice.mb().rem_intra4x4_pred_mode[luma4x4_block_idx] as isize)
      < pred_intra4x4_pred_mode
    {
      slice.mb_mut().intra4x4_pred_mode[luma4x4_block_idx] =
        slice.mb().rem_intra4x4_pred_mode[luma4x4_block_idx] as isize;
    } else {
      slice.mb_mut().intra4x4_pred_mode[luma4x4_block_idx] =
        slice.mb().rem_intra4x4_pred_mode[luma4x4_block_idx] as isize + 1;
    }
  }
}

trait SampleP: IndexMut<usize, Output = isize> + Index<usize, Output = isize> {
  fn p(&mut self, x: isize, y: isize) -> &mut isize {
    &mut self[(((y) + 1) * 9 + ((x) + 1)) as usize]
  }
}

impl<const N: usize> SampleP for [isize; N] {}
