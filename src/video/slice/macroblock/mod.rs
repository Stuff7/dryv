mod display;
mod impls;
mod mb_type;
mod part_pred_mode;
mod position;
mod sub_mb_type;

pub use display::*;
pub use impls::*;
pub use mb_type::*;
pub use part_pred_mode::*;
pub use position::*;
pub use sub_mb_type::*;

use super::consts::*;
use std::{
  cmp::Ordering,
  fmt::{Debug, Display},
  hash::{Hash, Hasher},
  ops::Deref,
};
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
  pub mb_type: MbType,

  /// Coded block pattern, indicating which 4x4 blocks within the macroblock are coded.
  pub coded_block_pattern: u8,

  /// Flag indicating the use of 8x8 transform size within the macroblock.
  pub transform_size_8x8_flag: u8,

  /// Quantization parameter delta for the macroblock.
  /// It represents the adjustment to the quantization step size.
  pub mb_qp_delta: isize,

  pub qpy: isize,

  pub qp1y: isize,

  pub qp1c: isize,

  pub qpc: isize,

  pub qsy: isize,

  pub qsc: isize,

  pub transform_bypass_mode_flag: bool,

  pub intra4x4_pred_mode: [isize; 16],

  pub intra8x8_pred_mode: [isize; 4],

  pub luma_pred_samples: [[[isize; 4]; 4]; 16],

  pub luma16x16_pred_samples: [[isize; 16]; 16],

  pub luma8x8_pred_samples: [[[isize; 8]; 8]; 4],

  pub chroma_pred_samples: [[isize; 16]; 8],

  pub mv_l0: [[[isize; 2]; 4]; 4],

  pub mv_l1: [[[isize; 2]; 4]; 4],

  pub ref_idxl0: [isize; 4],

  pub ref_idxl1: [isize; 4],

  pub pred_flagl0: [isize; 4],

  pub pred_flagl1: [isize; 4],

  pub mb_skip_flag: bool,

  pub transform_bypass_flag: bool,
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
  pub sub_mb_type: [SubMbType; 4],

  /// Reference indices for motion compensation in two reference lists.
  pub ref_idx: [[u8; 4]; 2],

  /// Motion vector differences (MVD) for each 4x4 block within the macroblock.
  pub mvd: [[[isize; 2]; 16]; 2],

  /// DC coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components.
  pub block_luma_dc: [[isize; 16]; 3],

  /// AC coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_luma_ac: [[[isize; 15]; 16]; 3],

  /// Coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_luma_4x4: [[[isize; 16]; 16]; 3],

  /// Coefficients for luma (Y), chroma blue (Cb), and chroma red (Cr) components
  /// for each 8x8 block within the macroblock.
  pub block_luma_8x8: [[[isize; 64]; 4]; 3],

  /// DC coefficients for chroma blue (Cb) and chroma red (Cr) components.
  pub block_chroma_dc: [[isize; 8]; 2],

  /// AC coefficients for chroma blue (Cb) and chroma red (Cr) components
  /// for each 4x4 block within the macroblock.
  pub block_chroma_ac: [[[isize; 15]; 8]; 2],

  /// Total coefficients for each 4x4 block within the macroblock
  /// for luma (Y), chroma blue (Cb), and chroma red (Cr) components.
  pub total_coeff: [[isize; 16]; 3],

  /// Coded block flags for each 4x4 block within the macroblock.
  /// Indicates whether each block is coded or not (1 for coded, 0 for not coded).
  /// The last element with index 16 represents DC coefficients.
  pub coded_block_flag: [[u8; 17]; 3],
}
