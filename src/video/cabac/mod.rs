pub mod binarization;
pub mod consts;
pub mod derivation;
pub mod prediction;
pub mod residual;
pub mod syntax_element;
pub mod table;

pub use binarization::*;
pub use consts::*;
pub use derivation::*;
pub use prediction::*;
pub use residual::*;
pub use syntax_element::*;
pub use table::*;

use super::slice::{
  consts::*,
  header::SliceType,
  macroblock::{BlockSize, Macroblock, MacroblockError, MbPosition},
  Slice,
};
use crate::math::clamp;
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
  pub p_state_idx: [isize; CTX_IDX_COUNT],

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
    let transform_8x8_mode_flag = slice.pps.extra_rbsp_data.as_ref().is_some_and(|pps| pps.transform_8x8_mode_flag);
    let direct_8x8_inference_flag = slice.sps.direct_8x8_inference_flag;
    let bit_depth_luma_minus8 = slice.sps.bit_depth_luma_minus8 as usize;
    let bit_depth_chroma_minus8 = slice.sps.bit_depth_chroma_minus8 as usize;
    self.mb_type(slice)?;
    if slice.mb().mb_type.is_pcm() {
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
      if slice.mb().mb_type.is_submb() {
        self.sub_mb_pred(slice)?;
        for i in 0..4 {
          if !slice.mb().sub_mb_type[i].is_b_direct8x8() {
            if SUB_MB_PART_INFO[*slice.mb().sub_mb_type[i] as usize][0] != 0 {
              no_sub_mb_part_size_less_than8x8_flag = 0;
            }
          } else if !direct_8x8_inference_flag {
            no_sub_mb_part_size_less_than8x8_flag = 0;
          }
        }
        slice.mb_mut().intra_chroma_pred_mode = 0;
      } else {
        if slice.mb().mb_type.is_i_nxn() || slice.mb().mb_type.is_si() {
          if transform_8x8_mode_flag {
            slice.mb_mut().transform_size_8x8_flag = self.transform_size_8x8_flag(slice)?;
          } else {
            slice.mb_mut().transform_size_8x8_flag = 0;
          }
        }
        self.mb_pred(slice)?;
      }
      if slice.mb().mb_type.is_i_nxn() || slice.mb().mb_type.is_si() || *slice.mb().mb_type >= MB_TYPE_SI {
        let has_chroma = slice.chroma_array_type < 3 && slice.chroma_array_type != 0;
        slice.mb_mut().coded_block_pattern = self.coded_block_pattern(slice, has_chroma)?;
        if *slice.mb().mb_type >= MB_TYPE_SI {
          if (slice.mb().coded_block_pattern & 0xf) != 0
            && transform_8x8_mode_flag
            && no_sub_mb_part_size_less_than8x8_flag != 0
            && (!slice.mb().mb_type.is_b_direct_16x16() || direct_8x8_inference_flag)
          {
            slice.mb_mut().transform_size_8x8_flag = self.transform_size_8x8_flag(slice)?;
          } else {
            slice.mb_mut().transform_size_8x8_flag = 0;
          }
        }
      } else {
        let mut infer_cbp = (((*slice.mb().mb_type - MB_TYPE_I_16X16_0_0_0) >> 2) % 3) << 4;
        if *slice.mb().mb_type >= MB_TYPE_I_16X16_0_0_1 {
          infer_cbp |= 0xf;
        }
        slice.mb_mut().coded_block_pattern = infer_cbp;
        slice.mb_mut().transform_size_8x8_flag = 0;
      }
      if slice.mb().coded_block_pattern != 0 || slice.mb().mb_type.is_intra_16x16() {
        slice.mb_mut().mb_qp_delta = self.mb_qp_delta(slice)?;
      } else {
        slice.mb_mut().mb_qp_delta = 0;
      }
      self.residual(slice, 0, 15)?;
    }
    Ok(())
  }

  fn init_context_variables(slice: &Slice) -> ([isize; CTX_IDX_COUNT], [u8; CTX_IDX_COUNT]) {
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
