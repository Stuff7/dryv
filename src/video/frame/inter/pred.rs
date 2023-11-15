use crate::{
  math::{clamp, inverse_raster_scan},
  video::{
    frame::{inverse_scanner4x4, inverse_scanner_8x8, BlockType},
    slice::Slice,
  },
};

use super::super::Frame;

impl Frame {
  pub fn inter_transform_for_8x8_luma_residual_blocks(
    &mut self,
    slice: &mut Slice,
    pred_part_l: &[[u8; 16]; 16],
  ) {
    self.scaling(slice, true, false);

    for luma8x8_blk_idx in 0..4 {
      let c = inverse_scanner_8x8(&slice.mb().block_luma_8x8[0][luma8x8_blk_idx]);
      let r = self.scaling_and_transform8x8(slice, &c, true, false);

      if slice.mb().transform_bypass_mode_flag
        && slice.mb().mode().is_intra_8x8()
        && (slice.mb().intra4x4_pred_mode[luma8x8_blk_idx] == 0
          || slice.mb().intra4x4_pred_mode[luma8x8_blk_idx] == 1)
      {
        todo!("Inter bypass transform decoding");
      }

      let x_o = inverse_raster_scan(luma8x8_blk_idx as isize, 8, 8, 16, 0) as usize;
      let y_o = inverse_raster_scan(luma8x8_blk_idx as isize, 8, 8, 16, 1) as usize;

      let mut u = [0; 64];

      for i in 0..8 {
        for j in 0..8 {
          u[i * 8 + j] = clamp(
            pred_part_l[x_o + j][y_o + i] as isize + r[i][j],
            0,
            (1 << slice.bit_depth_y) - 1,
          );
        }
      }
      self.picture_construction(slice, &u, BlockType::B8x8, luma8x8_blk_idx, true, false);
    }
  }

  pub fn inter_transform_for_4x4_luma_residual_blocks(
    &mut self,
    slice: &mut Slice,
    pred_part_l: &[[u8; 16]; 16],
  ) {
    if !slice.mb().mode().is_intra_16x16() {
      self.scaling(slice, true, false);
      for luma4x4_blk_idx in 0..16 {
        let c = inverse_scanner4x4(&slice.mb().block_luma_4x4[0][luma4x4_blk_idx]);
        let r = self.scaling_and_transform4x4(slice, &c, true, false);

        if slice.mb().transform_bypass_mode_flag
          && slice.mb().mode().is_intra_4x4()
          && (slice.mb().intra4x4_pred_mode[luma4x4_blk_idx] == 0
            || slice.mb().intra4x4_pred_mode[luma4x4_blk_idx] == 1)
        {
          todo!("Inter bypass transform decoding");
        }

        let x_o = (inverse_raster_scan(luma4x4_blk_idx as isize / 4, 8, 8, 16, 0)
          + inverse_raster_scan(luma4x4_blk_idx as isize % 4, 4, 4, 8, 0))
          as usize;
        let y_o = (inverse_raster_scan(luma4x4_blk_idx as isize / 4, 8, 8, 16, 1)
          + inverse_raster_scan(luma4x4_blk_idx as isize % 4, 4, 4, 8, 1))
          as usize;

        let mut u = [0; 16];
        for i in 0..4 {
          for j in 0..4 {
            u[i * 4 + j] = clamp(
              pred_part_l[x_o + j][y_o + i] as isize + r[i][j],
              0,
              (1 << slice.bit_depth_y) - 1,
            );
          }
        }

        self.picture_construction(slice, &u, BlockType::B4x4, luma4x4_blk_idx, true, false);
      }
    }
  }

  pub fn inter_transform_for_chroma_residual_blocks(
    &mut self,
    slice: &mut Slice,
    pred_part_c: &[[u8; 16]; 16],
    is_chroma_cb: bool,
  ) {
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

      let dc_c_to_chroma = [
        dc_c[0][0], dc_c[0][1], dc_c[1][0], dc_c[1][1], dc_c[2][0], dc_c[2][1], dc_c[3][0],
        dc_c[3][1],
      ];

      let mut r_mb = [[0; 16]; 8];

      for chroma4x4_blk_idx in 0..num_chroma4x4_blks {
        let mut chroma_list = [0; 16];
        chroma_list[0] = dc_c_to_chroma[chroma4x4_blk_idx];

        chroma_list[1..16]
          .copy_from_slice(&slice.mb().block_chroma_ac[i_cb_cr][chroma4x4_blk_idx][..(16 - 1)]);

        let c = inverse_scanner4x4(&chroma_list);
        let r = self.scaling_and_transform4x4(slice, &c, false, is_chroma_cb);

        let x_o = inverse_raster_scan(chroma4x4_blk_idx as isize, 4, 4, 8, 0) as usize;
        let y_o = inverse_raster_scan(chroma4x4_blk_idx as isize, 4, 4, 8, 1) as usize;

        for i in 0..4 {
          for j in 0..4 {
            r_mb[x_o + j][y_o + i] = r[i][j];
          }
        }
      }

      if slice.mb().transform_bypass_flag {
        todo!("Inter bypass transform decoding");
      }

      let mut u = vec![0; mb_width_c * mb_height_c];
      for i in 0..mb_width_c {
        for j in 0..mb_height_c {
          u[i * mb_width_c + j] = clamp(
            pred_part_c[j][i] as isize + r_mb[j][i],
            0,
            (1 << slice.bit_depth_c) - 1,
          );
        }
      }

      self.picture_construction(slice, &u, BlockType::B4x4, 0, false, is_chroma_cb);
    }
  }
}
