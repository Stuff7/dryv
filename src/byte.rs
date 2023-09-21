use std::{array::TryFromSliceError, ops::Deref, rc::Rc};

#[derive(Clone, Copy)]
pub struct Str<const N: usize>(pub [u8; N]);

impl<const N: usize> Default for Str<N> {
  fn default() -> Self {
    Self([0; N])
  }
}

impl<const N: usize> Str<N> {
  pub fn as_str(&self) -> Box<str> {
    match std::str::from_utf8(&self.0) {
      Ok(s) => s.into(),
      Err(_) => self.0.map(|c| c as char).iter().collect::<String>().into(),
    }
  }
}

impl<const N: usize> From<Str<N>> for Rc<str> {
  fn from(value: Str<N>) -> Self {
    value.as_str().into()
  }
}

impl<const N: usize> TryFrom<&[u8]> for Str<N> {
  type Error = TryFromSliceError;
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    Ok(Self(slice.try_into()?))
  }
}

impl<const N: usize> TryFromSlice for Str<N> {
  const SIZE: usize = N;
  fn try_from_slice(slice: &[u8]) -> Result<Self, TryFromSliceError> {
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
    write!(f, "{}", self.as_str())
  }
}

impl<const N: usize> std::fmt::Debug for Str<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "b{:?}", self.as_str())
  }
}

pub fn padded_array_from_slice<const N: usize>(slice: &[u8]) -> [u8; N] {
  let mut array = [0u8; N];
  let len = std::cmp::min(slice.len(), N);
  array[..len].copy_from_slice(&slice[..len]);
  array
}

pub fn pack_bits(data: &[u8], bit_offset: usize, bit_size: usize) -> u128 {
  let mut value = u128::from_be_bytes(padded_array_from_slice(data));
  value <<= bit_offset;
  value >>= 128 - bit_size;

  value
}

pub fn remove_emulation_prevention_bytes(input: &[u8]) -> Box<[u8]> {
  input
    .iter()
    .copied()
    .enumerate()
    .filter_map(|(i, byte)| {
      if i > 1 && byte == 0x03 && input[i - 1] == 0x00 && input[i - 2] == 0x00 {
        None
      } else {
        Some(byte)
      }
    })
    .collect()
}

pub fn pascal_string(slice: &[u8]) -> Box<str> {
  if slice.is_empty() {
    return "".into();
  }

  let length = slice[0] as usize;
  if length + 1 > slice.len() {
    return "".into();
  }

  std::str::from_utf8(&slice[1..=length])
    .unwrap_or_default()
    .into()
}

pub fn c_string(slice: &[u8]) -> Box<str> {
  if slice.last().unwrap_or(&1) != &0 {
    return "".into();
  }

  std::str::from_utf8(&slice[..slice.len() - 1])
    .unwrap_or_default()
    .into()
}

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
