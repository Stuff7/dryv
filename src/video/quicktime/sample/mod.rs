mod nal;

pub use nal::*;

use super::*;
use std::fs::File;

#[derive(Debug, Error)]
pub enum SampleError {
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
  #[error(transparent)]
  Bit(#[from] TryFromIntError),
}

pub type SampleResult<T = ()> = Result<T, SampleError>;

#[derive(Debug)]
pub struct SampleIter {
  chunk_index: usize,
  chunk_offset: u64,
  sample_offset: u64,
  sample_size_offset: u64,
  samples_in_chunk: u64,
  next_stsc: Option<StscItem>,
  reader: File,
  chunk_offsets: std::iter::Enumerate<SampleTable>,
  sample_sizes: SampleTable,
  sample_to_chunk_table: SampleTable<StscItem>,
}

impl SampleIter {
  pub fn new(decoder: &mut Decoder, stbl: &mut StblAtom) -> SampleResult<Self> {
    Ok(Self {
      chunk_index: 0,
      chunk_offset: 0,
      sample_offset: 1,
      sample_size_offset: 0,
      samples_in_chunk: 0,
      next_stsc: None,
      reader: decoder.file.try_clone()?,
      chunk_offsets: stbl.stco.chunk_offset_table(decoder)?.enumerate(),
      sample_sizes: stbl.stsz.sample_size_table(decoder)?,
      sample_to_chunk_table: stbl.stsc.decode(decoder)?.sample_to_chunk_table(decoder)?,
    })
  }
}

impl Iterator for SampleIter {
  type Item = Box<[u8]>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.sample_offset >= self.samples_in_chunk {
      self.sample_offset = 0;
      self.sample_size_offset = 0;
      (self.chunk_index, self.chunk_offset) = self.chunk_offsets.next()?;
      if self
        .next_stsc
        .as_ref()
        .is_some_and(|stsc| self.chunk_index >= stsc.first_chunk as usize - 1)
        || self.next_stsc.is_none()
      {
        let stsc = self
          .next_stsc
          .take()
          .or_else(|| self.sample_to_chunk_table.next())?;
        self.samples_in_chunk = stsc.samples_per_chunk as u64;
        self.next_stsc = self.sample_to_chunk_table.next();
      }
    }
    let sample_size = self.sample_sizes.next()?;

    self
      .reader
      .seek(SeekFrom::Start(self.chunk_offset + self.sample_size_offset))
      .ok()?;
    self.sample_offset += 1;
    self.sample_size_offset += sample_size;
    let mut buffer = vec![0; sample_size as usize];
    self.reader.read_exact(&mut buffer).ok()?;

    Some(buffer.into_boxed_slice())
  }
}
