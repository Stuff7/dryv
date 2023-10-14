use std::{
  fmt::Debug,
  hash::{Hash, Hasher},
};

use crate::display::DisplayArray;

use super::consts::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MacroblockError {
  #[error("Macroblock index \"{0}\" outside bounds for macroblocks length \"{1}\"")]
  MacroblockBounds(isize, usize),
}

pub type MacroblockResult<T = ()> = Result<T, MacroblockError>;

pub struct Macroblock {
  /// Flag indicating if macroblock field decoding is used.
  /// Field decoding divides a frame into fields for interlaced video.
  pub mb_field_decoding_flag: bool,

  /// Macroblock type, which defines the coding mode for the macroblock.
  /// The type determines how the macroblock is predicted and encoded.
  pub mb_type: u8,

  /// Coded block pattern, indicating which 4x4 blocks within the macroblock are coded.
  pub coded_block_pattern: u8,

  /// Flag indicating the use of 8x8 transform size within the macroblock.
  pub transform_size_8x8_flag: u8,

  /// Quantization parameter delta for the macroblock.
  /// It represents the adjustment to the quantization step size.
  pub mb_qp_delta: i16,

  /// PCM (Pulse Code Modulation) samples for luma (Y) component.
  /// PCM samples provide raw pixel values for luma.
  pub pcm_sample_luma: [u8; 256],

  /// PCM samples for chroma (Cb and Cr) components.
  /// PCM samples provide raw pixel values for chroma.
  pub pcm_sample_chroma: [u8; 512],

  /// Flags indicating previous intra 4x4 prediction mode for each 4x4 block within the macroblock.
  pub prev_intra4x4_pred_mode_flag: [u8; 16],

  /// Remaining intra 4x4 prediction modes for each 4x4 block within the macroblock.
  pub rem_intra4x4_pred_mode: [u8; 16],

  /// Flags indicating previous intra 8x8 prediction mode for each 8x8 block within the macroblock.
  pub prev_intra8x8_pred_mode_flag: [u8; 4],

  /// Remaining intra 8x8 prediction modes for each 8x8 block within the macroblock.
  pub rem_intra8x8_pred_mode: [u8; 4],

  /// Intra chroma prediction mode for chroma (Cb and Cr) components within the macroblock.
  pub intra_chroma_pred_mode: u8,

  /// Sub-macroblock types for each of the 4 sub-macroblocks (if applicable).
  pub sub_mb_type: [u8; 4],

  /// Reference indices for motion compensation in two reference lists.
  pub ref_idx: [[u8; 4]; 2],

  /// Motion vector differences (MVD) for each 4x4 block within the macroblock.
  pub mvd: [[[i16; 2]; 16]; 2],

  /// DC coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components.
  pub block_luma_dc: [[i16; 16]; 3],

  /// AC coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_luma_ac: [[[i16; 15]; 16]; 3],

  /// Coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_luma_4x4: [[[i16; 16]; 16]; 3],

  /// Coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 8x8 block within the macroblock.
  pub block_luma_8x8: [[[i16; 64]; 4]; 3],

  /// DC coefficients for chroma blue (Cb) and chroma red (Cr) components.
  pub block_chroma_dc: [[i16; 8]; 2],

  /// AC coefficients for chroma blue (Cb) and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_chroma_ac: [[[i16; 15]; 8]; 2],

  /// Total coefficients for each 4x4 block within the macroblock
  /// for luma (Y), chroma blue (Cb), and chroma red (Cr) components.
  pub total_coeff: [[i16; 16]; 3],

  /// Coded block flags for each 4x4 block within the macroblock.
  /// Indicates whether each block is coded or not (1 for coded, 0 for not coded).
  /// The last element with index 16 represents DC coefficients.
  pub coded_block_flag: [[u8; 17]; 3],
}

impl Macroblock {
  pub const fn empty() -> Self {
    Self {
      mb_field_decoding_flag: false,
      mb_type: 0,
      coded_block_pattern: 0,
      transform_size_8x8_flag: 0,
      mb_qp_delta: 0,
      pcm_sample_luma: [0; 256],
      pcm_sample_chroma: [0; 512],
      prev_intra4x4_pred_mode_flag: [0; 16],
      rem_intra4x4_pred_mode: [0; 16],
      prev_intra8x8_pred_mode_flag: [0; 4],
      rem_intra8x8_pred_mode: [0; 4],
      intra_chroma_pred_mode: 0,
      sub_mb_type: [0; 4],
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
      mb_type: MB_TYPE_UNAVAILABLE,
      coded_block_pattern: 0x0F,
      coded_block_flag: [
        [coded_block_flag; 17],
        [coded_block_flag; 17],
        [coded_block_flag; 17],
      ],
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

  pub fn offset<'a>(
    &'a self,
    macroblocks: &'a [Macroblock],
    offset: isize,
  ) -> MacroblockResult<&'a Self> {
    let index = unsafe { (self as *const Macroblock).offset_from(macroblocks.as_ptr()) } + offset;
    if index > macroblocks.len() as isize || index < 0 {
      return Err(MacroblockError::MacroblockBounds(index, macroblocks.len()));
    }
    Ok(&macroblocks[index as usize])
  }

  #[allow(unused)]
  pub fn offset_mut<'a>(
    &'a self,
    macroblocks: &'a mut [Macroblock],
    offset: isize,
  ) -> MacroblockResult<&'a mut Self> {
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
    self.mb_type == other.mb_type
  }
}

impl Hash for Macroblock {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.mb_type.hash(state);
  }
}

impl std::fmt::Debug for Macroblock {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut f = f.debug_struct("Macroblock");
    f.field("mb_field_decoding_flag", &self.mb_field_decoding_flag)
      .field(
        &format!("mb_type [{}]", name_mb_type(self.mb_type)),
        &self.mb_type,
      )
      .field("transform_size_8x8_flag", &self.transform_size_8x8_flag)
      .field("coded_block_pattern", &self.coded_block_pattern)
      .field(
        "prev_intra8x8_pred_mode_flag",
        &DisplayArray(&self.prev_intra8x8_pred_mode_flag),
      )
      .field(
        "rem_intra8x8_pred_mode",
        &DisplayArray(&self.rem_intra8x8_pred_mode),
      )
      .field("intra_chroma_pred_mode", &self.intra_chroma_pred_mode)
      .field("mb_qp_delta", &self.mb_qp_delta);

    const BLOCK_NAME: [&str; 3] = ["Luma", "Cb", "Cr"];
    if is_intra_16x16_mb_type(self.mb_type) {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        f.field(
          &format!("{} DC", name),
          &DisplayArray(&self.block_luma_dc[i]),
        );
        for j in 0..16 {
          f.field(
            &format!("{} AC {j}", name),
            &DisplayArray(&self.block_luma_ac[i][j]),
          );
        }
      }
    } else if self.transform_size_8x8_flag != 0 {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        for j in 0..4 {
          f.field(
            &format!("{} 8x8 {j}", name),
            &DisplayArray(&self.block_luma_8x8[i][j]),
          );
        }
      }
    } else {
      for (i, name) in BLOCK_NAME.iter().enumerate() {
        for j in 0..16 {
          f.field(
            &format!("{} 4x4 {j}", name),
            &DisplayArray(&self.block_luma_4x4[i][j]),
          );
        }
      }
    }
    for i in 0..2 {
      f.field(
        &format!("{} DC", BLOCK_NAME[i + 1]),
        &DisplayArray(&self.block_chroma_dc[i]),
      );
      for j in 0..8 {
        f.field(
          &format!("{} AC {j}", BLOCK_NAME[i + 1]),
          &DisplayArray(&self.block_chroma_ac[i][j]),
        );
      }
    }
    f.field("pcm_sample_luma", &DisplayArray(&self.pcm_sample_luma))
      .field("pcm_sample_chroma", &DisplayArray(&self.pcm_sample_chroma))
      .field(
        "prev_intra4x4_pred_mode_flag",
        &DisplayArray(&self.prev_intra4x4_pred_mode_flag),
      )
      .field(
        "rem_intra4x4_pred_mode",
        &DisplayArray(&self.rem_intra4x4_pred_mode),
      )
      .field(
        &format!("sub_mb_type {:?}", self.sub_mb_type.map(name_sub_mb_type)),
        &DisplayArray(&self.sub_mb_type),
      )
      .field("ref_idx_l0", &DisplayArray(&self.ref_idx[0]))
      .field("ref_idx_l1", &DisplayArray(&self.ref_idx[1]));

    for i in 0..2 {
      f.field(&format!("ref_idx_l{i}"), &DisplayArray(&self.ref_idx[i]));
      for j in 0..16 {
        f.field(&format!("mvd_l{i}[{j}]"), &DisplayArray(&self.mvd[i][j]));
      }
    }

    for i in 0..3 {
      f.field(
        &format!("total_coeff[{i}]"),
        &DisplayArray(&self.total_coeff[i]),
      )
      .field(
        &format!("coded_block_flag[{i}]"),
        &DisplayArray(&self.coded_block_flag[i]),
      );
    }
    f.finish()
  }
}

#[allow(unused)]
/// Represents the position of a macroblock (MB) within a block grid.
#[derive(Debug, Clone, Copy)]
pub enum MbPosition {
  /// The macroblock itself (current position).
  This,
  /// Position A: Refers to the macroblock to the left of the current macroblock.
  A,
  /// Position B: Refers to the macroblock above the current macroblock.
  B,
  /// Position C: Refers to the macroblock diagonally above and to the right of the current macroblock.
  C,
  /// Position D: Refers to the macroblock diagonally above and to the left of the current macroblock.
  D,
}

/// Represents the mode of a macroblock (MB) in the context of interlaced video coding.
#[derive(Debug, Clone, Copy)]
pub enum MbMode {
  /// Same mode: Indicates that the macroblock is in the same mode as the previous macroblock.
  Same,
  /// Frame from Field mode: Indicates that the macroblock constructs a frame from two fields.
  FrameFromField,
  /// Field from Frame mode: Indicates that the macroblock constructs two fields from a frame.
  FieldFromFrame,
}

impl MbMode {
  pub fn new(mb_t: &Macroblock, mb_p: &Macroblock) -> Self {
    if !mb_t.mb_field_decoding_flag
      && mb_p.mb_field_decoding_flag
      && mb_p.mb_type != MB_TYPE_UNAVAILABLE
    {
      Self::FrameFromField
    } else if mb_t.mb_field_decoding_flag
      && !mb_p.mb_field_decoding_flag
      && mb_p.mb_type != MB_TYPE_UNAVAILABLE
    {
      Self::FieldFromFrame
    } else {
      Self::Same
    }
  }
}

/// Represents the size of a block within a macroblock (MB).
#[derive(Debug, Clone, Copy)]
pub enum BlockSize {
  /// 4x4 block size: Used for luma (brightness) macroblocks.
  B4x4,
  /// Chroma block size: Used for chroma (color) macroblocks.
  Chroma,
  /// 8x8 block size: Typically used for luminance (Y) macroblocks in certain macroblock configurations.
  B8x8,
}

impl BlockSize {
  /// Checks if the block size represents a chroma (color) block.
  pub fn is_chroma(&self) -> bool {
    matches!(self, BlockSize::Chroma)
  }
}
