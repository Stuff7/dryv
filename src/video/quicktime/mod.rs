pub mod atom;

use atom::*;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QTError {
  #[error("QuickTime Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
}

pub type QTResult<T = ()> = Result<T, QTError>;

pub struct QTDecoder {
  pub file: File,
  pub size: u64,
}

impl QTDecoder {
  pub fn open<P: AsRef<Path>>(path: P) -> QTResult<Self> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    Ok(QTDecoder { size, file })
  }

  pub fn decode(&mut self) -> QTResult<RootAtom> {
    RootAtom::new(&mut self.file, self.size as u32).map_err(QTError::from)
  }
}
