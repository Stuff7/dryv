use super::*;

impl CabacContext {
  #[allow(clippy::too_many_arguments)]
  pub fn ueg(&mut self, slice: &mut Slice, ctx_indices: &[isize], num_idx: u8, mut k: isize, sign: isize, u_coff: u8) -> CabacResult<isize> {
    let tuval = self.tu(slice, ctx_indices, num_idx, u_coff)?;
    let mut rval = tuval as isize;
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
        rval += (self.bypass(slice)? as isize) << k;
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

  pub fn tu(&mut self, slice: &mut Slice, ctx_indices: &[isize], num_idx: u8, c_max: u8) -> CabacResult<u8> {
    let mut i = 0;
    while i < c_max {
      if self.decision(slice, ctx_indices[if i >= num_idx { num_idx - 1 } else { i } as usize])? == 0 {
        break;
      }
      i += 1;
    }
    Ok(i)
  }

  pub fn decision(&mut self, slice: &mut Slice, ctx_idx: isize) -> CabacResult<u8> {
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
      self.p_state_idx[ctx_idx] = TRANS_IDX_MPS[p_state_idx] as isize;
    } else {
      if p_state_idx == 0 {
        self.val_mps[ctx_idx] = (self.val_mps[ctx_idx] == 0) as u8;
      }
      self.p_state_idx[ctx_idx] = TRANS_IDX_LPS[p_state_idx] as isize;
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

  pub fn se(&mut self, slice: &mut Slice, table: &[SEValue], ctx_indices: &[isize], val: &mut u8) -> CabacResult {
    let mut byte = [0u8; 8];
    let mut bin_idx = [-1isize; 8];
    for se_value in table {
      let mut j = 0;
      for bit in se_value.bits {
        if bin_idx[j] == -1 {
          bin_idx[j] = bit.bin_idx as isize;
          byte[j] = self.decision(slice, ctx_indices[bin_idx[j] as usize])?;
        }
        if bin_idx[j] != bit.bin_idx as isize {
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
}
