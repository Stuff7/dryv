use super::*;

#[derive(Debug, Clone, Copy)]
pub enum PartPredMode {
  Intra4x4,
  Intra8x8,
  Intra16x16,
  PredL0,
  PredL1,
  Direct,
  BiPred,
  NA,
}

impl PartPredMode {
  pub fn is_inter_frame(&self) -> bool {
    matches!(self, Self::PredL0 | Self::PredL1 | Self::BiPred)
  }

  pub fn is_inter_mode(&self) -> bool {
    matches!(self, Self::Intra4x4 | Self::Intra8x8 | Self::Intra16x16)
  }

  pub fn is_predl0(&self) -> bool {
    matches!(self, Self::PredL0 | Self::BiPred)
  }

  pub fn is_predl1(&self) -> bool {
    matches!(self, Self::PredL1 | Self::BiPred)
  }

  pub fn is_intra_4x4(&self) -> bool {
    matches!(self, Self::Intra4x4)
  }

  pub fn is_intra_8x8(&self) -> bool {
    matches!(self, Self::Intra8x8)
  }

  pub fn is_intra_16x16(&self) -> bool {
    matches!(self, Self::Intra16x16)
  }
}

/// Table 7-11 - Macroblock types for I slices
/// Returns (MbPartPredMode, Intra16x16PredMode, CodedBlockPatternChroma, CodedBlockPatternLuma)
pub fn mb_type_intra(mb_type: u8, transform_size_8x8_flag: bool) -> (PartPredMode, u8, u8, u8) {
  if mb_type == MB_TYPE_I_NXN {
    return match transform_size_8x8_flag {
      false => (PartPredMode::Intra4x4, 0, 0, 0),
      true => (PartPredMode::Intra8x8, 0, 0, 0),
    };
  }
  match mb_type {
    MB_TYPE_I_16X16_0_0_0 => (PartPredMode::Intra16x16, 0, 0, 0),
    MB_TYPE_I_16X16_1_0_0 => (PartPredMode::Intra16x16, 1, 0, 0),
    MB_TYPE_I_16X16_2_0_0 => (PartPredMode::Intra16x16, 2, 0, 0),
    MB_TYPE_I_16X16_3_0_0 => (PartPredMode::Intra16x16, 3, 0, 0),
    MB_TYPE_I_16X16_0_1_0 => (PartPredMode::Intra16x16, 0, 1, 0),
    MB_TYPE_I_16X16_1_1_0 => (PartPredMode::Intra16x16, 1, 1, 0),
    MB_TYPE_I_16X16_2_1_0 => (PartPredMode::Intra16x16, 2, 1, 0),
    MB_TYPE_I_16X16_3_1_0 => (PartPredMode::Intra16x16, 3, 1, 0),
    MB_TYPE_I_16X16_0_2_0 => (PartPredMode::Intra16x16, 0, 2, 0),
    MB_TYPE_I_16X16_1_2_0 => (PartPredMode::Intra16x16, 1, 2, 0),
    MB_TYPE_I_16X16_2_2_0 => (PartPredMode::Intra16x16, 2, 2, 0),
    MB_TYPE_I_16X16_3_2_0 => (PartPredMode::Intra16x16, 3, 2, 0),
    MB_TYPE_I_16X16_0_0_1 => (PartPredMode::Intra16x16, 0, 0, 15),
    MB_TYPE_I_16X16_1_0_1 => (PartPredMode::Intra16x16, 1, 0, 15),
    MB_TYPE_I_16X16_2_0_1 => (PartPredMode::Intra16x16, 2, 0, 15),
    MB_TYPE_I_16X16_3_0_1 => (PartPredMode::Intra16x16, 3, 0, 15),
    MB_TYPE_I_16X16_0_1_1 => (PartPredMode::Intra16x16, 0, 1, 15),
    MB_TYPE_I_16X16_1_1_1 => (PartPredMode::Intra16x16, 1, 1, 15),
    MB_TYPE_I_16X16_2_1_1 => (PartPredMode::Intra16x16, 2, 1, 15),
    MB_TYPE_I_16X16_3_1_1 => (PartPredMode::Intra16x16, 3, 1, 15),
    MB_TYPE_I_16X16_0_2_1 => (PartPredMode::Intra16x16, 0, 2, 15),
    MB_TYPE_I_16X16_1_2_1 => (PartPredMode::Intra16x16, 1, 2, 15),
    MB_TYPE_I_16X16_2_2_1 => (PartPredMode::Intra16x16, 2, 2, 15),
    MB_TYPE_I_16X16_3_2_1 => (PartPredMode::Intra16x16, 3, 2, 15),
    n => panic!("Invalid intra mb_type {n}"),
  }
}

/// Table 7-13 - Macroblock type values 0 to 4 for P and SP slices
/// Table 7-14 - Macroblock type values 0 to 22 for B slices
/// Returns (NumMbPart, [MbPartPredMode; 1], MbPartWidth, MbPartHeight)
pub fn mb_type_inter(mb_type: u8) -> (i8, [PartPredMode; 2], u8, u8) {
  match mb_type {
    MB_TYPE_P_L0_16X16 => (1, [PartPredMode::PredL0, PartPredMode::NA], 16, 16),
    MB_TYPE_P_L0_L0_16X8 => (2, [PartPredMode::PredL0, PartPredMode::PredL0], 16, 8),
    MB_TYPE_P_L0_L0_8X16 => (2, [PartPredMode::PredL0, PartPredMode::PredL0], 8, 16),
    MB_TYPE_P_8X8 => (4, [PartPredMode::NA, PartPredMode::NA], 8, 8),
    MB_TYPE_P_8X8REF0 => (4, [PartPredMode::NA, PartPredMode::NA], 8, 8),
    MB_TYPE_P_SKIP => (1, [PartPredMode::PredL0, PartPredMode::NA], 16, 16),
    MB_TYPE_B_DIRECT_16X16 => (-1, [PartPredMode::Direct, PartPredMode::NA], 8, 8),
    MB_TYPE_B_L0_16X16 => (1, [PartPredMode::PredL0, PartPredMode::NA], 16, 16),
    MB_TYPE_B_L1_16X16 => (1, [PartPredMode::PredL1, PartPredMode::NA], 16, 16),
    MB_TYPE_B_BI_16X16 => (1, [PartPredMode::BiPred, PartPredMode::NA], 16, 16),
    MB_TYPE_B_L0_L0_16X8 => (2, [PartPredMode::PredL0, PartPredMode::PredL0], 16, 8),
    MB_TYPE_B_L0_L0_8X16 => (2, [PartPredMode::PredL0, PartPredMode::PredL0], 8, 16),
    MB_TYPE_B_L1_L1_16X8 => (2, [PartPredMode::PredL1, PartPredMode::PredL1], 16, 8),
    MB_TYPE_B_L1_L1_8X16 => (2, [PartPredMode::PredL1, PartPredMode::PredL1], 8, 16),
    MB_TYPE_B_L0_L1_16X8 => (2, [PartPredMode::PredL0, PartPredMode::PredL1], 16, 8),
    MB_TYPE_B_L0_L1_8X16 => (2, [PartPredMode::PredL0, PartPredMode::PredL1], 8, 16),
    MB_TYPE_B_L1_L0_16X8 => (2, [PartPredMode::PredL1, PartPredMode::PredL0], 16, 8),
    MB_TYPE_B_L1_L0_8X16 => (2, [PartPredMode::PredL1, PartPredMode::PredL0], 8, 16),
    MB_TYPE_B_L0_BI_16X8 => (2, [PartPredMode::PredL0, PartPredMode::BiPred], 16, 8),
    MB_TYPE_B_L0_BI_8X16 => (2, [PartPredMode::PredL0, PartPredMode::BiPred], 8, 16),
    MB_TYPE_B_L1_BI_16X8 => (2, [PartPredMode::PredL1, PartPredMode::BiPred], 16, 8),
    MB_TYPE_B_L1_BI_8X16 => (2, [PartPredMode::PredL1, PartPredMode::BiPred], 8, 16),
    MB_TYPE_B_BI_L0_16X8 => (2, [PartPredMode::BiPred, PartPredMode::PredL0], 16, 8),
    MB_TYPE_B_BI_L0_8X16 => (2, [PartPredMode::BiPred, PartPredMode::PredL0], 8, 16),
    MB_TYPE_B_BI_L1_16X8 => (2, [PartPredMode::BiPred, PartPredMode::PredL1], 16, 8),
    MB_TYPE_B_BI_L1_8X16 => (2, [PartPredMode::BiPred, PartPredMode::PredL1], 8, 16),
    MB_TYPE_B_BI_BI_16X8 => (2, [PartPredMode::BiPred, PartPredMode::BiPred], 16, 8),
    MB_TYPE_B_BI_BI_8X16 => (2, [PartPredMode::BiPred, PartPredMode::BiPred], 8, 16),
    MB_TYPE_B_8X8 => (4, [PartPredMode::NA, PartPredMode::NA], 8, 8),
    MB_TYPE_B_SKIP => (-1, [PartPredMode::Direct, PartPredMode::NA], 8, 8),
    n => panic!("Invalid inter mb_type {n}"),
  }
}
