use super::*;

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
    } else if x < 0 && (y >= 0 && y < max_h) {
      Some(Self::A)
    } else if (x >= 0 && x < max_w) && y < 0 {
      Some(Self::B)
    } else if x > max_w - 1 && y < 0 {
      Some(Self::C)
    } else if (x >= 0 && x < max_w) && (y >= 0 && y < max_h) {
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
