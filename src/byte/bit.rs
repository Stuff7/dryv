use super::*;
use std::ops::{Add, BitAnd, BitOr, Deref, Neg, Shl, Shr, Sub};

const EPB: [u8; 3] = [0x00, 0x00, 0x03];

#[derive(Debug)]
pub struct BitData {
  data: Box<[u8]>,
  offset: usize,
  bit_offset: usize,
}

impl BitData {
  pub fn new(data: &[u8]) -> Self {
    Self {
      data: data.into(),
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
  pub fn has_bits(&self) -> bool {
    self.bit_offset < 8 || self.offset < self.data.len()
  }

  pub fn skip_trailing_bits(&mut self) -> &mut Self {
    if self.bit_offset != 0 {
      self.bit_offset = 0;
      self.offset += 1;
    }
    self
  }

  pub fn byte(&mut self) -> u8 {
    self.bits_into(8)
  }

  pub fn bit_flag(&mut self) -> bool {
    self.bit() != 0
  }

  pub fn bit(&mut self) -> u8 {
    let byte = self.current_byte();
    let bit = (byte >> (7 - self.bit_offset)) & 1;
    self.consume_bits(1);
    bit
  }

  pub fn bits(&mut self, n: u8) -> u8 {
    let byte = self.current_byte();
    let bit = byte << self.bit_offset >> (8 - n);
    self.consume_bits(n as usize);
    bit
  }

  pub fn bits_into<T: LossyFrom<u128>>(&mut self, bits: usize) -> T {
    let number = pack_bits(&self.slice(bits), self.bit_offset, bits);
    self.consume_bits(bits);
    T::lossy_from(number)
  }

  pub fn peek_bits(&mut self, n: u8) -> u8 {
    let byte = self.current_byte();
    byte << self.bit_offset >> (8 - n)
  }

  pub fn exponential_golomb<
    T: Shl<u8, Output = T> + BitOr<Output = T> + Sub<Output = T> + From<u8>,
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

  pub fn signed_exponential_golomb<
    T: Shl<u8, Output = T>
      + Shr<u8, Output = T>
      + BitAnd<Output = T>
      + BitOr<Output = T>
      + Add<Output = T>
      + Sub<Output = T>
      + From<u8>
      + Copy
      + PartialEq
      + Neg<Output = T>,
  >(
    &mut self,
  ) -> T {
    let n: T = self.exponential_golomb();
    let one = T::from(1);
    let signed = (n >> 1) + (n & one);
    if ((n + one) & one) == one {
      -signed
    } else {
      signed
    }
  }

  pub fn next_into<T: LossyFrom<u128> + Sized>(&mut self) -> T {
    self.bits_into(std::mem::size_of::<T>() * 8)
  }

  fn consume_bits(&mut self, bits: usize) {
    let read_bits = self.bit_offset + bits;
    self.bit_offset = read_bits % 8;
    self.offset += read_bits >> 3;
  }

  fn current_byte(&mut self) -> u8 {
    if self.offset > 1 && self.data[self.offset - 2..=self.offset] == EPB {
      self.offset += 1;
    }
    self.data[self.offset]
  }

  fn slice(&mut self, bit_size: usize) -> Box<[u8]> {
    let size = (bit_size + 7) >> 3;
    let size = self.offset + size + if bit_size > 8 - self.bit_offset { 1 } else { 0 };
    if self.offset > 1 {
      let mut skipped = 0;
      return (self.offset..size)
        .map(|i| {
          if self.data[i - 2..=i] == EPB {
            self.offset += 1;
            skipped += 1;
          }
          self.data[i + skipped]
        })
        .collect();
    }
    (&self.data[self.offset..size]).into()
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
