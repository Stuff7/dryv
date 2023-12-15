use super::*;
use crate::display::{display_array1d, display_array3d, DisplayArray};

impl Display for Macroblock {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("mb_type: {}\n", self.mb_type.name()))?;
    f.write_str(&format!("mode: {:?} | {:?}\n", self.mb_type.mode(), self.mb_type.inter_mode(1)))?;
    f.write_str(&format!("NumMbPart: {}\n", self.mb_type.num_mb_part() as u8))?;
    f.write_str(&format!("MbPartWidth: {}\n", self.mb_type.mb_part_width()))?;
    f.write_str(&format!("MbPartHeight: {}\n\n", self.mb_type.mb_part_height()))?;

    if self.mb_type.is_inter() && self.mb_type.is_submb() {
      for (i, sub_mb_type) in self.sub_mb_type.iter().enumerate() {
        f.write_str(&format!("# sub_mb_type[{i}] {}\n", sub_mb_type.name()))?;
        f.write_str(&format!("NumSubMbPart: {}\n", sub_mb_type.num_sub_mb_part))?;
        f.write_str(&format!("subMode: {:?}\n", sub_mb_type.sub_mb_part_pred_mode))?;
        f.write_str(&format!("SubMbPartWidth: {}\n", sub_mb_type.sub_mb_part_width))?;
        f.write_str(&format!("SubMbPartHeight: {}\n\n", sub_mb_type.sub_mb_part_height))?;
      }
    }

    f.write_str(&display_array1d("predFlagL0", &self.pred_flagl0))?;
    f.write_str(&display_array1d("predFlagL1", &self.pred_flagl1))?;
    if self.mb_type.is_pcm() {
      f.write_str(&format!("pcm_sample_chroma:{:?}\n", DisplayArray(&self.pcm_sample_chroma)))?;
    }
    f.write_str(&format!("transform_size_8x8_flag: {}\n", self.transform_size_8x8_flag,))?;
    f.write_str(&format!("coded_block_pattern: {}\n", self.coded_block_pattern))?;
    if !self.mb_type.is_inter() {
      f.write_str(&format!("CodedBlockPatternLuma: {}\n", self.mb_type.coded_block_pattern_luma(),))?;
      f.write_str(&format!("CodedBlockPatternChroma: {}\n", self.mb_type.coded_block_pattern_chroma(),))?;
    }
    f.write_str(&format!("mb_qp_delta: {}\n\n", self.mb_qp_delta))?;

    f.write_str(&format!("QPY: {}\n", self.qpy))?;
    f.write_str(&format!("QP1Y: {}\n", self.qp1y))?;
    f.write_str(&format!("QP1C: {}\n", self.qp1c))?;
    f.write_str(&format!("QPC: {}\n", self.qpc))?;
    f.write_str(&format!("QSY: {}\n", self.qsy))?;
    f.write_str(&format!("QSC: {}\n\n", self.qsc))?;

    f.write_str(&format!("intra_chroma_pred_mode: {}\n", self.intra_chroma_pred_mode,))?;
    f.write_str(&format!("TransformBypassModeFlag: {}\n", self.transform_bypass_mode_flag as u8,))?;
    f.write_str(&format!("Intra16x16PredMode: {}\n", self.mb_type.intra16x16_pred_mode()))?;
    f.write_str(&format!("mb_skip_flag: {}\n\n", self.mb_skip_flag as u8))?;

    // f.write_str(&display_array1d("refIdxL0", &self.ref_idxl0))?;
    // f.write_str(&display_array1d("refIdxL1", &self.ref_idxl1))?;

    f.write_str(&display_array3d("mv_l0", &self.mv_l0))?;
    f.write_str(&display_array3d("mv_l1", &self.mv_l1))?;
    f.write_str("\n")?;

    for i in 0..4 {
      for j in 0..4 {
        f.write_str(&format!("mvd_l0[{i}][{j}]:"))?;
        for z in 0..2 {
          f.write_str(&format!(
            " {}",
            if j > 0 || i >= self.num_mb_part() { 0 } else { self.mvd_lx(0, i, j, z) }
          ))?;
        }
        f.write_str("\n")?;
      }
      f.write_str("\n")?;
    }
    for i in 0..4 {
      for j in 0..4 {
        f.write_str(&format!("mvd_l1[{i}][{j}]:"))?;
        for z in 0..2 {
          f.write_str(&format!(
            " {}",
            if j > 0 || i >= self.num_mb_part() { 0 } else { self.mvd_lx(1, i, j, z) }
          ))?;
        }
        f.write_str("\n")?;
      }
      f.write_str("\n")?;
    }

    match self.mb_type.mode() {
      PartPredMode::Intra4x4 => {
        for i in 0..16 {
          for j in 0..4 {
            f.write_str(&format!("lumaPredSamples[{i}][{j}]:{:?}\n", DisplayArray(&self.luma_pred_samples[i][j])))?;
          }
          f.write_str("\n")?;
        }
      }
      PartPredMode::Intra8x8 => {
        for i in 0..4 {
          for j in 0..8 {
            f.write_str(&format!(
              "luma8x8PredSamples[{i}][{j}]:{:?}\n",
              DisplayArray(&self.luma8x8_pred_samples[i][j])
            ))?;
          }
          f.write_str("\n")?;
        }
      }
      PartPredMode::Intra16x16 => {
        for i in 0..16 {
          f.write_str(&format!(
            "luma16x16PredSamples[{i}]:{:?}\n",
            DisplayArray(&self.luma16x16_pred_samples[i])
          ))?;
        }
        f.write_str("\n")?;
      }
      _ => (),
    }
    for i in 0..8 {
      f.write_str(&format!("chromaPredSamples[{i}]:{:?}\n", DisplayArray(&self.chroma_pred_samples[i])))?;
    }
    Ok(())
  }
}

impl std::fmt::Debug for Macroblock {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut f = f.debug_struct("Macroblock");
    f.field("mb_field_decoding_flag", &self.mb_field_decoding_flag)
      .field(&format!("mb_type [{}]", self.mb_type.name()), &self.mb_type)
      .field("transform_size_8x8_flag", &self.transform_size_8x8_flag)
      .field("coded_block_pattern", &self.coded_block_pattern)
      .field("prev_intra8x8_pred_mode_flag", &DisplayArray(&self.prev_intra8x8_pred_mode_flag))
      .field("rem_intra8x8_pred_mode", &DisplayArray(&self.rem_intra8x8_pred_mode))
      .field("intra_chroma_pred_mode", &self.intra_chroma_pred_mode)
      .field("mb_qp_delta", &self.mb_qp_delta)
      .field("mb_skip_flag", &self.mb_skip_flag)
      .field("qpy", &self.qpy)
      .field("qp1y", &self.qp1y)
      .field("qp1c", &self.qp1c)
      .field("qpc", &self.qpc)
      .field("qsy", &self.qsy)
      .field("qsc", &self.qsc)
      .field("transform_bypass_mode_flag", &self.transform_bypass_mode_flag)
      .field("intra4x4_pred_mode", &DisplayArray(&self.intra4x4_pred_mode))
      .field("intra8x8_pred_mode", &DisplayArray(&self.intra8x8_pred_mode));

    match self.mb_type.mode() {
      PartPredMode::Intra4x4 => {
        for i in 0..16 {
          for j in 0..4 {
            f.field(&format!("luma_pred_samples[{i}][{j}]"), &DisplayArray(&self.luma_pred_samples[i][j]));
          }
        }
      }
      PartPredMode::Intra8x8 => {
        for i in 0..4 {
          for j in 0..8 {
            f.field(
              &format!("luma8x8_pred_samples[{i}][{j}]"),
              &DisplayArray(&self.luma8x8_pred_samples[i][j]),
            );
          }
        }
      }
      PartPredMode::Intra16x16 => {
        for i in 0..16 {
          f.field(&format!("luma16x16_pred_samples[{i}]"), &DisplayArray(&self.luma16x16_pred_samples[i]));
        }
      }
      _ => (),
    }
    for i in 0..8 {
      f.field(&format!("chroma_pred_samples[{i}]"), &DisplayArray(&self.chroma_pred_samples[i]));
    }

    const BLOCK_NAME: [&str; 3] = ["Luma", "Cb", "Cr"];
    if self.mb_type.is_intra_16x16() {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        f.field(&format!("{} DC", name), &DisplayArray(&self.block_luma_dc[i]));
        for j in 0..16 {
          f.field(&format!("{} AC {j}", name), &DisplayArray(&self.block_luma_ac[i][j]));
        }
      }
    } else if self.transform_size_8x8_flag != 0 {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        for j in 0..4 {
          f.field(&format!("{} 8x8 {j}", name), &DisplayArray(&self.block_luma_8x8[i][j]));
        }
      }
    } else {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        for j in 0..16 {
          f.field(&format!("{} 4x4 {j}", name), &DisplayArray(&self.block_luma_4x4[i][j]));
        }
      }
    }
    for i in 0..2 {
      f.field(&format!("{} DC", BLOCK_NAME[i + 1]), &DisplayArray(&self.block_chroma_dc[i]));
      for j in 0..8 {
        f.field(&format!("{} AC {j}", BLOCK_NAME[i + 1]), &DisplayArray(&self.block_chroma_ac[i][j]));
      }
    }
    f.field("pcm_sample_luma", &DisplayArray(&self.pcm_sample_luma))
      .field("pcm_sample_chroma", &DisplayArray(&self.pcm_sample_chroma))
      .field("prev_intra4x4_pred_mode_flag", &DisplayArray(&self.prev_intra4x4_pred_mode_flag))
      .field("rem_intra4x4_pred_mode", &DisplayArray(&self.rem_intra4x4_pred_mode));
    let sub_mb_type_names: [&str; 4] = std::array::from_fn(|i| self.sub_mb_type[i].name());
    f.field(&format!("sub_mb_type {:?}", sub_mb_type_names), &self.sub_mb_type)
      .field("ref_idx_l0", &DisplayArray(&self.ref_idx[0]))
      .field("ref_idx_l1", &DisplayArray(&self.ref_idx[1]));

    for i in 0..2 {
      f.field(&format!("ref_idx_l{i}"), &DisplayArray(&self.ref_idx[i]));
      for k in 0..2 {
        let s: [isize; 16] = std::array::from_fn(|j| self.mvd[i][j][k]);
        f.field(&format!("mvd_l{i}[...][{k}]"), &DisplayArray(&s));
      }
    }

    for i in 0..3 {
      f.field(&format!("total_coeff[{i}]"), &DisplayArray(&self.total_coeff[i]))
        .field(&format!("coded_block_flag[{i}]"), &DisplayArray(&self.coded_block_flag[i]));
    }
    f.finish()
  }
}
