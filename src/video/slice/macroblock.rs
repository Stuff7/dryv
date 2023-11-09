use std::{
  fmt::Debug,
  hash::{Hash, Hasher},
  ops::Deref,
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

  pub qsc: isize,

  pub transform_bypass_mode_flag: bool,

  pub intra4x4_pred_mode: [isize; 16],

  pub intra8x8_pred_mode: [isize; 4],

  pub luma_pred_samples: [[[isize; 4]; 4]; 16],

  pub luma16x16_pred_samples: [[isize; 16]; 16],

  pub luma8x8_pred_samples: [[[isize; 8]; 8]; 4],

  pub chroma_pred_samples: [[isize; 16]; 8],

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
  pub sub_mb_type: [u8; 4],

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

impl Macroblock {
  pub fn set_mb_type(&mut self, mb_type: u8) {
    self.mb_type = MbType::new(mb_type, self.transform_size_8x8_flag != 0);
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
      qsc: 0,
      intra4x4_pred_mode: [0; 16],
      intra8x8_pred_mode: [0; 4],
      luma_pred_samples: [[[0; 4]; 4]; 16],
      luma16x16_pred_samples: [[0; 16]; 16],
      luma8x8_pred_samples: [[[0; 8]; 8]; 4],
      chroma_pred_samples: [[0; 16]; 8],
      transform_bypass_mode_flag: false,
      transform_bypass_flag: false,
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
      mb_type: MbType::Unavailable,
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

  pub fn index(&self, macroblocks: &[Macroblock]) -> isize {
    unsafe { (self as *const Macroblock).offset_from(macroblocks.as_ptr()) }
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
    *self.mb_type == *other.mb_type
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
      .field(&format!("mb_type [{}]", self.mb_type.name()), &self.mb_type)
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
      .field("mb_qp_delta", &self.mb_qp_delta)
      .field("qpy", &self.qpy)
      .field("qp1y", &self.qp1y)
      .field("qp1c", &self.qp1c)
      .field("qpc", &self.qpc)
      .field("qsc", &self.qsc)
      .field(
        "transform_bypass_mode_flag",
        &self.transform_bypass_mode_flag,
      )
      .field(
        "intra4x4_pred_mode",
        &DisplayArray(&self.intra4x4_pred_mode),
      )
      .field(
        "intra8x8_pred_mode",
        &DisplayArray(&self.intra8x8_pred_mode),
      );

    match self.mb_type.mode() {
      PartPredMode::Intra4x4 => {
        for i in 0..16 {
          for j in 0..4 {
            f.field(
              &format!("luma_pred_samples[{i}][{j}]"),
              &DisplayArray(&self.luma_pred_samples[i][j]),
            );
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
          f.field(
            &format!("luma16x16_pred_samples[{i}]"),
            &DisplayArray(&self.luma16x16_pred_samples[i]),
          );
        }
      }
      _ => (),
    }
    for i in 0..8 {
      f.field(
        &format!("chroma_pred_samples[{i}]"),
        &DisplayArray(&self.chroma_pred_samples[i]),
      );
    }

    const BLOCK_NAME: [&str; 3] = ["Luma", "Cb", "Cr"];
    if self.mb_type.is_intra_16x16() {
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

impl MbPosition {
  pub fn from_coords(x: isize, y: isize, max_w: isize, max_h: isize) -> Option<Self> {
    if x < 0 && y < 0 {
      Some(Self::D)
    } else if x < 0 && (y >= 0 && y <= max_h - 1) {
      Some(Self::A)
    } else if (x >= 0 && x <= max_w - 1) && y < 0 {
      Some(Self::B)
    } else if x > max_w - 1 && y < 0 {
      Some(Self::C)
    } else if (x >= 0 && x <= max_w - 1) && (y >= 0 && y <= max_h - 1) {
      Some(Self::This)
    } else {
      None
    }
  }

  pub fn coords(x: isize, y: isize, max_w: isize, max_h: isize) -> (isize, isize) {
    ((x + max_w) % max_w, (y + max_h) % max_h)
  }

  pub fn blk_idx4x4(x: isize, y: isize, max_w: isize, max_h: isize) -> isize {
    let (x, y) = Self::coords(x, y, max_w, max_h);
    8 * (y / 8) + 4 * (x / 8) + 2 * ((y % 8) / 4) + ((x % 8) / 4)
  }

  pub fn blk_idx8x8(x: isize, y: isize, max_w: isize, max_h: isize) -> isize {
    let (x, y) = Self::coords(x, y, max_w, max_h);
    2 * (y / 8) + (x / 8)
  }
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
    if !mb_t.mb_field_decoding_flag && mb_p.mb_field_decoding_flag && !mb_p.mb_type.is_available() {
      Self::FrameFromField
    } else if mb_t.mb_field_decoding_flag
      && !mb_p.mb_field_decoding_flag
      && !mb_p.mb_type.is_available()
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
    if mb_type < MB_TYPE_I_PCM {
      let (part_pred_mode, intra_pred_mode, coded_block_pattern_chroma, coded_block_pattern_luma) =
        mb_type_intra(mb_type, transform_size_8x8_flag);
      MbType::Intra {
        code: mb_type,
        intra_pred_mode,
        part_pred_mode,
        coded_block_pattern_luma,
        coded_block_pattern_chroma,
      }
    } else if mb_type == MB_TYPE_I_PCM {
      MbType::Pcm
    } else {
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

  pub fn mode_idx(&self, idx: usize) -> &PartPredMode {
    match self {
      Self::Intra { part_pred_mode, .. } => part_pred_mode,
      Self::Inter { part_pred_mode, .. } => &part_pred_mode[idx],
      _ => &PartPredMode::NA,
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

  pub fn is_pcm(&self) -> bool {
    matches!(self, Self::Pcm)
  }

  pub fn is_p_8x8ref0(&self) -> bool {
    **self == MB_TYPE_P_8X8REF0
  }

  pub fn is_skip(&self) -> bool {
    matches!(**self, MB_TYPE_P_SKIP | MB_TYPE_B_SKIP)
  }

  pub fn is_submb(&self) -> bool {
    matches!(**self, MB_TYPE_P_8X8 | MB_TYPE_P_8X8REF0 | MB_TYPE_B_8X8)
  }
}

#[derive(Debug)]
pub struct SubMbType {
  code: u8,
  sub_num_mb_part: u8,
  sub_part_pred_mode: PartPredMode,
  sub_part_width: u8,
  sub_part_height: u8,
}

impl SubMbType {
  pub fn new(sub_mb_type: u8) -> Self {
    let (sub_num_mb_part, sub_part_pred_mode, sub_part_width, sub_part_height) =
      sub_mb_type_fields(sub_mb_type);
    Self {
      code: sub_mb_type,
      sub_num_mb_part,
      sub_part_pred_mode,
      sub_part_width,
      sub_part_height,
    }
  }
}

#[derive(Debug)]
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
fn mb_type_intra(mb_type: u8, transform_size_8x8_flag: bool) -> (PartPredMode, u8, u8, u8) {
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
fn mb_type_inter(mb_type: u8) -> (i8, [PartPredMode; 2], u8, u8) {
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

/// Table 7-17 - Sub-macroblock types in P macroblocks
/// Table 7-18 - Sub-macroblock types in B macroblocks
/// Returns (NumSubMbPart, SubMbPartPredMode, SubMbPartWidth, SubMbPartHeight)
fn sub_mb_type_fields(sub_mb_type: u8) -> (u8, PartPredMode, u8, u8) {
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
    n => panic!("Invalid sub_mb_type {n}"),
  }
}
