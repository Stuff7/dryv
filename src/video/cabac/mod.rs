pub mod consts;
pub mod syntax_element;
pub mod table;

use super::slice::{
  consts::*,
  consts::{is_skip_mb_type, MB_TYPE_UNAVAILABLE},
  header::SliceType,
  macroblock::{MacroblockError, MbPosition},
  Slice,
};
use crate::math::clamp;
use consts::*;
use syntax_element::SEValue;
use syntax_element::*;
use table::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CabacError {
  #[error("Error cod_i_offset must not be 510 nor 511")]
  Engine,
  #[error("Renorm failed")]
  Renorm,
  #[error("Bypass failed")]
  Bypass,
  #[error("Inconsistent SE table")]
  SETable,
  #[error("No value from binarization")]
  Binarization,
  #[error("cabac_alignment_one_bit is not 1")]
  AlignmentOneBit,
  #[error("mb_skip_flag used in I/SI slice")]
  MbSkipFlagSlice,
  #[error(transparent)]
  Macroblock(#[from] MacroblockError),
  #[error("Ran out of bits")]
  Data,
  #[error("Invalid slice type for sub_mb_type")]
  SubMbType,
}

pub type CabacResult<T = ()> = Result<T, CabacError>;

pub struct CabacContext {
  /// Probability state index.
  pub p_state_idx: [i16; CTX_IDX_COUNT],
  /// Value of the most probable symbol.
  pub val_mps: [u8; CTX_IDX_COUNT],
  /// The status of the arithmetic decoding engine.
  pub cod_i_range: u16,
  pub cod_i_offset: u16,
  pub bin_count: i32,
}

impl CabacContext {
  pub fn new(slice: &mut Slice) -> CabacResult<Self> {
    if !slice.stream.skip_trailing_bits().bit_flag() {
      return Err(CabacError::AlignmentOneBit);
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

  pub fn sub_mb_type(&mut self, slice: &mut Slice, val: &mut u8) -> CabacResult {
    let slice_type = slice.slice_type;
    if slice_type.is_predictive() {
      let bidx = [
        CTXIDX_SUB_MB_TYPE_P,
        CTXIDX_SUB_MB_TYPE_P + 1,
        CTXIDX_SUB_MB_TYPE_P + 2,
      ];
      self.se(slice, SUB_MB_TYPE_P_TABLE, &bidx, val)
    } else if slice_type.is_bidirectional() {
      let bidx = [
        CTXIDX_SUB_MB_TYPE_B,
        CTXIDX_SUB_MB_TYPE_B + 1,
        CTXIDX_SUB_MB_TYPE_B + 2,
        CTXIDX_SUB_MB_TYPE_B + 3,
      ];
      self.se(slice, SUB_MB_TYPE_B_TABLE, &bidx, val)
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
    let mut val = slice.macroblocks[slice.curr_mb_addr].mb_type;
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
    slice.macroblocks[slice.curr_mb_addr].mb_type = val;
    Ok(())
  }

  pub fn mb_skip_flag(&mut self, slice: &mut Slice, bin_val: &mut u8) -> CabacResult {
    let ctx_idx_offset;
    let ctx_idx_inc;
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
    ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc, bin_val)
  }

  pub fn mb_field_decoding_flag(&mut self, slice: &mut Slice, bin_val: &mut u8) -> CabacResult {
    let ctx_idx_offset = CTXIDX_MB_FIELD_DECODING_FLAG;
    let cond_term_flag_a = slice.mb_nb_p(MbPosition::A, 0).mb_field_decoding_flag;
    let cond_term_flag_b = slice.mb_nb_p(MbPosition::B, 0).mb_field_decoding_flag;
    let ctx_idx_inc = cond_term_flag_a as i16 + cond_term_flag_b as i16;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc, bin_val)
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
          self.decision(slice, ctx_indices[bin_idx[j] as usize], &mut byte[j])?;
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

  pub fn ueg(
    &mut self,
    slice: &mut Slice,
    ctx_indices: &[i16],
    num_idx: u8,
    mut k: i16,
    sign: i16,
    u_coff: u8,
    val: &mut i16,
  ) -> CabacResult {
    let mut tuval = val.unsigned_abs() as u8;
    if tuval > u_coff {
      tuval = u_coff;
    }
    self.tu(slice, ctx_indices, num_idx, u_coff, &mut tuval)?;
    let mut rval = tuval;
    if tuval >= u_coff {
      let mut tmp = 0;
      loop {
        self.bypass(slice, &mut tmp)?;
        if tmp == 0 {
          break;
        }
        rval += 1 << k;
        k += 1;
      }
      while k > 0 {
        k -= 1;
        self.bypass(slice, &mut tmp)?;
        rval += tmp << k;
      }
    }
    let rval = rval as i16;
    if rval != 0 && sign != 0 {
      let mut s = (*val < 0) as u8;
      self.bypass(slice, &mut s)?;
      if s != 0 {
        *val = -rval;
      } else {
        *val = rval;
      }
    } else {
      *val = rval;
    }
    Ok(())
  }

  pub fn tu(
    &mut self,
    slice: &mut Slice,
    ctx_indices: &[i16],
    num_idx: u8,
    c_max: u8,
    val: &mut u8,
  ) -> CabacResult {
    let mut bit = 1u8;
    let mut i = 0;
    while i < c_max {
      self.decision(
        slice,
        ctx_indices[if i >= num_idx { num_idx - 1 } else { i } as usize],
        &mut bit,
      )?;
      if bit == 0 {
        break;
      }
      i += 1;
    }
    *val = i;
    Ok(())
  }

  pub fn decision(&mut self, slice: &mut Slice, ctx_idx: i16, bin_val: &mut u8) -> CabacResult {
    if ctx_idx == -1 {
      return self.bypass(slice, bin_val);
    }
    if ctx_idx == CTXIDX_TERMINATE {
      return self.terminate(slice, bin_val);
    }

    let ctx_idx = ctx_idx as usize;
    let p_state_idx = self.p_state_idx[ctx_idx] as usize;
    let q_cod_i_range_idx = (self.cod_i_range >> 6 & 3) as usize;

    let cod_i_range_lps = RANGE_TAB_LPS[p_state_idx][q_cod_i_range_idx] as u16;
    self.cod_i_range -= cod_i_range_lps;
    if self.cod_i_offset >= self.cod_i_range {
      *bin_val = (self.val_mps[ctx_idx] == 0) as u8;
      self.cod_i_offset -= self.cod_i_range;
      self.cod_i_range = cod_i_range_lps;
    } else {
      *bin_val = self.val_mps[ctx_idx];
    }
    if *bin_val == self.val_mps[ctx_idx] {
      self.p_state_idx[ctx_idx] = TRANS_IDX_MPS[p_state_idx] as i16;
    } else {
      if p_state_idx == 0 {
        self.val_mps[ctx_idx] = !self.val_mps[ctx_idx];
      }
      self.p_state_idx[ctx_idx] = TRANS_IDX_LPS[p_state_idx] as i16;
    }
    self.renorm(slice)?;
    self.bin_count += 1;
    Ok(())
  }

  pub fn bypass(&mut self, slice: &mut Slice, bin_val: &mut u8) -> CabacResult {
    self.cod_i_offset <<= 1;
    let bit = slice.stream.bit();
    if bit == 1 {
      return Err(CabacError::Bypass);
    }
    self.cod_i_offset |= bit as u16;
    if (self.cod_i_offset >= self.cod_i_range) {
      *bin_val = 1;
      self.cod_i_offset -= self.cod_i_range;
    } else {
      *bin_val = 0;
    }
    self.bin_count += 1;
    Ok(())
  }

  pub fn terminate(&mut self, slice: &mut Slice, bin_val: &mut u8) -> CabacResult {
    self.cod_i_range -= 2;
    if self.cod_i_offset >= self.cod_i_range {
      *bin_val = 1;
    } else {
      *bin_val = 0;
      self.renorm(slice)?;
    }
    self.bin_count += 1;
    Ok(())
  }

  pub fn renorm(&mut self, slice: &mut Slice) -> CabacResult {
    while self.cod_i_range < 256 {
      self.cod_i_range <<= 1;
      self.cod_i_offset <<= 1;
      let bit = slice.stream.bit();
      if bit == 1 {
        return Err(CabacError::Renorm);
      }
      self.cod_i_offset |= bit as u16;
    }
    Ok(())
  }

  fn init_context_variables(slice: &Slice) -> ([i16; CTX_IDX_COUNT], [u8; CTX_IDX_COUNT]) {
    let mut p_state_idx = [0; CTX_IDX_COUNT];
    let mut val_mps = [0; CTX_IDX_COUNT];

    for (ctx_idx, init) in CTX_INIT_TABLE.iter().enumerate() {
      let (m, n) = init[slice.cabac_init_mode];
      let pre_ctx_state = clamp(((m * clamp(0, 51, slice.sliceqpy)) >> 4) + n, 1, 126);
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
