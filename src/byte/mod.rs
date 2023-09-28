mod bit;
mod string;

pub use bit::*;
pub use string::*;

use std::array::TryFromSliceError;

pub trait FromSlice {
  fn from_slice(slice: &[u8]) -> Self;
}

impl FromSlice for u64 {
  fn from_slice(slice: &[u8]) -> Self {
    let mut result = 0;
    let size = slice.len();

    #[allow(clippy::needless_range_loop)]
    for i in 0..size {
      let shift = (size - i - 1) * 8;
      result |= u64::from(slice[i]) << shift;
    }

    result
  }
}

pub trait TryFromSlice: Sized {
  const SIZE: usize;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError>;
}

impl TryFromSlice for u32 {
  const SIZE: usize = 4;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
    Ok(u32::from_be_bytes(slice[..Self::SIZE].try_into()?))
  }
}

impl TryFromSlice for i32 {
  const SIZE: usize = 4;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
    Ok(i32::from_be_bytes(slice[..Self::SIZE].try_into()?))
  }
}

impl TryFromSlice for u16 {
  const SIZE: usize = 2;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
    Ok(u16::from_be_bytes(slice[..Self::SIZE].try_into()?))
  }
}

impl TryFromSlice for i16 {
  const SIZE: usize = 2;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
    Ok(i16::from_be_bytes(slice[..Self::SIZE].try_into()?))
  }
}

impl<const N: usize> TryFromSlice for [u8; N] {
  const SIZE: usize = N;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
    slice.try_into()
  }
}

pub fn padded_array_from_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
  let mut array = [0u8; N];
  let len = std::cmp::min(slice.len(), N);
  array[..len].copy_from_slice(&slice[..len]);
  array
}

pub trait LossyFrom<T> {
  fn lossy_from(value: T) -> Self;
}

impl LossyFrom<u128> for u8 {
  fn lossy_from(value: u128) -> Self {
    value as u8
  }
}

impl LossyFrom<u128> for u16 {
  fn lossy_from(value: u128) -> Self {
    value as u16
  }
}

impl LossyFrom<u128> for u32 {
  fn lossy_from(value: u128) -> Self {
    value as u32
  }
}

impl LossyFrom<u128> for u64 {
  fn lossy_from(value: u128) -> Self {
    value as u64
  }
}

impl LossyFrom<u128> for u128 {
  fn lossy_from(value: u128) -> Self {
    value
  }
}

impl LossyFrom<u128> for usize {
  fn lossy_from(value: u128) -> Self {
    value as usize
  }
}
