pub mod pred4x4;
pub mod transform;

use super::slice::Slice;
use crate::math::inverse_raster_scan;

pub struct Frame {
  pub luma_data: Box<[Box<[u8]>]>,
  pub chroma_cb_data: Box<[Box<[u8]>]>,
  pub chroma_cr_data: Box<[Box<[u8]>]>,
  pub level_scale4x4: [[[i16; 4]; 4]; 6],
  pub level_scale8x8: [[[i16; 4]; 4]; 6],
}

impl Frame {
  /// 8.5.14 Picture construction process prior to deblocking filter process
  pub fn picture_construction(
    &mut self,
    slice: &Slice,
    u: &[i16],
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
          self.luma_data[x_p + x_o + j][y_p + y_o + i] = u[i * n_e + j] as u8;
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
              [y_p / slice.sub_height_c as usize + y_o + i] = u[i * mb_width_c + j] as u8;
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
fn inverse_scanner4x4(value: &[i16; 16]) -> [[i16; 4]; 4] {
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
