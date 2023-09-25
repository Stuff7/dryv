use super::*;
use std::{
  num::TryFromIntError,
  ops::{BitOr, Deref, Shl, Sub},
};

pub type BitResult<T = ()> = Result<T, TryFromIntError>;

#[derive(Debug)]
pub struct BitData {
  data: Box<[u8]>,
  offset: usize,
  bit_offset: usize,
}

impl BitData {
  pub fn new(data: &[u8]) -> Self {
    Self {
      data: remove_emulation_prevention_bytes(data),
      offset: 0,
      bit_offset: 0,
    }
  }
}

impl Deref for BitData {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
    &self.data[self.offset..]
  }
}

impl BitData {
  pub fn skip_trailing_bits(&mut self) -> &mut Self {
    if self.bit_offset != 0 {
      self.bit_offset = 0;
      self.offset += 1;
    }
    self
  }

  pub fn byte(&mut self) -> BitResult<u8> {
    self.bits_into(8)
  }

  pub fn bit_flag(&mut self) -> bool {
    self.bit() != 0
  }

  pub fn bit(&mut self) -> u8 {
    let byte = self.data[self.offset];
    let bit = (byte >> (7 - self.bit_offset)) & 1;
    self.consume_bits(1);
    bit
  }

  pub fn bits(&mut self, n: u8) -> u8 {
    let byte = self.data[self.offset];
    let bit = byte << self.bit_offset >> (8 - n);
    self.consume_bits(n as usize);
    bit
  }

  pub fn bits_into<T: TryFrom<u128, Error = TryFromIntError>>(
    &mut self,
    bits: usize,
  ) -> BitResult<T> {
    let number = pack_bits(&self.data[self.offset..], self.bit_offset, bits);
    self.consume_bits(bits);
    number.try_into()
  }

  pub fn exponential_golomb<
    T: Shl<u8, Output = T> + BitOr<T, Output = T> + Sub<T, Output = T> + From<u8>,
  >(
    &mut self,
  ) -> T {
    let mut bits = BitIter::new(self.deref(), self.bit_offset);
    let k = bits.position(|bit| bit == 1).unwrap_or_default();
    let x = bits
      .take(k)
      .fold(T::from(1), |x, bit| x << 1 | T::from(bit));
    self.consume_bits(k + k + 1);
    x - T::from(1)
  }

  pub fn next_into<T: TryFrom<u128, Error = TryFromIntError> + Sized>(&mut self) -> BitResult<T> {
    self.bits_into(std::mem::size_of::<T>() * 8)
  }

  fn consume_bits(&mut self, bits: usize) {
    let read_bits = self.bit_offset + bits;
    self.bit_offset = read_bits % 8;
    self.offset += read_bits >> 3;
  }
}

pub struct BitIter<'a> {
  bytes: &'a [u8],
  current_byte_index: usize,
  current_bit_index: u8,
}

impl<'a> BitIter<'a> {
  pub fn new(bytes: &'a [u8], bit_offset: usize) -> Self {
    let current_byte_index = bit_offset / 8;
    let current_bit_index = (bit_offset % 8) as u8;

    Self {
      bytes,
      current_byte_index,
      current_bit_index,
    }
  }
}

impl<'a> Iterator for BitIter<'a> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_byte_index >= self.bytes.len() {
      return None;
    }

    let current_byte = self.bytes[self.current_byte_index];
    let bit_value = (current_byte >> (7 - self.current_bit_index)) & 1;

    self.current_bit_index += 1;
    if self.current_bit_index >= 8 {
      self.current_byte_index += 1;
      self.current_bit_index = 0;
    }

    Some(bit_value)
  }
}

pub fn pack_bits(data: &[u8], bit_offset: usize, bit_size: usize) -> u128 {
  let mut value = u128::from_be_bytes(padded_array_from_slice(data));
  value <<= bit_offset;
  value >>= 128 - bit_size;

  value
}
