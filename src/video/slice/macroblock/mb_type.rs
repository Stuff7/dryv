use super::*;

impl Deref for Macroblock {
  type Target = MbType;

  fn deref(&self) -> &Self::Target {
    &self.mb_type
  }
}

#[derive(Debug)]
pub enum MbType {
  Unavailable,
  Pcm,
  Intra {
    code: u8,
    intra_pred_mode: u8,
    part_pred_mode: PartPredMode,
    coded_block_pattern_chroma: u8,
    coded_block_pattern_luma: u8,
  },
  Inter {
    code: u8,
    num_mb_part: i8,
    part_pred_mode: [PartPredMode; 2],
    part_width: u8,
    part_height: u8,
  },
}

impl Deref for MbType {
  type Target = u8;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Pcm => &MB_TYPE_I_PCM,
      Self::Intra { code, .. } => code,
      Self::Inter { code, .. } => code,
      Self::Unavailable => &MB_TYPE_UNAVAILABLE,
    }
  }
}

impl MbType {
  pub fn new(mb_type: u8, transform_size_8x8_flag: bool) -> Self {
    match mb_type.cmp(&MB_TYPE_I_PCM) {
      Ordering::Less => {
        let (part_pred_mode, intra_pred_mode, coded_block_pattern_chroma, coded_block_pattern_luma) =
          mb_type_intra(mb_type, transform_size_8x8_flag);
        MbType::Intra {
          code: mb_type,
          intra_pred_mode,
          part_pred_mode,
          coded_block_pattern_luma,
          coded_block_pattern_chroma,
        }
      }
      Ordering::Equal => MbType::Pcm,
      Ordering::Greater => {
        let (num_mb_part, part_pred_mode, part_width, part_height) = mb_type_inter(mb_type);
        MbType::Inter {
          code: mb_type,
          num_mb_part,
          part_pred_mode,
          part_width,
          part_height,
        }
      }
    }
  }

  pub fn intra16x16_pred_mode(&self) -> u8 {
    match self {
      Self::Intra {
        intra_pred_mode, ..
      } => *intra_pred_mode,
      _ => 255,
    }
  }

  pub fn mode(&self) -> &PartPredMode {
    match self {
      Self::Intra { part_pred_mode, .. } => part_pred_mode,
      Self::Inter { part_pred_mode, .. } => &part_pred_mode[0],
      _ => &PartPredMode::NA,
    }
  }

  pub fn inter_mode(&self, idx: usize) -> &PartPredMode {
    match self {
      Self::Intra { part_pred_mode, .. } => part_pred_mode,
      Self::Inter { part_pred_mode, .. } => &part_pred_mode[std::cmp::min(idx, 1)],
      _ => &PartPredMode::NA,
    }
  }

  pub fn num_mb_part(&self) -> usize {
    match self {
      Self::Inter { num_mb_part, .. } => *num_mb_part as usize,
      _ => 0,
    }
  }

  pub fn coded_block_pattern_luma(&self) -> usize {
    match self {
      Self::Intra {
        coded_block_pattern_luma,
        ..
      } => *coded_block_pattern_luma as usize,
      _ => usize::MAX,
    }
  }

  pub fn coded_block_pattern_chroma(&self) -> usize {
    match self {
      Self::Intra {
        coded_block_pattern_chroma,
        ..
      } => *coded_block_pattern_chroma as usize,
      _ => usize::MAX,
    }
  }

  pub fn mb_part_width(&self) -> isize {
    match self {
      Self::Inter { part_width, .. } => *part_width as isize,
      _ => 0,
    }
  }

  pub fn mb_part_height(&self) -> isize {
    match self {
      Self::Inter { part_height, .. } => *part_height as isize,
      _ => 0,
    }
  }

  pub fn name(&self) -> &str {
    name_mb_type(**self)
  }

  pub fn is_intra_16x16(&self) -> bool {
    is_intra_16x16_mb_type(**self)
  }

  pub fn is_inter(&self) -> bool {
    **self >= MB_TYPE_P_L0_16X16
  }

  pub fn is_unavailable(&self) -> bool {
    matches!(self, Self::Unavailable)
  }

  pub fn is_available(&self) -> bool {
    !matches!(self, Self::Unavailable)
  }

  pub fn is_si(&self) -> bool {
    **self == MB_TYPE_SI
  }

  pub fn is_i_nxn(&self) -> bool {
    **self == MB_TYPE_I_NXN
  }

  pub fn is_b_direct_16x16(&self) -> bool {
    **self == MB_TYPE_B_DIRECT_16X16
  }

  pub fn is_b8x8(&self) -> bool {
    **self == MB_TYPE_B_8X8
  }

  pub fn is_p8x8(&self) -> bool {
    **self == MB_TYPE_P_8X8
  }

  pub fn is_pcm(&self) -> bool {
    matches!(self, Self::Pcm)
  }

  pub fn is_p_8x8ref0(&self) -> bool {
    **self == MB_TYPE_P_8X8REF0
  }

  pub fn is_skip(&self) -> bool {
    matches!(**self, MB_TYPE_P_SKIP | MB_TYPE_B_SKIP)
  }

  pub fn is_b_skip(&self) -> bool {
    matches!(**self, MB_TYPE_B_SKIP)
  }

  pub fn is_p_skip(&self) -> bool {
    matches!(**self, MB_TYPE_P_SKIP)
  }

  pub fn is_submb(&self) -> bool {
    matches!(**self, MB_TYPE_P_8X8 | MB_TYPE_P_8X8REF0 | MB_TYPE_B_8X8)
  }
}
