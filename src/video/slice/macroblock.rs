use super::consts::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MacroblockError {
  #[error("Macroblock index \"{0}\" outside bounds for macroblocks length \"{1}\"")]
  MacroblockBounds(isize, usize),
}

pub type MacroblockResult<T = ()> = Result<T, MacroblockError>;

#[derive(Debug)]
pub struct Macroblock {
  pub index: usize,
  pub mb_field_decoding_flag: bool,
  pub mb_type: u8,
  pub coded_block_pattern: u16,
  pub transform_size_8x8_flag: u16,
  pub mb_qp_delta: i16,
  pub pcm_sample_luma: [u16; 256],
  pub pcm_sample_chroma: [u16; 512],
  pub prev_intra4x4_pred_mode_flag: [u16; 16],
  pub rem_intra4x4_pred_mode: [u16; 16],
  pub prev_intra8x8_pred_mode_flag: [u16; 4],
  pub rem_intra8x8_pred_mode: [u16; 4],
  pub intra_chroma_pred_mode: u16,
  pub sub_mb_type: [u8; 4],
  pub ref_idx: [[u16; 4]; 2],
  pub mvd: [[[i16; 2]; 16]; 2],
  /// [0 luma, 1 cb, 2 cr][coeff]
  pub block_luma_dc: [[i16; 16]; 3],
  /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
  pub block_luma_ac: [[[i16; 15]; 16]; 3],
  /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
  pub block_luma_4x4: [[[i16; 16]; 16]; 3],
  /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
  pub block_luma_8x8: [[[i16; 64]; 4]; 3],
  /// [0 cb, 1 cr][coeff]
  pub block_chroma_dc: [[i16; 8]; 2],
  /// [0 cb, 1 cr][blkIdx][coeff]
  pub block_chroma_ac: [[[i16; 15]; 8]; 2],
  /// [0 luma, 1 cb, 2 cr][blkIdx]
  pub total_coeff: [[i16; 16]; 3],
  /// [0 luma, 1 cb, 2 cr][blkIdx], with blkIdx == 16 being DC
  pub coded_block_flag: [[i16; 17]; 3],
}

impl Macroblock {
  pub const fn empty(coded_block_flag: i16) -> Self {
    Self {
      index: 0,
      mb_field_decoding_flag: false,
      mb_type: MB_TYPE_UNAVAILABLE,
      coded_block_pattern: 0x0F,
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
      /// [0 luma, 1 cb, 2 cr][coeff]
      block_luma_dc: [[0; 16]; 3],
      /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
      block_luma_ac: [[[0; 15]; 16]; 3],
      /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
      block_luma_4x4: [[[0; 16]; 16]; 3],
      /// [0 luma, 1 cb, 2 cr][blkIdx][coeff]
      block_luma_8x8: [[[0; 64]; 4]; 3],
      /// [0 cb, 1 cr][coeff]
      block_chroma_dc: [[0; 8]; 2],
      /// [0 cb, 1 cr][blkIdx][coeff]
      block_chroma_ac: [[[0; 15]; 8]; 2],
      /// [0 luma, 1 cb, 2 cr][blkIdx]
      total_coeff: [[0; 16]; 3],
      /// [0 luma, 1 cb, 2 cr][blkIdx], with blkIdx == 16 being DC
      coded_block_flag: [
        [coded_block_flag; 17],
        [coded_block_flag; 17],
        [coded_block_flag; 17],
      ],
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

#[derive(Debug, Clone, Copy)]
pub enum MbPosition {
  This,
  /// Left
  A,
  /// Above
  B,
  /// Right and above
  C,
  /// Left and above
  D,
}
