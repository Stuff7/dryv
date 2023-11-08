use super::{BlockType, Frame};
use crate::math::clamp;
use crate::video::frame::{inverse_scanner4x4, inverse_scanner_8x8};
use crate::video::slice::Slice;

impl Frame {
  /// 8.5.9 Derivation process for scaling functions
  pub fn scaling(&mut self, slice: &Slice, is_luma: bool, is_chroma_cb: bool) {
    let mb_is_inter_flag = slice.mb().mb_type.mode().is_inter_frame();

    let i_y_cb_cr = if let Some(color_plane_id) = slice.color_plane_id {
      color_plane_id
    } else if is_luma {
      0
    } else if is_chroma_cb {
      1
    } else {
      2
    };

    let idx = (i_y_cb_cr + if mb_is_inter_flag { 3 } else { 0 }) as usize;
    let weight_scale4x4 = inverse_scanner4x4(&slice.scaling_list4x4[idx]);

    const V4X4: [[isize; 3]; 6] = [
      [10, 16, 13],
      [11, 18, 14],
      [13, 20, 16],
      [14, 23, 18],
      [16, 25, 20],
      [18, 29, 23],
    ];

    for m in 0..6 {
      for i in 0..4 {
        for j in 0..4 {
          if i % 2 == 0 && j % 2 == 0 {
            self.level_scale4x4[m][i][j] = weight_scale4x4[i][j] * V4X4[m][0];
          } else if i % 2 == 1 && j % 2 == 1 {
            self.level_scale4x4[m][i][j] = weight_scale4x4[i][j] * V4X4[m][1];
          } else {
            self.level_scale4x4[m][i][j] = weight_scale4x4[i][j] * V4X4[m][2];
          }
        }
      }
    }

    let idx = (2 * i_y_cb_cr + mb_is_inter_flag as u8) as usize;
    let weight_scale8x8 = inverse_scanner_8x8(&slice.scaling_list8x8[idx]);

    const V8X8: [[isize; 6]; 6] = [
      [20, 18, 32, 19, 25, 24],
      [22, 19, 35, 21, 28, 26],
      [26, 23, 42, 24, 33, 31],
      [28, 25, 45, 26, 35, 33],
      [32, 28, 51, 30, 40, 38],
      [36, 32, 58, 34, 46, 43],
    ];

    for m in 0..6 {
      for i in 0..8 {
        for j in 0..8 {
          if i % 4 == 0 && j % 4 == 0 {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][0];
          } else if i % 2 == 1 && j % 2 == 1 {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][1];
          } else if i % 4 == 2 && j % 4 == 2 {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][2];
          } else if (i % 4 == 0 && j % 2 == 1) || (i % 2 == 1 && j % 4 == 0) {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][3];
          } else if (i % 4 == 0 && j % 4 == 2) || (i % 4 == 2 && j % 4 == 0) {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][4];
          } else {
            self.level_scale8x8[m][i][j] = weight_scale8x8[i][j] * V8X8[m][5];
          }
        }
      }
    }
  }

  /// 8.5.1 Specification of transform decoding process for 4x4 luma residual blocks
  pub fn transform_for_4x4_luma_residual_blocks(&mut self, slice: &mut Slice) {
    if !slice.mb().mb_type.mode().is_intra_16x16() {
      self.scaling(slice, true, false);
      for luma4x4_blk_idx in 0..16 {
        let c = inverse_scanner4x4(&slice.mb().block_luma_4x4[0][luma4x4_blk_idx]);
        let r = self.scaling_and_transform4x4(slice, &c, true, false);

        if slice.mb().transform_bypass_mode_flag
          && slice.mb().mb_type.mode().is_intra_4x4()
          && (slice.mb().intra4x4_pred_mode[luma4x4_blk_idx] == 0
            || slice.mb().intra4x4_pred_mode[luma4x4_blk_idx] == 1)
        {
          todo!("Bypass transform decoding");
        }

        self.intra4x4_prediction(slice, luma4x4_blk_idx, true);

        let mut u = [0; 16];
        for i in 0..4 {
          for j in 0..4 {
            let idx = i * 4 + j;
            u[idx] = clamp(
              slice.mb().luma_pred_samples[luma4x4_blk_idx][j][i] + r[i][j],
              0,
              (1 << slice.bit_depth_y) - 1,
            );
          }
        }

        self.picture_construction(slice, &u, BlockType::B4x4, luma4x4_blk_idx, true, false);
      }
    }
  }

  /// 8.5.12 Scaling and transformation process for residual 4x4 blocks
  pub fn scaling_and_transform4x4(
    &self,
    slice: &mut Slice,
    c: &[[isize; 4]; 4],
    is_luma: bool,
    is_chroma_cb: bool,
  ) -> [[isize; 4]; 4] {
    chroma_quantization_parameters(slice, is_chroma_cb);
    let bit_depth = if is_luma {
      slice.bit_depth_y
    } else {
      slice.bit_depth_c
    };

    let s_mb_flag = slice.mb().mb_type.is_si()
      || (slice.slice_type.is_switching_p() && slice.mb().mb_type.mode().is_inter_frame());

    let q_p = if is_luma && !s_mb_flag {
      slice.mb().qp1y
    } else if is_luma && s_mb_flag {
      slice.qsy
    } else if !is_luma && !s_mb_flag {
      slice.mb().qp1c
    } else {
      slice.mb().qsc
    } as isize;

    let mut r = [[0; 4]; 4];
    if slice.mb().transform_bypass_mode_flag {
      r.copy_from_slice(c);
    } else {
      let mut d = [[0; 4]; 4];
      for i in 0..4 {
        for j in 0..4 {
          if i == 0
            && j == 0
            && ((is_luma && slice.mb().mb_type.mode().is_intra_16x16()) || !is_luma)
          {
            d[0][0] = c[0][0];
          } else if q_p >= 24 {
            d[i][j] = (c[i][j] * self.level_scale4x4[q_p as usize % 6][i][j]) << (q_p / 6 - 4);
          } else {
            d[i][j] = (c[i][j] * self.level_scale4x4[q_p as usize % 6][i][j]
              + (1 << (3 - q_p as u32 / 6)))
              >> (4 - q_p / 6);
          }
        }
      }

      let mut f = [[0; 4]; 4];
      let mut h = [[0; 4]; 4];
      for i in 0..4 {
        let ei0 = d[i][0] + d[i][2];
        let ei1 = d[i][0] - d[i][2];
        let ei2 = (d[i][1] >> 1) - d[i][3];
        let ei3 = d[i][1] + (d[i][3] >> 1);

        f[i][0] = ei0 + ei3;
        f[i][1] = ei1 + ei2;
        f[i][2] = ei1 - ei2;
        f[i][3] = ei0 - ei3;
      }

      for j in 0..=3 {
        let g0j = f[0][j] + f[2][j];
        let g1j = f[0][j] - f[2][j];
        let g2j = (f[1][j] >> 1) - f[3][j];
        let g3j = f[1][j] + (f[3][j] >> 1);

        h[0][j] = g0j + g3j;
        h[1][j] = g1j + g2j;
        h[2][j] = g1j - g2j;
        h[3][j] = g0j - g3j;
      }

      for i in 0..4 {
        for j in 0..4 {
          r[i][j] = (h[i][j] + 32) >> 6;
        }
      }
    }

    r
  }
}

pub fn get_qpc(slice: &Slice, is_chroma_cb: bool) -> isize {
  let qp_offset = if is_chroma_cb {
    slice.pps.chroma_qp_index_offset
  } else {
    slice
      .pps
      .extra_rbsp_data
      .as_ref()
      .map(|pps| pps.second_chroma_qp_index_offset)
      .unwrap_or(slice.pps.chroma_qp_index_offset)
  } as isize;

  let qpi = clamp(slice.mb().qpy + qp_offset, -slice.qp_bd_offset_c, 51);

  if qpi < 30 {
    qpi
  } else {
    const QPCS: [isize; 22] = [
      29, 30, 31, 32, 32, 33, 34, 34, 35, 35, 36, 36, 37, 37, 37, 38, 38, 38, 39, 39, 39, 39,
    ];
    QPCS[qpi as usize - 30]
  }
}

pub fn chroma_quantization_parameters(slice: &mut Slice, is_chroma_cb: bool) {
  slice.mb_mut().qpc = get_qpc(slice, is_chroma_cb);
  slice.mb_mut().qp1c = slice.mb().qpc + slice.qp_bd_offset_c;

  if slice.slice_type.is_switching() {
    slice.qsy = slice.mb().qpy;
    slice.mb_mut().qsc = slice.mb().qpc;
  }
}
