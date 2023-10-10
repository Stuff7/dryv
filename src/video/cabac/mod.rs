pub mod consts;
pub mod syntax_element;
pub mod table;

use super::slice::{
  consts::*,
  consts::{is_skip_mb_type, MB_TYPE_UNAVAILABLE},
  header::SliceType,
  macroblock::{BlockSize, Macroblock, MacroblockError, MbPosition},
  Slice,
};
use crate::{log, math::clamp};
use consts::*;
use syntax_element::SEValue;
use syntax_element::*;
use table::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CabacError {
  #[error("Error cod_i_offset must not be 510 nor 511")]
  Engine,
  #[error("Inconsistent SE table")]
  SETable,
  #[error("No value from binarization")]
  Binarization,
  #[error("pcm_alignment_zero_bit must be 0")]
  PcmAlignmentZeroBit,
  #[error("cabac_alignment_one_bit must be 1")]
  CabacAlignmentOneBit,
  #[error("cabac_zero_word must be 0")]
  CabacZeroWord,
  #[error("mb_skip_flag used in I/SI slice")]
  MbSkipFlagSlice,
  #[error(transparent)]
  Macroblock(#[from] MacroblockError),
  #[error("Invalid slice type for sub_mb_type")]
  SubMbType,
}

pub type CabacResult<T = ()> = Result<T, CabacError>;

/// Represents the context for Context-Adaptive Binary Arithmetic Coding (CABAC) in H.264 video encoding.
#[derive(Debug)]
pub struct CabacContext {
  /// Probability state indices for CABAC encoding.
  /// CABAC adapts its encoding probability based on the context of previous symbols.
  /// These indices help track the state of probability models.
  pub p_state_idx: [i16; CTX_IDX_COUNT],

  /// Values representing Most Probable Symbols (MPS) for CABAC encoding.
  /// CABAC encodes binary symbols as either the MPS or the Least Probable Symbol (LPS).
  /// These values determine the default symbol to be encoded (MPS).
  pub val_mps: [u8; CTX_IDX_COUNT],

  /// Range for interval coding in CABAC.
  /// CABAC encodes symbols by mapping them to an interval within a specified range.
  /// This `cod_i_range` parameter defines the size of the interval.
  pub cod_i_range: u16,

  /// Offset for interval coding in CABAC.
  /// CABAC intervals are defined by a starting point within the range.
  /// The `cod_i_offset` parameter specifies this offset.
  pub cod_i_offset: u16,

  /// Count of binary symbols processed by CABAC.
  /// This counter keeps track of the number of binary symbols encoded or decoded using CABAC.
  pub bin_count: i32,
}

impl CabacContext {
  pub fn new(slice: &mut Slice) -> CabacResult<Self> {
    if !slice.stream.is_byte_aligned(1) {
      return Err(CabacError::CabacAlignmentOneBit);
    }

    let (p_state_idx, val_mps) = Self::init_context_variables(slice);
    let (cod_i_range, cod_i_offset) = Self::init_decoding_engine(slice)?;

    Ok(Self {
      p_state_idx,
      val_mps,
      cod_i_offset,
      cod_i_range,
      bin_count: 0,
    })
  }

  pub fn macroblock_layer(&mut self, slice: &mut Slice) -> CabacResult {
    let transform_8x8_mode_flag = slice
      .pps
      .extra_rbsp_data
      .as_ref()
      .is_some_and(|pps| pps.transform_8x8_mode_flag);
    let direct_8x8_inference_flag = slice.sps.direct_8x8_inference_flag;
    let bit_depth_luma_minus8 = slice.sps.bit_depth_luma_minus8 as usize;
    let bit_depth_chroma_minus8 = slice.sps.bit_depth_chroma_minus8 as usize;
    self.mb_type(slice)?;
    if slice.mb().mb_type == MB_TYPE_I_PCM {
      if !slice.stream.is_byte_aligned(0) {
        return Err(CabacError::PcmAlignmentZeroBit);
      }
      for i in 0..256 {
        slice.mb_mut().pcm_sample_luma[i] = slice.stream.bits_into(bit_depth_luma_minus8 + 8);
      }
      if slice.chroma_array_type != 0 {
        for i in 0..(64 << slice.chroma_array_type) {
          slice.mb_mut().pcm_sample_chroma[i] = slice.stream.bits_into(bit_depth_chroma_minus8 + 8);
        }
      }
      (self.cod_i_range, self.cod_i_offset) = Self::init_decoding_engine(slice)?; // ?
      slice.mb_mut().mb_qp_delta = 0;
      slice.mb_mut().transform_size_8x8_flag = 0;
      slice.mb_mut().coded_block_pattern = 0x2f;
      slice.mb_mut().intra_chroma_pred_mode = 0;
      slice.infer_intra(0);
      slice.infer_intra(1);
      for i in 0..17 {
        slice.mb_mut().coded_block_flag[0][i] = 1;
        slice.mb_mut().coded_block_flag[1][i] = 1;
        slice.mb_mut().coded_block_flag[2][i] = 1;
      }
      for i in 0..16 {
        slice.mb_mut().total_coeff[0][i] = 16;
        slice.mb_mut().total_coeff[1][i] = 16;
        slice.mb_mut().total_coeff[2][i] = 16;
      }
    } else {
      let mut no_sub_mb_part_size_less_than8x8_flag = 1;
      if is_submb_mb_type(slice.mb().mb_type) {
        self.sub_mb_pred(slice)?;
        for i in 0..4 {
          if slice.mb().sub_mb_type[i] != SUB_MB_TYPE_B_DIRECT_8X8 {
            if SUB_MB_PART_INFO[slice.mb().sub_mb_type[i] as usize][0] != 0 {
              no_sub_mb_part_size_less_than8x8_flag = 0;
            }
          } else if !direct_8x8_inference_flag {
            no_sub_mb_part_size_less_than8x8_flag = 0;
          }
        }
        slice.mb_mut().intra_chroma_pred_mode = 0;
      } else {
        if slice.mb().mb_type == MB_TYPE_I_NXN || slice.mb().mb_type == MB_TYPE_SI {
          if transform_8x8_mode_flag {
            slice.mb_mut().transform_size_8x8_flag = self.transform_size_8x8_flag(slice)?;
          } else {
            slice.mb_mut().transform_size_8x8_flag = 0;
          }
        }
        self.mb_pred(slice)?;
      }
      if slice.mb().mb_type == MB_TYPE_I_NXN
        || slice.mb().mb_type == MB_TYPE_SI
        || slice.mb().mb_type >= MB_TYPE_SI
      {
        let has_chroma = slice.chroma_array_type < 3 && slice.chroma_array_type != 0;
        slice.mb_mut().coded_block_pattern = self.coded_block_pattern(slice, has_chroma)?;
        if slice.mb().mb_type >= MB_TYPE_SI {
          if (slice.mb().coded_block_pattern & 0xf) != 0
            && transform_8x8_mode_flag
            && no_sub_mb_part_size_less_than8x8_flag != 0
            && (slice.mb().mb_type != MB_TYPE_B_DIRECT_16X16 || direct_8x8_inference_flag)
          {
            slice.mb_mut().transform_size_8x8_flag = self.transform_size_8x8_flag(slice)?;
          } else {
            slice.mb_mut().transform_size_8x8_flag = 0;
          }
        }
      } else {
        let mut infer_cbp = (((slice.mb().mb_type - MB_TYPE_I_16X16_0_0_0) >> 2) % 3) << 4;
        if slice.mb().mb_type >= MB_TYPE_I_16X16_0_0_1 {
          infer_cbp |= 0xf;
        }
        slice.mb_mut().coded_block_pattern = infer_cbp;
        slice.mb_mut().transform_size_8x8_flag = 0;
      }
      if slice.mb().coded_block_pattern != 0 || is_intra_16x16_mb_type(slice.mb().mb_type) {
        slice.mb_mut().mb_qp_delta = self.mb_qp_delta(slice)?;
      } else {
        slice.mb_mut().mb_qp_delta = 0;
      }
      self.residual(slice, 0, 15)?;
    }
    Ok(())
  }

  pub fn mb_pred(&mut self, slice: &mut Slice) -> CabacResult {
    if slice.mb().mb_type < MB_TYPE_P_L0_16X16 {
      if !is_intra_16x16_mb_type(slice.mb().mb_type) {
        if slice.mb().transform_size_8x8_flag == 0 {
          for i in 0..16 {
            slice.mb_mut().prev_intra4x4_pred_mode_flag[i] =
              self.prev_intra_pred_mode_flag(slice)?;
            if slice.mb().prev_intra4x4_pred_mode_flag[i] == 0 {
              slice.mb_mut().rem_intra4x4_pred_mode[i] = self.rem_intra_pred_mode(slice)?;
            }
          }
        } else {
          for i in 0..4 {
            slice.mb_mut().prev_intra8x8_pred_mode_flag[i] =
              self.prev_intra_pred_mode_flag(slice)?;
            if slice.mb().prev_intra8x8_pred_mode_flag[i] == 0 {
              slice.mb_mut().rem_intra8x8_pred_mode[i] = self.rem_intra_pred_mode(slice)?;
            }
          }
        }
      }
      if slice.chroma_array_type == 1 || slice.chroma_array_type == 2 {
        slice.mb_mut().intra_chroma_pred_mode = self.intra_chroma_pred_mode(slice)?;
      } else {
        slice.mb_mut().intra_chroma_pred_mode = 0;
      }
      slice.infer_intra(0);
      slice.infer_intra(1);
    } else if slice.mb().mb_type != MB_TYPE_B_DIRECT_16X16 {
      let mut ifrom = [0; 4];
      let mut pmode = [0; 4];
      ifrom[0] = -1isize as usize;
      pmode[0] = MB_PART_INFO[slice.mb().mb_type as usize][1];
      let mb_type = slice.mb().mb_type as usize;
      match MB_PART_INFO[mb_type][0] {
        0 => {
          // 16x16
          ifrom[1] = 0;
          ifrom[2] = 0;
          ifrom[3] = 0;
        }
        1 => {
          // 16x8
          ifrom[1] = 0;
          ifrom[2] = -1isize as usize;
          ifrom[3] = 2;
          pmode[2] = MB_PART_INFO[mb_type][2];
        }
        2 => {
          ifrom[1] = -1isize as usize;
          ifrom[2] = 0;
          ifrom[3] = 1;
          pmode[1] = MB_PART_INFO[mb_type][2];
        }
        _ => unreachable!(),
      }
      let mut max = slice.num_ref_idx_l0_active_minus1.unwrap_or_default();
      if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
        max *= 2;
        max += 1;
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 1) != 0 {
            slice.mb_mut().ref_idx[0][i] = self.ref_idx(slice, i as isize, 0, max)?;
          } else {
            slice.mb_mut().ref_idx[0][i] = 0;
          }
        } else {
          slice.mb_mut().ref_idx[0][i] = slice.mb().ref_idx[0][ifrom[i]];
        }
      }
      max = slice.num_ref_idx_l1_active_minus1.unwrap_or_default();
      if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
        max *= 2;
        max += 1;
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 2) != 0 {
            slice.mb_mut().ref_idx[1][i] = self.ref_idx(slice, i as isize, 1, max)?;
          } else {
            slice.mb_mut().ref_idx[1][i] = 0;
          }
        } else {
          slice.mb_mut().ref_idx[1][i] = slice.mb().ref_idx[1][ifrom[i]];
        }
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 1) != 0 {
            slice.mb_mut().mvd[0][i * 4][0] = self.mvd(slice, i as isize * 4, 0, 0)?;
            slice.mb_mut().mvd[0][i * 4][1] = self.mvd(slice, i as isize * 4, 1, 0)?;
          } else {
            slice.mb_mut().mvd[0][i * 4][0] = 0;
            slice.mb_mut().mvd[0][i * 4][1] = 0;
          }
        } else {
          slice.mb_mut().mvd[0][i * 4][0] = slice.mb().mvd[0][ifrom[i] * 4][0];
          slice.mb_mut().mvd[0][i * 4][1] = slice.mb().mvd[0][ifrom[i] * 4][1];
        }
        for j in 1..4 {
          slice.mb_mut().mvd[0][i * 4 + j][0] = slice.mb().mvd[0][i * 4][0];
          slice.mb_mut().mvd[0][i * 4 + j][1] = slice.mb().mvd[0][i * 4][1];
        }
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 2) != 0 {
            slice.mb_mut().mvd[1][i * 4][0] = self.mvd(slice, i as isize * 4, 0, 1)?;
            slice.mb_mut().mvd[1][i * 4][1] = self.mvd(slice, i as isize * 4, 1, 1)?;
          } else {
            slice.mb_mut().mvd[1][i * 4][0] = 0;
            slice.mb_mut().mvd[1][i * 4][1] = 0;
          }
        } else {
          slice.mb_mut().mvd[1][i * 4][0] = slice.mb().mvd[1][ifrom[i] * 4][0];
          slice.mb_mut().mvd[1][i * 4][1] = slice.mb().mvd[1][ifrom[i] * 4][1];
        }
        for j in 1..4 {
          slice.mb_mut().mvd[1][i * 4 + j][0] = slice.mb().mvd[1][i * 4][0];
          slice.mb_mut().mvd[1][i * 4 + j][1] = slice.mb().mvd[1][i * 4][1];
        }
      }
      slice.mb_mut().intra_chroma_pred_mode = 0;
    } else {
      slice.mb_mut().intra_chroma_pred_mode = 0;
      slice.infer_intra(0);
      slice.infer_intra(1);
    }
    Ok(())
  }

  pub fn sub_mb_pred(&mut self, slice: &mut Slice) -> CabacResult {
    let mut pmode = [0; 4];
    let mut ifrom = [0; 16];
    for i in 0..4 {
      slice.mb_mut().sub_mb_type[i] = self.sub_mb_type(slice)?;
      pmode[i] = SUB_MB_PART_INFO[slice.mb().sub_mb_type[i] as usize][1];
      let sm = SUB_MB_PART_INFO[slice.mb().sub_mb_type[i] as usize][0];
      ifrom[i * 4] = -1isize as usize;
      match sm {
        0 => {
          ifrom[i * 4 + 1] = i * 4;
          ifrom[i * 4 + 2] = i * 4;
          ifrom[i * 4 + 3] = i * 4;
        }
        1 => {
          ifrom[i * 4 + 1] = i * 4;
          ifrom[i * 4 + 2] = -1isize as usize;
          ifrom[i * 4 + 3] = i * 4 + 2;
        }
        2 => {
          ifrom[i * 4 + 1] = -1isize as usize;
          ifrom[i * 4 + 2] = i * 4;
          ifrom[i * 4 + 3] = i * 4 + 1;
        }
        3 => {
          ifrom[i * 4 + 1] = -1isize as usize;
          ifrom[i * 4 + 2] = -1isize as usize;
          ifrom[i * 4 + 3] = -1isize as usize;
        }
        _ => unreachable!(),
      }
    }
    let mut max = slice.num_ref_idx_l0_active_minus1.unwrap_or_default();
    if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
      max *= 2;
      max += 1;
    }
    for (i, pmode) in pmode.iter().enumerate() {
      if (pmode & 1) != 0 && slice.mb().mb_type != MB_TYPE_P_8X8REF0 {
        slice.mb_mut().ref_idx[0][i] = self.ref_idx(slice, i as isize, 0, max)?;
      } else {
        slice.mb_mut().ref_idx[0][i] = 0;
      }
    }
    max = slice.num_ref_idx_l1_active_minus1.unwrap_or_default();
    if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
      max *= 2;
      max += 1;
    }
    for (i, pmode) in pmode.iter().enumerate() {
      if (pmode & 2) != 0 {
        slice.mb_mut().ref_idx[1][i] = self.ref_idx(slice, i as isize, 1, max)?;
      } else {
        slice.mb_mut().ref_idx[1][i] = 0;
      }
    }
    for i in 0..16 {
      if ifrom[i] == -1isize as usize {
        if (pmode[i / 4] & 1) != 0 {
          slice.mb_mut().mvd[0][i][0] = self.mvd(slice, i as isize, 0, 0)?;
          slice.mb_mut().mvd[0][i][1] = self.mvd(slice, i as isize, 1, 0)?;
        } else {
          slice.mb_mut().mvd[0][i][0] = 0;
          slice.mb_mut().mvd[0][i][1] = 0;
        }
      } else {
        slice.mb_mut().mvd[0][i][0] = slice.mb().mvd[0][ifrom[i]][0];
        slice.mb_mut().mvd[0][i][1] = slice.mb().mvd[0][ifrom[i]][1];
      }
    }
    for i in 0..16 {
      if ifrom[i] == -1isize as usize {
        if (pmode[i / 4] & 2) != 0 {
          slice.mb_mut().mvd[1][i][0] = self.mvd(slice, i as isize, 0, 1)?;
          slice.mb_mut().mvd[1][i][1] = self.mvd(slice, i as isize, 1, 1)?;
        } else {
          slice.mb_mut().mvd[1][i][0] = 0;
          slice.mb_mut().mvd[1][i][1] = 0;
        }
      } else {
        slice.mb_mut().mvd[1][i][0] = slice.mb().mvd[1][ifrom[i]][0];
        slice.mb_mut().mvd[1][i][1] = slice.mb().mvd[1][ifrom[i]][1];
      }
    }
    slice.mb_mut().intra_chroma_pred_mode = 0;
    Ok(())
  }

  pub fn residual(&mut self, slice: &mut Slice, start: usize, end: usize) -> CabacResult {
    self.residual_luma(slice, start, end, 0)?;
    if slice.chroma_array_type == 1 || slice.chroma_array_type == 2 {
      for i in 0..2 {
        self.residual_cabac(
          slice,
          ResidualBlock::ChromaDc(i),
          CTXBLOCKCAT_CHROMA_DC,
          i as isize,
          0,
          4 * slice.chroma_array_type as usize - 1,
          4 * slice.chroma_array_type as usize,
          ((slice.mb().coded_block_pattern & 0x30) != 0 && start == 0) as usize,
        )?;
      }
      for i in 0..2 {
        for j in 0..4 * slice.chroma_array_type as usize {
          self.residual_cabac(
            slice,
            ResidualBlock::ChromaAc(i, j),
            CTXBLOCKCAT_CHROMA_AC,
            (i * 8 + j) as isize,
            if start != 0 { start - 1 } else { 0 },
            end - 1,
            15,
            slice.mb().coded_block_pattern as usize & 0x20,
          )?;
        }
      }
    } else if slice.chroma_array_type == 3 {
      self.residual_luma(slice, start, end, 1)?;
      self.residual_luma(slice, start, end, 2)?;
    }
    Ok(())
  }

  pub fn residual_luma(
    &mut self,
    slice: &mut Slice,
    start: usize,
    end: usize,
    which: usize,
  ) -> CabacResult {
    if start == 0 && is_intra_16x16_mb_type(slice.mb().mb_type) {
      self.residual_cabac(
        slice,
        ResidualBlock::LumaDc(which),
        LUMA_CAT_TAB[which][0],
        0,
        0,
        15,
        16,
        1,
      )?;
    } else {
      slice.mb_mut().coded_block_flag[which][16] = 0;
    }
    let n = if is_intra_16x16_mb_type(slice.mb().mb_type) {
      15
    } else {
      16
    };
    let ss = if is_intra_16x16_mb_type(slice.mb().mb_type) {
      if start != 0 {
        start - 1
      } else {
        0
      }
    } else {
      start
    };
    let se = if is_intra_16x16_mb_type(slice.mb().mb_type) {
      end - 1
    } else {
      end
    };
    if slice.mb().transform_size_8x8_flag == 0 {
      for i in 0..16 {
        let mut tmp = [0; 16];
        let cat;
        if slice.mb().transform_size_8x8_flag != 0 {
          for (j, tmp) in tmp.iter_mut().enumerate() {
            *tmp = slice.mb().block_luma_8x8[which][i >> 2][4 * j + (i & 3)];
          }
          cat = LUMA_CAT_TAB[which][3];
        } else if is_intra_16x16_mb_type(slice.mb().mb_type) {
          tmp[..15].copy_from_slice(&slice.mb().block_luma_ac[which][i][..15]);
          cat = LUMA_CAT_TAB[which][1];
        } else {
          tmp[..16].copy_from_slice(&slice.mb().block_luma_4x4[which][i][..16]);
          cat = LUMA_CAT_TAB[which][2];
        }
        self.residual_cabac(
          slice,
          ResidualBlock::Custom(&mut tmp),
          cat,
          i as isize,
          ss,
          se,
          n,
          (slice.mb().coded_block_pattern >> (i >> 2) & 1) as usize,
        )?;
        if slice.mb().transform_size_8x8_flag != 0 {
          for (j, tmp) in tmp.iter().enumerate() {
            slice.mb_mut().block_luma_8x8[which][i >> 2][4 * j + (i & 3)] = *tmp;
          }
        } else if is_intra_16x16_mb_type(slice.mb().mb_type) {
          slice.mb_mut().block_luma_ac[which][i][..15].copy_from_slice(&tmp[..15]);
        } else {
          slice.mb_mut().block_luma_4x4[which][i][..16].copy_from_slice(&tmp[..16]);
        }
      }
    } else {
      for i in 0..4 {
        self.residual_cabac(
          slice,
          ResidualBlock::Luma8x8(which, i),
          LUMA_CAT_TAB[which][3],
          i as isize,
          4 * start,
          4 * end + 3,
          64,
          (slice.mb().coded_block_pattern >> i & 1) as usize,
        )?;
      }
    }
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub fn residual_cabac(
    &mut self,
    slice: &mut Slice,
    mut blocks: ResidualBlock,
    cat: u8,
    idx: isize,
    start: usize,
    end: usize,
    maxnumcoeff: usize,
    coded: usize,
  ) -> CabacResult {
    let coded_block_flag;
    if coded != 0 {
      if maxnumcoeff != 64 || slice.chroma_array_type == 3 {
        coded_block_flag = self.coded_block_flag(slice, cat, idx)?;
      } else {
        coded_block_flag = 1;
      }
    } else {
      coded_block_flag = 0;
    }
    let idx = idx as usize;
    match cat {
      CTXBLOCKCAT_LUMA_DC => {
        slice.mb_mut().coded_block_flag[0][16] = coded_block_flag;
      }
      CTXBLOCKCAT_LUMA_AC | CTXBLOCKCAT_LUMA_4X4 => {
        slice.mb_mut().coded_block_flag[0][idx] = coded_block_flag;
      }
      CTXBLOCKCAT_LUMA_8X8 => {
        slice.mb_mut().coded_block_flag[0][idx * 4] = coded_block_flag;
        slice.mb_mut().coded_block_flag[0][idx * 4 + 1] = coded_block_flag;
        slice.mb_mut().coded_block_flag[0][idx * 4 + 2] = coded_block_flag;
        slice.mb_mut().coded_block_flag[0][idx * 4 + 3] = coded_block_flag;
      }
      CTXBLOCKCAT_CB_DC => {
        slice.mb_mut().coded_block_flag[1][16] = coded_block_flag;
      }
      CTXBLOCKCAT_CB_AC | CTXBLOCKCAT_CB_4X4 => {
        slice.mb_mut().coded_block_flag[1][idx] = coded_block_flag;
      }
      CTXBLOCKCAT_CB_8X8 => {
        slice.mb_mut().coded_block_flag[1][idx * 4] = coded_block_flag;
        slice.mb_mut().coded_block_flag[1][idx * 4 + 1] = coded_block_flag;
        slice.mb_mut().coded_block_flag[1][idx * 4 + 2] = coded_block_flag;
        slice.mb_mut().coded_block_flag[1][idx * 4 + 3] = coded_block_flag;
      }
      CTXBLOCKCAT_CR_DC => {
        slice.mb_mut().coded_block_flag[2][16] = coded_block_flag;
      }
      CTXBLOCKCAT_CR_AC | CTXBLOCKCAT_CR_4X4 => {
        slice.mb_mut().coded_block_flag[2][idx] = coded_block_flag;
      }
      CTXBLOCKCAT_CR_8X8 => {
        slice.mb_mut().coded_block_flag[2][idx * 4] = coded_block_flag;
        slice.mb_mut().coded_block_flag[2][idx * 4 + 1] = coded_block_flag;
        slice.mb_mut().coded_block_flag[2][idx * 4 + 2] = coded_block_flag;
        slice.mb_mut().coded_block_flag[2][idx * 4 + 3] = coded_block_flag;
      }
      CTXBLOCKCAT_CHROMA_DC => {
        slice.mb_mut().coded_block_flag[idx + 1][16] = coded_block_flag;
      }
      CTXBLOCKCAT_CHROMA_AC => {
        slice.mb_mut().coded_block_flag[(idx >> 3) + 1][idx & 7] = coded_block_flag;
      }
      cat => panic!("Invalid ctx_block_cat passed to residual_cabac {cat}"),
    }
    if coded_block_flag != 0 {
      let field = slice.mb().mb_field_decoding_flag || slice.field_pic_flag;
      let mut significant_coeff_flag = [0; 64];
      let mut last_significant_coeff_flag = [0; 64];
      let mut numcoeff = end + 1;
      let mut i = start;
      while i < numcoeff - 1 {
        significant_coeff_flag[i] =
          self.significant_coeff_flag(slice, field as usize, cat, i, 0)?;
        if significant_coeff_flag[i] != 0 {
          last_significant_coeff_flag[i] =
            self.significant_coeff_flag(slice, field as usize, cat, i, 1)?;
          if last_significant_coeff_flag[i] != 0 {
            numcoeff = i + 1;
          }
        }
        i += 1;
      }
      significant_coeff_flag[numcoeff - 1] = 1;
      for block in blocks.content(slice).iter_mut().take(maxnumcoeff) {
        *block = 0;
      }
      let mut num1 = 0;
      let mut numgt1 = 0;
      let mut i = numcoeff as isize - 1;
      while i >= start as isize {
        let idx = i as usize;
        if significant_coeff_flag[idx] != 0 {
          let cam1 = self.coeff_abs_level_minus1(slice, cat, num1, numgt1)?;
          let s = self.bypass(slice)?;
          if cam1 != 0 {
            numgt1 += 1;
          } else {
            num1 += 1;
          }
          blocks.content(slice)[idx] = if s != 0 { -(cam1 + 1) } else { cam1 + 1 };
        }
        i -= 1;
      }
    } else {
      for block in blocks.content(slice).iter_mut() {
        *block = 0;
      }
    }
    Ok(())
  }

  pub fn coeff_abs_level_minus1(
    &mut self,
    slice: &mut Slice,
    cat: u8,
    num1: i16,
    numgt1: i16,
  ) -> CabacResult<i16> {
    let mut ctx_idx = [0; 2];
    ctx_idx[0] = COEFF_ABS_LEVEL_MINUS1_BASE_CTX[cat as usize]
      + if numgt1 != 0 {
        0
      } else if num1 >= 4 {
        4
      } else {
        num1 + 1
      };
    let clamp = if cat == CTXBLOCKCAT_CHROMA_DC { 3 } else { 4 };
    ctx_idx[1] = COEFF_ABS_LEVEL_MINUS1_BASE_CTX[cat as usize]
      + 5
      + if numgt1 > clamp { clamp } else { numgt1 };
    self.ueg(slice, &ctx_idx, 2, 0, 0, 14)
  }

  pub fn significant_coeff_flag(
    &mut self,
    slice: &mut Slice,
    field: usize,
    cat: u8,
    idx: usize,
    last: usize,
  ) -> CabacResult<u8> {
    let mut ctx_inc: i16;
    match cat {
      CTXBLOCKCAT_LUMA_DC
      | CTXBLOCKCAT_LUMA_AC
      | CTXBLOCKCAT_LUMA_4X4
      | CTXBLOCKCAT_CB_DC
      | CTXBLOCKCAT_CB_AC
      | CTXBLOCKCAT_CB_4X4
      | CTXBLOCKCAT_CR_DC
      | CTXBLOCKCAT_CR_AC
      | CTXBLOCKCAT_CR_4X4
      | CTXBLOCKCAT_CHROMA_AC => {
        ctx_inc = idx as i16;
      }
      CTXBLOCKCAT_CHROMA_DC => {
        ctx_inc = idx as i16 / slice.chroma_array_type as i16;
        if ctx_inc > 2 {
          ctx_inc = 2;
        }
      }
      CTXBLOCKCAT_LUMA_8X8 | CTXBLOCKCAT_CB_8X8 | CTXBLOCKCAT_CR_8X8 => {
        if last != 0 {
          ctx_inc = SIGNIFICANT_COEFF_FLAG_TAB8X8[idx][2] as i16;
        } else {
          ctx_inc = SIGNIFICANT_COEFF_FLAG_TAB8X8[idx][field] as i16;
        }
      }
      cat => panic!("Invalid ctx_block_cat passed to significant_coeff_flag {cat}"),
    }
    let ctx_idx = SIGNIFICANT_COEFF_FLAG_BASE_CTX[last][field][cat as usize] + ctx_inc;
    self.decision(slice, ctx_idx)
  }

  pub fn coded_block_flag(
    &mut self,
    slice: &mut Slice,
    cat: u8,
    mut idx: isize,
  ) -> CabacResult<u8> {
    let mb_t = slice.mb_nb_p(MbPosition::This, 0);
    let inter = is_inter_mb_type(mb_t.mb_type) as u8;
    let which;
    match cat {
      CTXBLOCKCAT_LUMA_DC | CTXBLOCKCAT_LUMA_AC | CTXBLOCKCAT_LUMA_4X4 | CTXBLOCKCAT_LUMA_8X8 => {
        which = 0;
      }
      CTXBLOCKCAT_CB_DC | CTXBLOCKCAT_CB_AC | CTXBLOCKCAT_CB_4X4 | CTXBLOCKCAT_CB_8X8 => {
        which = 1;
      }
      CTXBLOCKCAT_CR_DC | CTXBLOCKCAT_CR_AC | CTXBLOCKCAT_CR_4X4 | CTXBLOCKCAT_CR_8X8 => {
        which = 2;
      }
      CTXBLOCKCAT_CHROMA_DC => {
        which = idx + 1;
      }
      CTXBLOCKCAT_CHROMA_AC => {
        which = (idx >> 3) + 1;
        idx &= 7;
      }
      cat => panic!("Invalid ctx_block_cat passed to coded_block_flag {cat}"),
    }
    let mut mb_a;
    let mut mb_b;
    let mut idx_a = 0;
    let mut idx_b = 0;
    match cat {
      CTXBLOCKCAT_LUMA_DC | CTXBLOCKCAT_CB_DC | CTXBLOCKCAT_CR_DC | CTXBLOCKCAT_CHROMA_DC => {
        mb_a = slice.mb_nb(MbPosition::A, inter)?;
        mb_b = slice.mb_nb(MbPosition::B, inter)?;
        idx_a = 16;
        idx_b = 16;
      }
      CTXBLOCKCAT_LUMA_AC | CTXBLOCKCAT_LUMA_4X4 | CTXBLOCKCAT_CB_AC | CTXBLOCKCAT_CB_4X4
      | CTXBLOCKCAT_CR_AC | CTXBLOCKCAT_CR_4X4 => {
        mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B4x4, inter, idx, &mut idx_a)?;
        mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B4x4, inter, idx, &mut idx_b)?;
      }
      CTXBLOCKCAT_LUMA_8X8 | CTXBLOCKCAT_CB_8X8 | CTXBLOCKCAT_CR_8X8 => {
        mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B8x8, inter, idx, &mut idx_a)?;
        mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B8x8, inter, idx, &mut idx_b)?;
        idx_a *= 4;
        idx_b *= 4;
        if mb_a.transform_size_8x8_flag == 0
          && mb_a.mb_type != MB_TYPE_I_PCM
          && mb_a.mb_type != MB_TYPE_UNAVAILABLE
        {
          Macroblock::unavailable(1);
        }
        if mb_b.transform_size_8x8_flag == 0
          && mb_b.mb_type != MB_TYPE_I_PCM
          && mb_b.mb_type != MB_TYPE_UNAVAILABLE
        {
          Macroblock::unavailable(1);
        }
      }
      CTXBLOCKCAT_CHROMA_AC => {
        mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::Chroma, inter, idx, &mut idx_a)?;
        mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::Chroma, inter, idx, &mut idx_b)?;
      }
      cat => panic!("Invalid ctx_block_cat passed to coded_block_flag {cat}"),
    }
    mb_a = slice.inter_filter(mb_a, inter);
    mb_b = slice.inter_filter(mb_b, inter);
    let cond_term_flag_a = mb_a.coded_block_flag[which as usize][idx_a as usize] as i16;
    let cond_term_flag_b = mb_b.coded_block_flag[which as usize][idx_b as usize] as i16;
    let ctx_idx = CODED_BLOCK_FLAG_BASE_CTX[cat as usize] + cond_term_flag_a + cond_term_flag_b * 2;
    self.decision(slice, ctx_idx)
  }

  pub fn mb_qp_delta(&mut self, slice: &mut Slice) -> CabacResult<i16> {
    let mut ctx_idx = [0; 3];
    if slice.prev_mb_addr != -1 && slice.macroblocks[slice.prev_mb_addr as usize].mb_qp_delta != 0 {
      ctx_idx[0] = CTXIDX_MB_QP_DELTA + 1;
    } else {
      ctx_idx[0] = CTXIDX_MB_QP_DELTA;
    }
    ctx_idx[1] = CTXIDX_MB_QP_DELTA + 2;
    ctx_idx[2] = CTXIDX_MB_QP_DELTA + 3;
    let tmp = self.tu(slice, &ctx_idx, 3, -1i8 as u8)? as i16;
    Ok(if (tmp & 1) != 0 {
      (tmp + 1) >> 1
    } else {
      -(tmp >> 1)
    })
  }

  #[allow(clippy::cast_ref_to_mut)]
  pub fn coded_block_pattern(&mut self, slice: &mut Slice, has_chroma: bool) -> CabacResult<u8> {
    let mut bit = [0u8; 6];
    let mut ctx_idx;
    let mb_t = slice.mb_nb(MbPosition::This, 0)?;
    let mut idx_a = 0;
    let mut idx_b = 0;
    let mut mb_a;
    let mut mb_b;
    for i in 0..4 {
      mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B8x8, 0, i, &mut idx_a)?;
      mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B8x8, 0, i, &mut idx_b)?;
      let cond_term_flag_a = ((if std::ptr::eq(mb_a as *const _, mb_t as *const _) {
        bit[idx_a as usize]
      } else {
        mb_a.coded_block_pattern >> idx_a & 1
      }) == 0) as i16;
      let cond_term_flag_b = ((if std::ptr::eq(mb_b as *const _, mb_t as *const _) {
        bit[idx_b as usize]
      } else {
        mb_b.coded_block_pattern >> idx_b & 1
      }) == 0) as i16;
      ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_LUMA + cond_term_flag_a + cond_term_flag_b * 2;
      bit[i as usize] = self.decision(unsafe { &mut *(slice as *const _ as *mut _) }, ctx_idx)?;
    }
    if has_chroma {
      mb_a = slice.mb_nb(MbPosition::A, 0)?;
      mb_b = slice.mb_nb(MbPosition::B, 0)?;
      let cond_term_flag_a = ((mb_a.coded_block_pattern >> 4) > 0) as i16;
      let cond_term_flag_b = ((mb_b.coded_block_pattern >> 4) > 0) as i16;
      ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_CHROMA + cond_term_flag_a + cond_term_flag_b * 2;
      bit[4] = self.decision(unsafe { &mut *(slice as *const _ as *mut _) }, ctx_idx)?;
      if bit[4] != 0 {
        let cond_term_flag_a = ((mb_a.coded_block_pattern >> 4) > 1) as i16;
        let cond_term_flag_b = ((mb_b.coded_block_pattern >> 4) > 1) as i16;
        ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_CHROMA + cond_term_flag_a + cond_term_flag_b * 2 + 4;
        bit[5] = self.decision(slice, ctx_idx)?;
      }
    }
    Ok(bit[0] | bit[1] << 1 | bit[2] << 2 | bit[3] << 3 | bit[4] << (4 + bit[5]))
  }

  pub fn intra_chroma_pred_mode(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let cond_term_flag_a = (slice.mb_nb(MbPosition::A, 0)?.intra_chroma_pred_mode != 0) as i16;
    let cond_term_flag_b = (slice.mb_nb(MbPosition::B, 0)?.intra_chroma_pred_mode != 0) as i16;
    let ctx_idx = [
      CTXIDX_INTRA_CHROMA_PRED_MODE + cond_term_flag_a + cond_term_flag_b,
      CTXIDX_INTRA_CHROMA_PRED_MODE + 3,
    ];
    self.tu(slice, &ctx_idx, 2, 3)
  }

  pub fn rem_intra_pred_mode(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let bit = [
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
    ];
    Ok(bit[0] | bit[1] << 1 | bit[2] << 2)
  }

  pub fn prev_intra_pred_mode_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    self.decision(slice, CTXIDX_PREV_INTRA_PRED_MODE_FLAG)
  }

  pub fn transform_size_8x8_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let ctx_idx_offset = CTXIDX_TRANSFORM_SIZE_8X8_FLAG;
    let cond_term_flag_a = slice.mb_nb(MbPosition::A, 0)?.transform_size_8x8_flag;
    let cond_term_flag_b = slice.mb_nb(MbPosition::B, 0)?.transform_size_8x8_flag;
    let ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc as i16)
  }

  pub fn mvd(
    &mut self,
    slice: &mut Slice,
    idx: isize,
    comp: usize,
    which: usize,
  ) -> CabacResult<i16> {
    let base_idx = if comp != 0 {
      CTXIDX_MVD_Y
    } else {
      CTXIDX_MVD_X
    };
    let mut idx_a = 0;
    let mut idx_b = 0;
    let mb_t = slice.mb_nb(MbPosition::This, 0)?;
    let mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B4x4, 0, idx, &mut idx_a)?;
    let mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B4x4, 0, idx, &mut idx_b)?;
    let mut abs_mvd_comp_a = mb_a.mvd[which][idx_a as usize][comp].unsigned_abs();
    let mut abs_mvd_comp_b = mb_b.mvd[which][idx_b as usize][comp].unsigned_abs();
    if comp != 0 {
      if mb_t.mb_field_decoding_flag && !mb_a.mb_field_decoding_flag {
        abs_mvd_comp_a /= 2;
      }
      if !mb_t.mb_field_decoding_flag && mb_a.mb_field_decoding_flag {
        abs_mvd_comp_a *= 2;
      }
      if mb_t.mb_field_decoding_flag && !mb_b.mb_field_decoding_flag {
        abs_mvd_comp_b /= 2;
      }
      if !mb_t.mb_field_decoding_flag && mb_b.mb_field_decoding_flag {
        abs_mvd_comp_b *= 2;
      }
    }
    let sum = abs_mvd_comp_a + abs_mvd_comp_b;
    let inc;
    if sum < 3 {
      inc = 0;
    } else if sum <= 32 {
      inc = 1;
    } else {
      inc = 2;
    }
    let ctx_idx = [
      base_idx + inc,
      base_idx + 3,
      base_idx + 4,
      base_idx + 5,
      base_idx + 6,
    ];
    self.ueg(slice, &ctx_idx, 5, 3, 1, 9)
  }

  pub fn ref_idx(
    &mut self,
    slice: &mut Slice,
    idx: isize,
    which: usize,
    max: u16,
  ) -> CabacResult<u8> {
    if max == 0 {
      return Ok(0);
    }
    let mut idx_a = 0;
    let mut idx_b = 0;
    let mb_t = slice.mb_nb(MbPosition::This, 0)?;
    let mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B8x8, 0, idx, &mut idx_a)?;
    let mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B8x8, 0, idx, &mut idx_b)?;
    let thr_a = !mb_t.mb_field_decoding_flag && mb_a.mb_field_decoding_flag;
    let thr_b = !mb_t.mb_field_decoding_flag && mb_b.mb_field_decoding_flag;
    let cond_term_flag_a = (mb_a.ref_idx[which][idx_a as usize] > thr_a as u8) as i16;
    let cond_term_flag_b = (mb_b.ref_idx[which][idx_b as usize] > thr_b as u8) as i16;
    let ctx_idx = [
      CTXIDX_REF_IDX + cond_term_flag_a + 2 * cond_term_flag_b,
      CTXIDX_REF_IDX + 4,
      CTXIDX_REF_IDX + 5,
    ];
    self.tu(slice, &ctx_idx, 3, -1i8 as u8)
  }

  pub fn sub_mb_type(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let mut val = 0;
    let slice_type = slice.slice_type;
    if slice_type.is_predictive() {
      let bidx = [
        CTXIDX_SUB_MB_TYPE_P,
        CTXIDX_SUB_MB_TYPE_P + 1,
        CTXIDX_SUB_MB_TYPE_P + 2,
      ];
      self.se(slice, SUB_MB_TYPE_P_TABLE, &bidx, &mut val)?;
      Ok(val)
    } else if slice_type.is_bidirectional() {
      let bidx = [
        CTXIDX_SUB_MB_TYPE_B,
        CTXIDX_SUB_MB_TYPE_B + 1,
        CTXIDX_SUB_MB_TYPE_B + 2,
        CTXIDX_SUB_MB_TYPE_B + 3,
      ];
      self.se(slice, SUB_MB_TYPE_B_TABLE, &bidx, &mut val)?;
      Ok(val)
    } else {
      Err(CabacError::SubMbType)
    }
  }

  pub fn mb_type(&mut self, slice: &mut Slice) -> CabacResult {
    let mut bidx = [0i16; 11];
    let mbt_a = slice.mb_nb(MbPosition::A, 0)?.mb_type;
    let mbt_b = slice.mb_nb(MbPosition::B, 0)?.mb_type;
    let mut cond_term_flag_a = mbt_a != MB_TYPE_UNAVAILABLE;
    let mut cond_term_flag_b = mbt_b != MB_TYPE_UNAVAILABLE;
    let slice_type = slice.slice_type;
    let mut val = slice.mb().mb_type;
    match slice_type {
      SliceType::SI => {
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_SI;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_SI;
        let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
        bidx[7] = CTXIDX_MB_TYPE_SI_PRE + ctx_idx_inc;
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_I_NXN;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_I_NXN;
        let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
        bidx[0] = CTXIDX_MB_TYPE_I + ctx_idx_inc;
        bidx[1] = CTXIDX_TERMINATE;
        bidx[2] = CTXIDX_MB_TYPE_I + 3;
        bidx[3] = CTXIDX_MB_TYPE_I + 4;
        bidx[4] = CTXIDX_MB_TYPE_I + 5;
        bidx[5] = CTXIDX_MB_TYPE_I + 6;
        bidx[6] = CTXIDX_MB_TYPE_I + 7;
        self.se(slice, MB_TYPE_SI_TABLE, &bidx, &mut val)?;
      }
      SliceType::I => {
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_I_NXN;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_I_NXN;
        let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
        bidx[0] = CTXIDX_MB_TYPE_I + ctx_idx_inc;
        bidx[1] = CTXIDX_TERMINATE;
        bidx[2] = CTXIDX_MB_TYPE_I + 3;
        bidx[3] = CTXIDX_MB_TYPE_I + 4;
        bidx[4] = CTXIDX_MB_TYPE_I + 5;
        bidx[5] = CTXIDX_MB_TYPE_I + 6;
        bidx[6] = CTXIDX_MB_TYPE_I + 7;
        self.se(slice, MB_TYPE_I_TABLE, &bidx, &mut val)?;
      }
      SliceType::P | SliceType::SP => {
        bidx[7] = CTXIDX_MB_TYPE_P_PRE;
        bidx[8] = CTXIDX_MB_TYPE_P_PRE + 1;
        bidx[9] = CTXIDX_MB_TYPE_P_PRE + 2;
        bidx[10] = CTXIDX_MB_TYPE_P_PRE + 3;
        bidx[0] = CTXIDX_MB_TYPE_P_SUF;
        bidx[1] = CTXIDX_TERMINATE;
        bidx[2] = CTXIDX_MB_TYPE_P_SUF + 1;
        bidx[3] = CTXIDX_MB_TYPE_P_SUF + 2;
        bidx[4] = CTXIDX_MB_TYPE_P_SUF + 2;
        bidx[5] = CTXIDX_MB_TYPE_P_SUF + 3;
        bidx[6] = CTXIDX_MB_TYPE_P_SUF + 3;
        self.se(slice, MB_TYPE_P_TABLE, &bidx, &mut val)?;
      }
      SliceType::B => {
        cond_term_flag_a =
          cond_term_flag_a && mbt_a != MB_TYPE_B_SKIP && mbt_a != MB_TYPE_B_DIRECT_16X16;
        cond_term_flag_b =
          cond_term_flag_b && mbt_b != MB_TYPE_B_SKIP && mbt_b != MB_TYPE_B_DIRECT_16X16;
        let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
        bidx[7] = CTXIDX_MB_TYPE_B_PRE + ctx_idx_inc;
        bidx[8] = CTXIDX_MB_TYPE_B_PRE + 3;
        bidx[9] = CTXIDX_MB_TYPE_B_PRE + 4;
        bidx[10] = CTXIDX_MB_TYPE_B_PRE + 5;
        bidx[0] = CTXIDX_MB_TYPE_B_SUF;
        bidx[1] = CTXIDX_TERMINATE;
        bidx[2] = CTXIDX_MB_TYPE_B_SUF + 1;
        bidx[3] = CTXIDX_MB_TYPE_B_SUF + 2;
        bidx[4] = CTXIDX_MB_TYPE_B_SUF + 2;
        bidx[5] = CTXIDX_MB_TYPE_B_SUF + 3;
        bidx[6] = CTXIDX_MB_TYPE_B_SUF + 3;
        self.se(slice, MB_TYPE_B_TABLE, &bidx, &mut val)?;
      }
    }
    slice.macroblocks[slice.curr_mb_addr as usize].mb_type = val;
    Ok(())
  }

  pub fn mb_skip_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let ctx_idx_offset;
    if slice.slice_type.is_predictive() {
      ctx_idx_offset = CTXIDX_MB_SKIP_FLAG_P;
    } else if slice.slice_type.is_bidirectional() {
      ctx_idx_offset = CTXIDX_MB_SKIP_FLAG_B;
    } else {
      return Err(CabacError::MbSkipFlagSlice);
    }
    let mb_a = slice.mb_nb(MbPosition::A, 0)?;
    let mb_b = slice.mb_nb(MbPosition::B, 0)?;
    let cond_term_flag_a = mb_a.mb_type != MB_TYPE_UNAVAILABLE && !is_skip_mb_type(mb_a.mb_type);
    let cond_term_flag_b = mb_b.mb_type != MB_TYPE_UNAVAILABLE && !is_skip_mb_type(mb_b.mb_type);
    let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc)
  }

  pub fn mb_field_decoding_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let ctx_idx_offset = CTXIDX_MB_FIELD_DECODING_FLAG;
    let cond_term_flag_a = slice.mb_nb_p(MbPosition::A, 0).mb_field_decoding_flag;
    let cond_term_flag_b = slice.mb_nb_p(MbPosition::B, 0).mb_field_decoding_flag;
    let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc)
  }

  pub fn se(
    &mut self,
    slice: &mut Slice,
    table: &[SEValue],
    ctx_indices: &[i16],
    val: &mut u8,
  ) -> CabacResult {
    let mut byte = [0u8; 8];
    let mut bin_idx = [-1i16; 8];
    for se_value in table {
      let mut j = 0;
      for bit in se_value.bits {
        if bin_idx[j] == -1 {
          bin_idx[j] = bit.bin_idx as i16;
          byte[j] = self.decision(slice, ctx_indices[bin_idx[j] as usize])?;
        }
        if bin_idx[j] != bit.bin_idx as i16 {
          return Err(CabacError::SETable);
        }
        if byte[j] != bit.value {
          break;
        }
        j += 1;
      }

      if j == se_value.bits.len() {
        if let Some(sub_table) = se_value.sub_table {
          return self.se(slice, sub_table, ctx_indices, val);
        } else {
          *val = se_value.value;
          return Ok(());
        }
      }
    }
    Err(CabacError::Binarization)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn ueg(
    &mut self,
    slice: &mut Slice,
    ctx_indices: &[i16],
    num_idx: u8,
    mut k: i16,
    sign: i16,
    u_coff: u8,
  ) -> CabacResult<i16> {
    let tuval = self.tu(slice, ctx_indices, num_idx, u_coff)?;
    let mut rval = tuval as i16;
    if tuval >= u_coff {
      loop {
        if self.bypass(slice)? == 0 {
          break;
        }
        rval += 1 << k;
        k += 1;
      }
      while k > 0 {
        k -= 1;
        rval += (self.bypass(slice)? as i16) << k;
      }
    }
    Ok(if rval != 0 && sign != 0 {
      if self.bypass(slice)? != 0 {
        -rval
      } else {
        rval
      }
    } else {
      rval
    })
  }

  pub fn tu(
    &mut self,
    slice: &mut Slice,
    ctx_indices: &[i16],
    num_idx: u8,
    c_max: u8,
  ) -> CabacResult<u8> {
    let mut i = 0;
    while i < c_max {
      if self.decision(
        slice,
        ctx_indices[if i >= num_idx { num_idx - 1 } else { i } as usize],
      )? == 0
      {
        break;
      }
      i += 1;
    }
    Ok(i)
  }

  pub fn decision(&mut self, slice: &mut Slice, ctx_idx: i16) -> CabacResult<u8> {
    let bin_val;
    if ctx_idx == -1 {
      return self.bypass(slice);
    }
    if ctx_idx == CTXIDX_TERMINATE {
      return self.terminate(slice);
    }

    let ctx_idx = ctx_idx as usize;
    let p_state_idx = self.p_state_idx[ctx_idx] as usize;
    let q_cod_i_range_idx = (self.cod_i_range >> 6 & 3) as usize;

    let cod_i_range_lps = RANGE_TAB_LPS[p_state_idx][q_cod_i_range_idx] as u16;
    self.cod_i_range -= cod_i_range_lps;
    if self.cod_i_offset >= self.cod_i_range {
      bin_val = (self.val_mps[ctx_idx] == 0) as u8;
      self.cod_i_offset -= self.cod_i_range;
      self.cod_i_range = cod_i_range_lps;
    } else {
      bin_val = self.val_mps[ctx_idx];
    }
    if bin_val == self.val_mps[ctx_idx] {
      self.p_state_idx[ctx_idx] = TRANS_IDX_MPS[p_state_idx] as i16;
    } else {
      if p_state_idx == 0 {
        self.val_mps[ctx_idx] = (self.val_mps[ctx_idx] == 0) as u8;
      }
      self.p_state_idx[ctx_idx] = TRANS_IDX_LPS[p_state_idx] as i16;
    }
    self.renorm(slice)?;
    self.bin_count += 1;
    Ok(bin_val)
  }

  pub fn bypass(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let bin_val;
    self.cod_i_offset <<= 1;
    let bit = slice.stream.bit();
    self.cod_i_offset |= bit as u16;
    if self.cod_i_offset >= self.cod_i_range {
      bin_val = 1;
      self.cod_i_offset -= self.cod_i_range;
    } else {
      bin_val = 0;
    }
    self.bin_count += 1;
    Ok(bin_val)
  }

  pub fn terminate(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let bin_val;
    self.cod_i_range -= 2;
    if self.cod_i_offset >= self.cod_i_range {
      bin_val = 1;
    } else {
      bin_val = 0;
      self.renorm(slice)?;
    }
    self.bin_count += 1;
    Ok(bin_val)
  }

  pub fn renorm(&mut self, slice: &mut Slice) -> CabacResult {
    while self.cod_i_range < 256 {
      self.cod_i_range <<= 1;
      self.cod_i_offset <<= 1;
      let bit = slice.stream.bit();
      self.cod_i_offset |= bit as u16;
    }
    Ok(())
  }

  fn init_context_variables(slice: &Slice) -> ([i16; CTX_IDX_COUNT], [u8; CTX_IDX_COUNT]) {
    let mut p_state_idx = [0; CTX_IDX_COUNT];
    let mut val_mps = [0; CTX_IDX_COUNT];

    for (ctx_idx, init) in CTX_INIT_TABLE.iter().enumerate() {
      let (m, n) = init[slice.cabac_init_mode];
      let pre_ctx_state = clamp(((m * clamp(slice.sliceqpy, 0, 51)) >> 4) + n, 1, 126);
      if pre_ctx_state < 64 {
        p_state_idx[ctx_idx] = 63 - pre_ctx_state;
        val_mps[ctx_idx] = 0;
      } else {
        p_state_idx[ctx_idx] = pre_ctx_state - 64;
        val_mps[ctx_idx] = 1;
      }
    }
    (p_state_idx, val_mps)
  }

  fn init_decoding_engine(slice: &mut Slice) -> CabacResult<(u16, u16)> {
    let cod_i_range = 510;
    let cod_i_offset = slice.stream.bits_into(9);

    if cod_i_offset == 510 || cod_i_offset == 511 {
      return Err(CabacError::Engine);
    }

    Ok((cod_i_range, cod_i_offset))
  }
}
