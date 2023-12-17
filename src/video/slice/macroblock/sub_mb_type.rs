use super::*;

#[derive(Debug, Clone, Copy)]
pub struct SubMbType {
  pub code: u8,
  pub num_sub_mb_part: usize,
  pub sub_mb_part_pred_mode: PartPredMode,
  pub sub_mb_part_width: u8,
  pub sub_mb_part_height: u8,
}

impl Deref for SubMbType {
  type Target = u8;

  fn deref(&self) -> &Self::Target {
    &self.code
  }
}

impl PartialEq for SubMbType {
  fn eq(&self, other: &Self) -> bool {
    self.code == other.code
  }
}

impl SubMbType {
  pub fn new(sub_mb_type: u8) -> Self {
    let (num_sub_mb_part, sub_mb_part_pred_mode, sub_mb_part_width, sub_mb_part_height) = sub_mb_type_fields(sub_mb_type);
    Self {
      code: sub_mb_type,
      num_sub_mb_part,
      sub_mb_part_pred_mode,
      sub_mb_part_width,
      sub_mb_part_height,
    }
  }

  pub fn none() -> Self {
    Self::new(u8::MAX)
  }

  pub const fn empty() -> Self {
    Self {
      code: 0,
      num_sub_mb_part: 0,
      sub_mb_part_pred_mode: PartPredMode::Intra4x4,
      sub_mb_part_width: 0,
      sub_mb_part_height: 0,
    }
  }

  pub fn name(&self) -> &str {
    name_sub_mb_type(**self)
  }

  pub fn is_b_direct8x8(&self) -> bool {
    **self == SUB_MB_TYPE_B_DIRECT_8X8
  }
}

/// Table 7-17 - Sub-macroblock types in P macroblocks
/// Table 7-18 - Sub-macroblock types in B macroblocks
/// Returns (NumSubMbPart, SubMbPartPredMode, SubMbPartWidth, SubMbPartHeight)
fn sub_mb_type_fields(sub_mb_type: u8) -> (usize, PartPredMode, u8, u8) {
  match sub_mb_type {
    SUB_MB_TYPE_P_L0_8X8 => (1, PartPredMode::PredL0, 8, 8),
    SUB_MB_TYPE_P_L0_8X4 => (2, PartPredMode::PredL0, 8, 4),
    SUB_MB_TYPE_P_L0_4X8 => (2, PartPredMode::PredL0, 4, 8),
    SUB_MB_TYPE_P_L0_4X4 => (4, PartPredMode::PredL0, 4, 4),
    SUB_MB_TYPE_B_DIRECT_8X8 => (4, PartPredMode::Direct, 4, 4),
    SUB_MB_TYPE_B_L0_8X8 => (1, PartPredMode::PredL0, 8, 8),
    SUB_MB_TYPE_B_L1_8X8 => (1, PartPredMode::PredL1, 8, 8),
    SUB_MB_TYPE_B_BI_8X8 => (1, PartPredMode::BiPred, 8, 8),
    SUB_MB_TYPE_B_L0_8X4 => (2, PartPredMode::PredL0, 8, 4),
    SUB_MB_TYPE_B_L0_4X8 => (2, PartPredMode::PredL0, 4, 8),
    SUB_MB_TYPE_B_L1_8X4 => (2, PartPredMode::PredL1, 8, 4),
    SUB_MB_TYPE_B_L1_4X8 => (2, PartPredMode::PredL1, 4, 8),
    SUB_MB_TYPE_B_BI_8X4 => (2, PartPredMode::BiPred, 8, 4),
    SUB_MB_TYPE_B_BI_4X8 => (2, PartPredMode::BiPred, 4, 8),
    SUB_MB_TYPE_B_L0_4X4 => (4, PartPredMode::PredL0, 4, 4),
    SUB_MB_TYPE_B_L1_4X4 => (4, PartPredMode::PredL1, 4, 4),
    SUB_MB_TYPE_B_BI_4X4 => (4, PartPredMode::BiPred, 4, 4),
    _ => (0, PartPredMode::NA, 0, 0),
  }
}
