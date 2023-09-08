use crate::{ascii::LogDisplay, log};

use super::atom::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MP4Error {
  #[error("MP4 IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Box(#[from] BoxError),
}

pub type MP4Result<T = ()> = Result<T, MP4Error>;

pub struct MP4Decoder {
  file: std::fs::File,
  size: u64,
}

impl MP4Decoder {
  pub fn new(file_path: &str) -> MP4Result<Self> {
    let file = std::fs::File::open(file_path)?;
    Ok(MP4Decoder {
      size: file.metadata()?.len(),
      file,
    })
  }

  pub fn decode(&mut self) -> MP4Result {
    println!("FILE LEN: {}", self.size);
    let atoms = AtomBoxIter::new(&mut self.file, self.size as u32);
    for atom in atoms {
      match atom {
        Ok(atom) => log!("Atom Box: {atom:#?}"),
        Err(e) => log!(err@"{e}"),
      }
    }
    Ok(())
  }
}
