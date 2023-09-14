use std::{
  array::TryFromSliceError,
  fmt,
  ops::{Deref, Index, IndexMut},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MathError {
  #[error(transparent)]
  InvalidBytes(#[from] TryFromSliceError),
}

type MathResult<T = ()> = Result<T, MathError>;

#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
  data: [f32; 9],
}

impl Default for Matrix3x3 {
  fn default() -> Self {
    Self::identity()
  }
}

impl Matrix3x3 {
  pub fn identity() -> Self {
    Self {
      data: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    }
  }

  pub fn from_bytes(bytes: &[u8]) -> MathResult<Self> {
    let mut matrix = [0_f32; 9];
    for (i, matrix_value) in matrix.iter_mut().enumerate() {
      let bytes_i = i * 4;
      let value = i32::from_be_bytes((&bytes[bytes_i..bytes_i + 4]).try_into()?) as f32;
      // | 0 1 2 |
      // | 3 4 5 |
      // | 6 7 8 |
      // All numbers are stored in native endianness, as 16.16 fixed-point values,
      // except for 2, 5 and 8, which are stored as 2.30 fixed-point values.
      *matrix_value = fixed_point_to_f32(value, if i == 2 || i == 5 || i == 8 { 30 } else { 16 });
    }
    Ok(Self { data: matrix })
  }

  /// Extract the rotation component of the transformation matrix and
  /// returns the angle (in degrees) by which the transformation rotates
  /// the frame clockwise. The angle will be in range `[-180.0, 180.0]`,
  /// and 0 if the matrix is singular
  ///
  /// # Arguments
  /// * `matrix` - The transformation matrix
  ///
  /// *Based on the implementation from
  /// [libavutil](https://ffmpeg.org/doxygen/trunk/display_8c_source.html#l00035)*
  pub fn rotation(&self) -> f32 {
    let mut scale = [0_f32; 2];

    scale[0] = f32::hypot(self.data[0], self.data[3]);
    scale[1] = f32::hypot(self.data[1], self.data[4]);

    if scale[0] == 0.0 || scale[1] == 0.0 {
      return 0.;
    }

    f32::atan2(self.data[1] / scale[1], self.data[0] / scale[0]) * 180_f32 / std::f32::consts::PI
  }
}

impl Index<(usize, usize)> for Matrix3x3 {
  type Output = f32;

  fn index(&self, index: (usize, usize)) -> &f32 {
    let (i, j) = index;
    &self.data[3 * i + j]
  }
}

impl IndexMut<(usize, usize)> for Matrix3x3 {
  fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
    let (i, j) = index;
    &mut self.data[3 * i + j]
  }
}

impl fmt::Display for Matrix3x3 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for i in 0..3 {
      write!(f, "|")?;
      for j in 0..3 {
        write!(f, "{:>8.1}", self[(i, j)])?;
      }
      writeln!(f, "|")?;
    }
    Ok(())
  }
}

impl Deref for Matrix3x3 {
  type Target = [f32; 9];

  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

pub fn fixed_point_to_f32(x: f32, n: u8) -> f32 {
  x / (1 << n) as f32
}
