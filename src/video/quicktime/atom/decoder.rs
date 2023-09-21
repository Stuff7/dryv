use super::*;
use crate::{
  byte::{pack_bits, Str, TryFromSlice},
  math::fixed_point_to_f32,
};
use std::{num::TryFromIntError, ops::Deref};

pub const HEADER_SIZE: u64 = 8;

#[derive(Debug)]
pub struct UnknownAtom(Atom);

impl AtomDecoder for UnknownAtom {
  const NAME: [u8; 4] = [0; 4];
  fn decode_unchecked(atom: Atom, _: &mut Decoder) -> AtomResult<Self> {
    Err(AtomError::UnknownAtom(atom))
  }
}

#[derive(Debug, Default)]
pub enum EncodedAtom<T: AtomDecoder = UnknownAtom> {
  Encoded(Atom),
  Decoded(T),
  #[default]
  Required,
}

impl<T: AtomDecoder> EncodedAtom<T> {
  pub fn decode(&mut self, decoder: &mut Decoder) -> AtomResult<&mut T> {
    match self {
      EncodedAtom::Decoded(decoded) => Ok(decoded),
      EncodedAtom::Encoded(atom) => {
        let decoded = T::decode(*atom, decoder)?;
        *self = EncodedAtom::Decoded(decoded);
        if let EncodedAtom::Decoded(decoded) = self {
          Ok(decoded)
        } else {
          Err(AtomError::Required(T::NAME))
        }
      }
      EncodedAtom::Required => Err(AtomError::Required(T::NAME)),
    }
  }
}

pub trait AtomDecoder: std::marker::Sized {
  const NAME: [u8; 4] = [0; 4];
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self>;
  #[inline]
  fn decode(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    if *atom.name == Self::NAME {
      Self::decode_unchecked(atom, decoder)
    } else {
      Err(AtomError::AtomType(Str(Self::NAME), atom.name))
    }
  }
}

#[derive(Debug)]
pub struct AtomData {
  data: Box<[u8]>,
  offset: usize,
  reader_offset: u64,
}

impl AtomData {
  pub fn new(data: &[u8], reader_offset: u64) -> Self {
    Self {
      data: data.into(),
      offset: 0,
      reader_offset,
    }
  }
}

impl Deref for AtomData {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
    &self.data[self.offset..]
  }
}

impl AtomData {
  pub fn reserved(&mut self, size: usize) -> &mut Self {
    self.offset += size;
    self
  }

  pub fn version(&mut self) -> u8 {
    self.offset += 1;
    self.data[0]
  }

  pub fn flags(&mut self) -> [u8; 3] {
    self.offset += 3;
    [self.data[1], self.data[2], self.data[3]]
  }

  pub fn byte(&mut self) -> u8 {
    let index = self.offset;
    self.offset += 1;
    self.data[index]
  }

  pub fn fixed_point_16(&mut self) -> AtomResult<f32> {
    let s = self.offset;
    self.offset += 4;
    Ok(fixed_point_to_f32(
      i32::from_be_bytes((&self.data[s..self.offset]).try_into()?) as f32,
      16,
    ))
  }

  pub fn fixed_point_8(&mut self) -> AtomResult<f32> {
    let s = self.offset;
    self.offset += 2;
    Ok(fixed_point_to_f32(
      i16::from_be_bytes((&self.data[s..self.offset]).try_into()?) as f32,
      8,
    ))
  }

  pub fn next_into<T: TryFromSlice>(&mut self) -> AtomResult<T> {
    let s = self.offset;
    self.offset += T::SIZE;
    Ok(T::try_from_slice(&self.data[s..self.offset])?)
  }

  pub fn next(&mut self, size: usize) -> &[u8] {
    let s = self.offset;
    self.offset += size;
    &self.data[s..self.offset]
  }

  pub fn atoms(&self) -> AtomDataIter {
    AtomDataIter::new(
      &self.data[self.offset..],
      self.reader_offset + self.offset as u64,
    )
  }
}

#[derive(Debug)]
pub struct AtomBitData {
  data: Box<[u8]>,
  offset: usize,
  bit_offset: usize,
}

impl From<AtomData> for AtomBitData {
  fn from(data: AtomData) -> Self {
    Self {
      data: data.deref().into(),
      offset: 0,
      bit_offset: 0,
    }
  }
}

impl Deref for AtomBitData {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
    &self.data[self.offset..]
  }
}

impl AtomBitData {
  pub fn byte(&mut self) -> AtomResult<u8> {
    self.bits_into(8)
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
  ) -> AtomResult<T> {
    let number = pack_bits(&self.data[self.offset..], self.bit_offset, bits);
    self.consume_bits(bits);
    Ok(number.try_into()?)
  }

  pub fn exponential_golomb(&mut self) -> u32 {
    let mut bits = BitIter::new(self.deref(), self.bit_offset);
    let k = bits.position(|bit| bit == 1).unwrap_or_default();
    let x = bits.take(k).fold(1, |x, bit| x << 1 | bit as u32);
    self.consume_bits(k + k + 1);
    x - 1
  }

  pub fn next_into<T: TryFrom<u128, Error = TryFromIntError> + Sized>(&mut self) -> AtomResult<T> {
    self.bits_into(std::mem::size_of::<T>() * 8)
  }

  fn consume_bits(&mut self, bits: usize) {
    let read_bits = self.bit_offset + bits;
    self.bit_offset = read_bits % 8;
    self.offset += read_bits >> 3;
  }
}

pub fn decode_header(data: &[u8]) -> AtomResult<(u32, &[u8])> {
  let size = u32::from_be_bytes((&data[..4]).try_into()?);
  Ok((size, &data[4..8]))
}

pub fn unpack_language_code(bytes: &[u8]) -> AtomResult<[u8; 3]> {
  let code = u16::from_be_bytes((bytes).try_into()?);
  let char1 = ((code >> 10) & 0x1F) as u8 + 0x60;
  let char2 = ((code >> 5) & 0x1F) as u8 + 0x60;
  let char3 = (code & 0x1F) as u8 + 0x60;
  Ok([char1, char2, char3])
}
