use super::*;

impl CabacContext {
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

  pub fn residual_luma(&mut self, slice: &mut Slice, start: usize, end: usize, which: usize) -> CabacResult {
    if start == 0 && slice.mb().mb_type.is_intra_16x16() {
      self.residual_cabac(slice, ResidualBlock::LumaDc(which), LUMA_CAT_TAB[which][0], 0, 0, 15, 16, 1)?;
    } else {
      slice.mb_mut().coded_block_flag[which][16] = 0;
    }
    let n = if slice.mb().mb_type.is_intra_16x16() { 15 } else { 16 };
    let ss = if slice.mb().mb_type.is_intra_16x16() {
      if start != 0 {
        start - 1
      } else {
        0
      }
    } else {
      start
    };
    let se = if slice.mb().mb_type.is_intra_16x16() { end - 1 } else { end };
    if slice.mb().transform_size_8x8_flag == 0 {
      for i in 0..16 {
        let mut tmp = [0; 16];
        let cat;
        if slice.mb().transform_size_8x8_flag != 0 {
          for (j, tmp) in tmp.iter_mut().enumerate() {
            *tmp = slice.mb().block_luma_8x8[which][i >> 2][4 * j + (i & 3)];
          }
          cat = LUMA_CAT_TAB[which][3];
        } else if slice.mb().mb_type.is_intra_16x16() {
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
        } else if slice.mb().mb_type.is_intra_16x16() {
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
        significant_coeff_flag[i] = self.significant_coeff_flag(slice, field as usize, cat, i, 0)?;
        if significant_coeff_flag[i] != 0 {
          last_significant_coeff_flag[i] = self.significant_coeff_flag(slice, field as usize, cat, i, 1)?;
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
}
