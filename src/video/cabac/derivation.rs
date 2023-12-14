use super::*;

impl CabacContext {
  pub fn coeff_abs_level_minus1(&mut self, slice: &mut Slice, cat: u8, num1: isize, numgt1: isize) -> CabacResult<isize> {
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
    ctx_idx[1] = COEFF_ABS_LEVEL_MINUS1_BASE_CTX[cat as usize] + 5 + if numgt1 > clamp { clamp } else { numgt1 };
    self.ueg(slice, &ctx_idx, 2, 0, 0, 14)
  }

  pub fn significant_coeff_flag(&mut self, slice: &mut Slice, field: usize, cat: u8, idx: usize, last: usize) -> CabacResult<u8> {
    let mut ctx_inc: isize;
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
        ctx_inc = idx as isize;
      }
      CTXBLOCKCAT_CHROMA_DC => {
        ctx_inc = idx as isize / slice.chroma_array_type as isize;
        if ctx_inc > 2 {
          ctx_inc = 2;
        }
      }
      CTXBLOCKCAT_LUMA_8X8 | CTXBLOCKCAT_CB_8X8 | CTXBLOCKCAT_CR_8X8 => {
        if last != 0 {
          ctx_inc = SIGNIFICANT_COEFF_FLAG_TAB8X8[idx][2] as isize;
        } else {
          ctx_inc = SIGNIFICANT_COEFF_FLAG_TAB8X8[idx][field] as isize;
        }
      }
      cat => panic!("Invalid ctx_block_cat passed to significant_coeff_flag {cat}"),
    }
    let ctx_idx = SIGNIFICANT_COEFF_FLAG_BASE_CTX[last][field][cat as usize] + ctx_inc;
    self.decision(slice, ctx_idx)
  }

  pub fn coded_block_flag(&mut self, slice: &mut Slice, cat: u8, mut idx: isize) -> CabacResult<u8> {
    let mb_t = slice.mb_nb_p(MbPosition::This, 0);
    let inter = mb_t.mb_type.is_inter() as u8;
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
      CTXBLOCKCAT_LUMA_AC | CTXBLOCKCAT_LUMA_4X4 | CTXBLOCKCAT_CB_AC | CTXBLOCKCAT_CB_4X4 | CTXBLOCKCAT_CR_AC | CTXBLOCKCAT_CR_4X4 => {
        mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B4x4, inter, idx, &mut idx_a)?;
        mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B4x4, inter, idx, &mut idx_b)?;
      }
      CTXBLOCKCAT_LUMA_8X8 | CTXBLOCKCAT_CB_8X8 | CTXBLOCKCAT_CR_8X8 => {
        mb_a = slice.mb_nb_b(MbPosition::A, BlockSize::B8x8, inter, idx, &mut idx_a)?;
        mb_b = slice.mb_nb_b(MbPosition::B, BlockSize::B8x8, inter, idx, &mut idx_b)?;
        idx_a *= 4;
        idx_b *= 4;
        if mb_a.transform_size_8x8_flag == 0 && !mb_a.mb_type.is_pcm() && mb_a.mb_type.is_available() {
          Macroblock::unavailable(1);
        }
        if mb_b.transform_size_8x8_flag == 0 && !mb_b.mb_type.is_pcm() && mb_b.mb_type.is_available() {
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
    let cond_term_flag_a = mb_a.coded_block_flag[which as usize][idx_a as usize] as isize;
    let cond_term_flag_b = mb_b.coded_block_flag[which as usize][idx_b as usize] as isize;
    let ctx_idx = CODED_BLOCK_FLAG_BASE_CTX[cat as usize] + cond_term_flag_a + cond_term_flag_b * 2;
    self.decision(slice, ctx_idx)
  }

  pub fn mb_qp_delta(&mut self, slice: &mut Slice) -> CabacResult<isize> {
    let mut ctx_idx = [0; 3];
    if slice.prev_mb_addr != -1 && slice.macroblocks[slice.prev_mb_addr as usize].mb_qp_delta != 0 {
      ctx_idx[0] = CTXIDX_MB_QP_DELTA + 1;
    } else {
      ctx_idx[0] = CTXIDX_MB_QP_DELTA;
    }
    ctx_idx[1] = CTXIDX_MB_QP_DELTA + 2;
    ctx_idx[2] = CTXIDX_MB_QP_DELTA + 3;
    let tmp = self.tu(slice, &ctx_idx, 3, -1i8 as u8)? as isize;
    Ok(if (tmp & 1) != 0 { (tmp + 1) >> 1 } else { -(tmp >> 1) })
  }

  #[allow(invalid_reference_casting)]
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
      }) == 0) as isize;
      let cond_term_flag_b = ((if std::ptr::eq(mb_b as *const _, mb_t as *const _) {
        bit[idx_b as usize]
      } else {
        mb_b.coded_block_pattern >> idx_b & 1
      }) == 0) as isize;
      ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_LUMA + cond_term_flag_a + cond_term_flag_b * 2;
      bit[i as usize] = self.decision(unsafe { &mut *(slice as *const _ as *mut _) }, ctx_idx)?;
    }
    if has_chroma {
      mb_a = slice.mb_nb(MbPosition::A, 0)?;
      mb_b = slice.mb_nb(MbPosition::B, 0)?;
      let cond_term_flag_a = ((mb_a.coded_block_pattern >> 4) > 0) as isize;
      let cond_term_flag_b = ((mb_b.coded_block_pattern >> 4) > 0) as isize;
      ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_CHROMA + cond_term_flag_a + cond_term_flag_b * 2;
      bit[4] = self.decision(unsafe { &mut *(slice as *const _ as *mut _) }, ctx_idx)?;
      if bit[4] != 0 {
        let cond_term_flag_a = ((mb_a.coded_block_pattern >> 4) > 1) as isize;
        let cond_term_flag_b = ((mb_b.coded_block_pattern >> 4) > 1) as isize;
        ctx_idx = CTXIDX_CODED_BLOCK_PATTERN_CHROMA + cond_term_flag_a + cond_term_flag_b * 2 + 4;
        bit[5] = self.decision(slice, ctx_idx)?;
      }
    }
    Ok(bit[0] | bit[1] << 1 | bit[2] << 2 | bit[3] << 3 | bit[4] << (4 + bit[5]))
  }

  pub fn transform_size_8x8_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let ctx_idx_offset = CTXIDX_TRANSFORM_SIZE_8X8_FLAG;
    let cond_term_flag_a = slice.mb_nb(MbPosition::A, 0)?.transform_size_8x8_flag;
    let cond_term_flag_b = slice.mb_nb(MbPosition::B, 0)?.transform_size_8x8_flag;
    let ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc as isize)
  }

  pub fn mvd(&mut self, slice: &mut Slice, idx: isize, comp: usize, which: usize) -> CabacResult<isize> {
    let base_idx = if comp != 0 { CTXIDX_MVD_Y } else { CTXIDX_MVD_X };
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
    let ctx_idx = [base_idx + inc, base_idx + 3, base_idx + 4, base_idx + 5, base_idx + 6];
    self.ueg(slice, &ctx_idx, 5, 3, 1, 9)
  }

  pub fn ref_idx(&mut self, slice: &mut Slice, idx: isize, which: usize, max: u16) -> CabacResult<u8> {
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
    let cond_term_flag_a = (mb_a.ref_idx[which][idx_a as usize] > thr_a as u8) as isize;
    let cond_term_flag_b = (mb_b.ref_idx[which][idx_b as usize] > thr_b as u8) as isize;
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
      let bidx = [CTXIDX_SUB_MB_TYPE_P, CTXIDX_SUB_MB_TYPE_P + 1, CTXIDX_SUB_MB_TYPE_P + 2];
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
    let mut bidx = [0isize; 11];
    let mbt_a = *slice.mb_nb(MbPosition::A, 0)?.mb_type;
    let mbt_b = *slice.mb_nb(MbPosition::B, 0)?.mb_type;
    let mut cond_term_flag_a = mbt_a != MB_TYPE_UNAVAILABLE;
    let mut cond_term_flag_b = mbt_b != MB_TYPE_UNAVAILABLE;
    let slice_type = slice.slice_type;
    let mut val = *slice.mb().mb_type;
    match slice_type {
      SliceType::SI => {
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_SI;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_SI;
        let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
        bidx[7] = CTXIDX_MB_TYPE_SI_PRE + ctx_idx_inc;
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_I_NXN;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_I_NXN;
        let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
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
        let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
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
        cond_term_flag_a = cond_term_flag_a && mbt_a != MB_TYPE_B_SKIP && mbt_a != MB_TYPE_B_DIRECT_16X16;
        cond_term_flag_b = cond_term_flag_b && mbt_b != MB_TYPE_B_SKIP && mbt_b != MB_TYPE_B_DIRECT_16X16;
        let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
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
    slice.macroblocks[slice.curr_mb_addr as usize].set_mb_type(val);
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
    let cond_term_flag_a = mb_a.mb_type.is_available() && !mb_a.mb_type.is_skip();
    let cond_term_flag_b = mb_b.mb_type.is_available() && !mb_b.mb_type.is_skip();
    let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc)
  }

  pub fn mb_field_decoding_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let ctx_idx_offset = CTXIDX_MB_FIELD_DECODING_FLAG;
    let cond_term_flag_a = slice.mb_nb_p(MbPosition::A, 0).mb_field_decoding_flag;
    let cond_term_flag_b = slice.mb_nb_p(MbPosition::B, 0).mb_field_decoding_flag;
    let ctx_idx_inc = cond_term_flag_a as isize + cond_term_flag_b as isize;
    self.decision(slice, ctx_idx_offset + ctx_idx_inc)
  }
}
