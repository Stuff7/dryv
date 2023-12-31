use super::*;
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
