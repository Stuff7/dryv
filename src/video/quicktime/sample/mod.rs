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

pub struct SampleIter {
  reader: File,
  sync_samples: Option<SampleTable>,
  chunk_offsets: SampleTable,
  sample_sizes: SampleTable,
}

impl SampleIter {
  pub fn new(decoder: &mut Decoder, stbl: &mut StblAtom) -> SampleResult<Self> {
    Ok(Self {
      reader: decoder.file.try_clone()?,
      sync_samples: stbl
        .stss
        .as_mut()
        .map(|stss| stss.sync_sample_table(decoder))
        .transpose()?,
      chunk_offsets: stbl.stco.chunk_offset_table(decoder)?,
      sample_sizes: stbl.stsz.sample_size_table(decoder)?,
    })
  }
}

impl Iterator for SampleIter {
  type Item = Box<[u8]>;
  fn next(&mut self) -> Option<Self::Item> {
    if let Some(ref mut sync_samples) = self.sync_samples {
      if let Some(chunk_offset_index) = sync_samples.next() {
        let chunk_offset = self.chunk_offsets.nth(chunk_offset_index as usize - 1)?;
        let sample_size = self.sample_sizes.next()?;

        self.reader.seek(SeekFrom::Start(chunk_offset)).ok()?;
        let mut buffer = vec![0; sample_size as usize];
        self.reader.read_exact(&mut buffer).ok()?;

        return Some(buffer.into_boxed_slice());
      }
    }
    todo!("OOPS")
  }
}
