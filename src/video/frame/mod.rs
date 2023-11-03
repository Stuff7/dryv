use super::slice::Slice;
use crate::math::{clamp, inverse_raster_scan};

pub struct Frame {
  pub luma_data: Box<[Box<[u8]>]>,
  pub chroma_cb_data: Box<[Box<[u8]>]>,
  pub chroma_cr_data: Box<[Box<[u8]>]>,
  pub level_scale4x4: [[[i16; 4]; 4]; 6],
  pub level_scale8x8: [[[i16; 4]; 4]; 6],
}

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
    let weight_scale4x4 = inverse_scanner_4x4(&slice.scaling_list4x4[idx]);

    const V4X4: [[i16; 3]; 6] = [
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

    const V8X8: [[i16; 6]; 6] = [
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

  /// 8.5.12 Scaling and transformation process for residual 4x4 blocks
  pub fn scaling_and_transform4x4(
    &self,
    slice: &mut Slice,
    c: &[[i16; 4]; 4],
    is_luma: bool,
    is_chroma_cb: bool,
  ) -> [[i16; 4]; 4] {
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
      slice.mb().qsy
    } else if !is_luma && !s_mb_flag {
      slice.mb().qp1c
    } else {
      slice.mb().qsc
    };

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
              + (2i16.pow(3 - q_p as u32 / 6)))
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

  /// 8.5.14 Picture construction process prior to deblocking filter process
  pub fn picture_construction(
    &mut self,
    slice: &Slice,
    u: &[u8],
    blk_type: BlockType,
    blk_idx: usize,
    is_luma: bool,
    is_chroma_cb: bool,
  ) {
    let x_p = inverse_raster_scan(
      slice.curr_mb_addr as usize,
      16,
      16,
      slice.pic_width_in_samples_l as usize,
      0,
    );
    let y_p = inverse_raster_scan(
      slice.curr_mb_addr as usize,
      16,
      16,
      slice.pic_width_in_samples_l as usize,
      1,
    );

    let mut x_o = 0;
    let mut y_o = 0;
    if is_luma {
      let n_e;
      if blk_type.is_16x16() {
        x_o = 0;
        y_o = 0;
        n_e = 16;
      } else if blk_type.is_4x4() {
        x_o = inverse_raster_scan(blk_idx / 4, 8, 8, 16, 0)
          + inverse_raster_scan(blk_idx % 4, 4, 4, 8, 0);
        y_o = inverse_raster_scan(blk_idx / 4, 8, 8, 16, 1)
          + inverse_raster_scan(blk_idx % 4, 4, 4, 8, 1);
        n_e = 4;
      } else {
        x_o = inverse_raster_scan(blk_idx, 8, 8, 16, 0);
        y_o = inverse_raster_scan(blk_idx, 8, 8, 16, 1);
        n_e = 8;
      }

      for i in 0..n_e {
        for j in 0..n_e {
          self.luma_data[x_p + x_o + j][y_p + y_o + i] = u[i * n_e + j];
        }
      }
    } else {
      let mb_width_c = slice.mb_width_c as usize;
      let mb_height_c = slice.mb_height_c as usize;

      if slice.chroma_array_type == 1 || slice.chroma_array_type == 2 {
        let chroma_data = if is_chroma_cb {
          &mut self.chroma_cb_data
        } else {
          &mut self.chroma_cr_data
        };

        for i in 0..mb_width_c {
          for j in 0..mb_height_c {
            chroma_data[x_p / slice.sub_width_c as usize + x_o + j]
              [y_p / slice.sub_height_c as usize + y_o + i] = u[i * mb_width_c + j];
          }
        }
      }
    }
  }
}

pub enum BlockType {
  B16x16,
  B8x8,
  B4x4,
}

impl BlockType {
  pub fn is_16x16(&self) -> bool {
    matches!(self, Self::B16x16)
  }

  pub fn is_8x8(&self) -> bool {
    matches!(self, Self::B8x8)
  }

  pub fn is_4x4(&self) -> bool {
    matches!(self, Self::B4x4)
  }
}

/// Zig-zag
fn inverse_scanner_4x4(value: &[i16; 16]) -> [[i16; 4]; 4] {
  let mut c = [[0; 4]; 4];

  c[0][0] = value[0];
  c[0][1] = value[1];
  c[1][0] = value[2];
  c[2][0] = value[3];

  c[1][1] = value[4];
  c[0][2] = value[5];
  c[0][3] = value[6];
  c[1][2] = value[7];

  c[2][1] = value[8];
  c[3][0] = value[9];
  c[3][1] = value[10];
  c[2][2] = value[11];

  c[1][3] = value[12];
  c[2][3] = value[13];
  c[3][2] = value[14];
  c[3][3] = value[15];

  c
}

/// 8x8 zig-zag scan
fn inverse_scanner_8x8(value: &[i16; 64]) -> [[i16; 8]; 8] {
  let mut c = [[0; 8]; 8];

  c[0][0] = value[0];
  c[0][1] = value[1];
  c[1][0] = value[2];
  c[2][0] = value[3];
  c[1][1] = value[4];
  c[0][2] = value[5];
  c[0][3] = value[6];
  c[1][2] = value[7];
  c[2][1] = value[8];
  c[3][0] = value[9];
  c[4][0] = value[10];
  c[3][1] = value[11];
  c[2][2] = value[12];
  c[1][3] = value[13];
  c[0][4] = value[14];
  c[0][5] = value[15];

  c[1][4] = value[16];
  c[2][3] = value[17];
  c[3][2] = value[18];
  c[4][1] = value[19];
  c[5][0] = value[20];
  c[6][0] = value[21];
  c[5][1] = value[22];
  c[4][2] = value[23];
  c[3][3] = value[24];
  c[2][4] = value[25];
  c[1][5] = value[26];
  c[0][6] = value[27];
  c[0][7] = value[28];
  c[1][6] = value[29];
  c[2][5] = value[30];
  c[3][4] = value[31];

  c[4][3] = value[32];
  c[5][2] = value[33];
  c[6][1] = value[34];
  c[7][0] = value[35];
  c[7][1] = value[36];
  c[6][2] = value[37];
  c[5][3] = value[38];
  c[4][4] = value[39];
  c[3][5] = value[40];
  c[2][6] = value[41];
  c[1][7] = value[42];
  c[2][7] = value[43];
  c[3][6] = value[44];
  c[4][5] = value[45];
  c[5][4] = value[46];
  c[6][3] = value[47];

  c[7][2] = value[48];
  c[7][3] = value[49];
  c[6][4] = value[50];
  c[5][5] = value[51];
  c[4][6] = value[52];
  c[3][7] = value[53];
  c[4][7] = value[54];
  c[5][6] = value[55];
  c[6][5] = value[56];
  c[7][4] = value[57];
  c[7][5] = value[58];
  c[6][6] = value[59];
  c[5][7] = value[60];
  c[6][7] = value[61];
  c[7][6] = value[62];
  c[7][7] = value[63];

  c
}

fn get_qpc(slice: &Slice, is_chroma_cb: bool) -> i16 {
  let qp_offset = if is_chroma_cb {
    slice.pps.chroma_qp_index_offset
  } else {
    slice
      .pps
      .extra_rbsp_data
      .as_ref()
      .map(|pps| pps.second_chroma_qp_index_offset)
      .unwrap_or(slice.pps.chroma_qp_index_offset)
  };

  let qpi = clamp(slice.mb().qpy + qp_offset, -slice.qp_bd_offset_c, 51);

  if qpi < 30 {
    qpi
  } else {
    const QPCS: [i16; 22] = [
      29, 30, 31, 32, 32, 33, 34, 34, 35, 35, 36, 36, 37, 37, 37, 38, 38, 38, 39, 39, 39, 39,
    ];
    QPCS[qpi as usize - 30]
  }
}

fn chroma_quantization_parameters(slice: &mut Slice, is_chroma_cb: bool) {
  slice.mb_mut().qpc = get_qpc(slice, is_chroma_cb);
  slice.mb_mut().qp1c = slice.mb().qpc + slice.qp_bd_offset_c;

  if slice.slice_type.is_switching() {
    slice.mb_mut().qsy = slice.mb().qpy;
    slice.mb_mut().qsc = slice.mb().qpc;
  }
}
