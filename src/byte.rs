use std::{array::TryFromSliceError, ops::Deref};

#[derive(Clone, Copy)]
pub struct Str<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Str<N> {
  fn default() -> Self {
    Self([0; N])
  }
}

impl<const N: usize> Str<N> {
  pub fn as_string(&self) -> String {
    self.0.map(|c| c as char).iter().collect()
  }
}

impl<const N: usize> TryFrom<&[u8]> for Str<N> {
  type Error = TryFromSliceError;
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    Ok(Self(slice.try_into()?))
  }
}

impl<const N: usize> Deref for Str<N> {
  type Target = [u8; N];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<const N: usize> std::fmt::Display for Str<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.as_string())
  }
}

impl<const N: usize> std::fmt::Debug for Str<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "b{:?}", self.as_string())
  }
}

pub fn from_be_slice(bytes: &[u8], size: usize) -> u64 {
  let mut result = 0;

  #[allow(clippy::needless_range_loop)]
  for i in 0..size {
    let shift = (size - i - 1) * 8;
    result |= u64::from(bytes[i]) << shift;
  }

  result
}
