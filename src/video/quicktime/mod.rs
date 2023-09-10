mod atom;

use atom::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QTError {
  #[error("QuickTime Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Box(#[from] BoxError),
}

pub type QTResult<T = ()> = Result<T, QTError>;

pub struct QTDecoder {
  file: std::fs::File,
  size: u64,
}

impl QTDecoder {
  pub fn new(file_path: &str) -> QTResult<Self> {
    let file = std::fs::File::open(file_path)?;
    Ok(QTDecoder {
      size: file.metadata()?.len(),
      file,
    })
  }

  pub fn decode(&mut self) -> Vec<BoxResult<AtomBox>> {
    println!("FILE LEN: {}", self.size);
    let atoms = AtomBoxIter::new(&mut self.file, self.size as u32);
    atoms.collect()
  }
}
