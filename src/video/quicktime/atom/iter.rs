use super::*;

#[derive(Debug)]
pub struct AtomDataIter<'a> {
  data: &'a [u8],
  start: u64,
  end: u64,
  reader_offset: u64,
}

impl<'a> AtomDataIter<'a> {
  pub fn new(data: &'a [u8], reader_offset: u64) -> Self {
    Self {
      data,
      start: 0,
      end: data.len() as u64,
      reader_offset,
    }
  }
}

impl<'a> Iterator for AtomDataIter<'a> {
  type Item = AtomResult<(Atom, &'a [u8])>;

  fn next(&mut self) -> Option<Self::Item> {
    (self.start + HEADER_SIZE < self.end).then(|| {
      let s = self.start as usize;
      let e = s + HEADER_SIZE as usize;
      decode_header(&self.data[s..e]).and_then(|(atom_size, atom_type)| {
        let offset = self.start + HEADER_SIZE + self.reader_offset;
        self.start += atom_size as u64;
        Ok((
          Atom::new(atom_size, atom_type, offset)?,
          &self.data[s + 8..s + atom_size as usize],
        ))
      })
    })
  }
}

pub struct AtomIter<'a, R: Read + Seek> {
  pub reader: &'a mut R,
  pub buffer: [u8; HEADER_SIZE as usize],
  pub start: u64,
  pub end: u64,
}

impl<'a, R: Read + Seek> AtomIter<'a, R> {
  pub fn new(reader: &'a mut R, start: u64, end: u64) -> Self {
    Self {
      reader,
      buffer: [0; HEADER_SIZE as usize],
      start,
      end,
    }
  }
}

impl<'a, R: Read + Seek> Iterator for AtomIter<'a, R> {
  type Item = AtomResult<Atom>;

  fn next(&mut self) -> Option<Self::Item> {
    (self.start + HEADER_SIZE < self.end).then(|| {
      self.reader.seek(SeekFrom::Start(self.start))?;
      self.reader.read_exact(&mut self.buffer)?;
      decode_header(&self.buffer).and_then(|(atom_size, atom_type)| {
        let offset = self.start + HEADER_SIZE;
        self.start += atom_size as u64;
        Atom::new(atom_size, atom_type, offset)
      })
    })
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
