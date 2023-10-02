mod consts;
mod syntax_element;
mod table;

use super::slice::Slice;
use crate::math::clamp;
use consts::*;
use syntax_element::SEValue;
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
}

pub type CabacResult<T = ()> = Result<T, CabacError>;

pub struct CabacContext<'a> {
  pub slice: &'a mut Slice<'a>,
  /// Probability state index.
  pub p_state_idx: [i16; CTX_IDX_COUNT],
  /// Value of the most probable symbol.
  pub val_mps: [u8; CTX_IDX_COUNT],
  /// The status of the arithmetic decoding engine.
  pub cod_i_range: u16,
  pub cod_i_offset: u16,
  pub bin_count: i32,
}

impl<'a> CabacContext<'a> {
  pub fn new(slice: &'a mut Slice<'a>) -> CabacResult<Self> {
    let (p_state_idx, val_mps) = Self::init_context_variables(slice);
    let (cod_i_range, cod_i_offset) = Self::init_decoding_engine(slice)?;

    Ok(Self {
      slice,
      p_state_idx,
      val_mps,
      cod_i_offset,
      cod_i_range,
      bin_count: 0,
    })
  }

  pub fn se(&mut self, table: &[SEValue], ctx_indices: &[i16], val: &mut u8) -> CabacResult {
    let mut byte = [0u8; 8];
    let mut bin_idx = [-1i16; 8];
    for se_value in table {
      let mut j = 0;
      for bit in se_value.bits {
        if bin_idx[j] == -1 {
          bin_idx[j] = bit.bin_idx as i16;
          self.decision(ctx_indices[bin_idx[j] as usize], &mut byte[j])?;
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
          return self.se(sub_table, ctx_indices, val);
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
    self.tu(ctx_indices, num_idx, u_coff, &mut tuval)?;
    let mut rval = tuval;
    if tuval >= u_coff {
      let mut tmp = 0;
      loop {
        self.bypass(&mut tmp)?;
        if tmp == 0 {
          break;
        }
        rval += 1 << k;
        k += 1;
      }
      while k > 0 {
        k -= 1;
        self.bypass(&mut tmp)?;
        rval += tmp << k;
      }
    }
    let rval = rval as i16;
    if rval != 0 && sign != 0 {
      let mut s = (*val < 0) as u8;
      self.bypass(&mut s)?;
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

  pub fn tu(&mut self, ctx_indices: &[i16], num_idx: u8, c_max: u8, val: &mut u8) -> CabacResult {
    let mut bit = 1u8;
    let mut i = 0;
    while i < c_max {
      self.decision(
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

  pub fn decision(&mut self, ctx_idx: i16, bin_val: &mut u8) -> CabacResult {
    if ctx_idx == -1 {
      return self.bypass(bin_val);
    }
    if ctx_idx == CTXIDX_TERMINATE {
      return self.terminate(bin_val);
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
    self.renorm()?;
    self.bin_count += 1;
    Ok(())
  }

  pub fn bypass(&mut self, bin_val: &mut u8) -> CabacResult {
    self.cod_i_offset <<= 1;
    let bit = self.slice.stream.bit();
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

  pub fn terminate(&mut self, bin_val: &mut u8) -> CabacResult {
    self.cod_i_range -= 2;
    if self.cod_i_offset >= self.cod_i_range {
      *bin_val = 1;
    } else {
      *bin_val = 0;
      self.renorm()?;
    }
    self.bin_count += 1;
    Ok(())
  }

  pub fn renorm(&mut self) -> CabacResult {
    while self.cod_i_range < 256 {
      self.cod_i_range <<= 1;
      self.cod_i_offset <<= 1;
      let bit = self.slice.stream.bit();
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
