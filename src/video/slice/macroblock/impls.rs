use super::*;

impl Macroblock {
  pub fn mvd_lx(&self, x: usize, mb_part_idx: usize, sub_mb_part_idx: usize, y: usize) -> isize {
    self.mvd[x][mb_part_idx * self.mb_type.partitions() + sub_mb_part_idx][y]
  }

  pub fn set_mb_type(&mut self, mb_type: u8) {
    self.mb_type = MbType::new(mb_type, self.transform_size_8x8_flag != 0);
  }

  pub fn set_sub_mb_type(&mut self, idx: usize, sub_mb_type: u8) {
    self.sub_mb_type[idx] = SubMbType::new(sub_mb_type);
  }

  pub fn blk_idx4x4(&self, x: isize, y: isize, max_w: isize, max_h: isize) -> isize {
    if self.mb_type.is_unavailable() {
      return -1;
    }
    MbPosition::blk_idx4x4(x, y, max_w, max_h)
  }

  pub fn blk_idx8x8(&self, x: isize, y: isize, max_w: isize, max_h: isize) -> isize {
    if self.mb_type.is_unavailable() {
      return -1;
    }
    MbPosition::blk_idx8x8(x, y, max_w, max_h)
  }

  pub fn update_intra_pred_mode(&mut self) {
    if let MbType::Intra { code, .. } = self.mb_type {
      self.mb_type = MbType::new(code, self.transform_size_8x8_flag != 0);
    }
  }

  pub const fn empty() -> Self {
    Self {
      mb_field_decoding_flag: false,
      mb_type: MbType::Intra {
        code: 0,
        intra_pred_mode: 0,
        part_pred_mode: PartPredMode::Intra4x4,
        coded_block_pattern_chroma: 0,
        coded_block_pattern_luma: 0,
      },
      coded_block_pattern: 0,
      transform_size_8x8_flag: 0,
      mb_qp_delta: 0,
      qpy: 0,
      qp1y: 0,
      qp1c: 0,
      qpc: 0,
      qsy: 0,
      qsc: 0,
      intra4x4_pred_mode: [0; 16],
      intra8x8_pred_mode: [0; 4],
      luma_pred_samples: [[[0; 4]; 4]; 16],
      luma16x16_pred_samples: [[0; 16]; 16],
      luma8x8_pred_samples: [[[0; 8]; 8]; 4],
      chroma_pred_samples: [[0; 16]; 8],
      mv_l0: [[[0; 2]; 4]; 4],
      mv_l1: [[[0; 2]; 4]; 4],
      ref_idxl0: [0; 4],
      ref_idxl1: [0; 4],
      pred_flagl0: [0; 4],
      pred_flagl1: [0; 4],
      mb_skip_flag: false,
      transform_bypass_mode_flag: false,
      transform_bypass_flag: false,
      pcm_sample_luma: [0; 256],
      pcm_sample_chroma: [0; 512],
      prev_intra4x4_pred_mode_flag: [0; 16],
      rem_intra4x4_pred_mode: [0; 16],
      prev_intra8x8_pred_mode_flag: [0; 4],
      rem_intra8x8_pred_mode: [0; 4],
      intra_chroma_pred_mode: 0,
      sub_mb_type: [SubMbType::empty(), SubMbType::empty(), SubMbType::empty(), SubMbType::empty()],
      ref_idx: [[0; 4]; 2],
      mvd: [[[0; 2]; 16]; 2],
      block_luma_dc: [[0; 16]; 3],
      block_luma_ac: [[[0; 15]; 16]; 3],
      block_luma_4x4: [[[0; 16]; 16]; 3],
      block_luma_8x8: [[[0; 64]; 4]; 3],
      block_chroma_dc: [[0; 8]; 2],
      block_chroma_ac: [[[0; 15]; 8]; 2],
      total_coeff: [[0; 16]; 3],
      coded_block_flag: [[0; 17]; 3],
    }
  }

  pub const fn empty_unavailable(coded_block_flag: u8) -> Self {
    Self {
      mb_type: MbType::Unavailable,
      coded_block_pattern: 0x0F,
      coded_block_flag: [[coded_block_flag; 17], [coded_block_flag; 17], [coded_block_flag; 17]],
      ..Self::empty()
    }
  }

  pub const fn unavailable(inter: u8) -> &'static Self {
    if inter != 0 {
      &MB_UNAVAILABLE_INTER
    } else {
      &MB_UNAVAILABLE_INTRA
    }
  }

  pub fn index(&self, macroblocks: &[Macroblock]) -> isize {
    let index = unsafe { (self as *const Macroblock).offset_from(macroblocks.as_ptr()) };
    if index < 0 || index >= macroblocks.len() as isize {
      -1
    } else {
      index
    }
  }

  pub fn offset<'a>(&'a self, macroblocks: &'a [Macroblock], offset: isize) -> MacroblockResult<&'a Self> {
    let index = unsafe { (self as *const Macroblock).offset_from(macroblocks.as_ptr()) } + offset;
    if index > macroblocks.len() as isize || index < 0 {
      return Err(MacroblockError::MacroblockBounds(index, macroblocks.len()));
    }
    Ok(&macroblocks[index as usize])
  }

  #[allow(unused)]
  pub fn offset_mut<'a>(&'a self, macroblocks: &'a mut [Macroblock], offset: isize) -> MacroblockResult<&'a mut Self> {
    let index = unsafe { (self as *const Macroblock).offset_from(macroblocks.as_ptr()) } + offset;
    if index > macroblocks.len() as isize || index < 0 {
      return Err(MacroblockError::MacroblockBounds(index, macroblocks.len()));
    }
    Ok(&mut macroblocks[index as usize])
  }
}

impl Eq for Macroblock {}

impl PartialEq for Macroblock {
  fn eq(&self, other: &Self) -> bool {
    *self.mb_type == *other.mb_type
  }
}

impl Hash for Macroblock {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.mb_type.hash(state);
  }
}
