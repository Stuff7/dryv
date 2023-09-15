pub mod atom;

use atom::*;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use thiserror::Error;

use crate::byte::Str;
use crate::log;

#[derive(Debug, Error)]
pub enum DecoderError {
  #[error("Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
  #[error("Decoder does not support brand {0:?}")]
  Unsupported(Str<4>),
}

pub type DecoderResult<T = ()> = Result<T, DecoderError>;

#[derive(Debug)]
pub enum DecoderBrand {
  QuickTime,
  Isom,
  None,
}

impl DecoderBrand {
  pub fn is_isom(&self) -> bool {
    matches!(self, DecoderBrand::Isom)
  }
}

impl TryFrom<Str<4>> for DecoderBrand {
  type Error = DecoderError;
  fn try_from(brand: Str<4>) -> Result<Self, Self::Error> {
    match &*brand {
      b"qt  " => Ok(Self::QuickTime),
      b"isom" => Ok(Self::Isom),
      _ => Err(DecoderError::Unsupported(brand)),
    }
  }
}

#[derive(Debug)]
pub struct Decoder {
  pub file: File,
  pub size: u64,
  pub brand: DecoderBrand,
}

impl Decoder {
  pub fn open<P: AsRef<Path>>(path: P) -> DecoderResult<Self> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    Ok(Self {
      size,
      file,
      brand: DecoderBrand::None,
    })
  }

  pub fn decode_root(&mut self) -> DecoderResult<RootAtom> {
    let root = RootAtom::new(&mut self.file, self.size)?;
    self.brand = DecoderBrand::try_from(root.ftyp.major_brand).or_else(|e| {
      root
        .ftyp
        .compatible_brands
        .iter()
        .find_map(|b| DecoderBrand::try_from(*b).ok())
        .ok_or(e)
    })?;
    Ok(root)
  }

  pub fn decode_moov_meta<'a>(
    &mut self,
    root: &'a mut RootAtom,
  ) -> DecoderResult<Option<&'a mut MetaAtom>> {
    log!(File@"{:-^100}", "meta");
    root
      .moov
      .meta
      .as_mut()
      .map(|meta| {
        let meta = meta.decode(self)?;
        log!(File@"MOOV.META TAGS {:#?}", meta.tags(self));
        meta.ilst.decode(self)?;
        meta.hdlr.decode(self)?;
        meta.keys.decode(self)?;
        log!(File@"MOOV.META {:#?}", meta);
        Ok(meta)
      })
      .transpose()
  }

  pub fn decode_udta_meta<'a>(
    &mut self,
    root: &'a mut RootAtom,
  ) -> DecoderResult<Vec<&'a mut MetaAtom>> {
    log!(File@"{:-^100}", "meta");
    let decoder = self;
    root
      .moov
      .udta
      .as_mut()
      .map(|udta| {
        let udta = udta.decode(decoder)?;
        udta
          .metas
          .iter_mut()
          .map(|meta| {
            let meta = meta.decode(decoder)?;
            meta.ilst.decode(decoder)?;
            meta.hdlr.decode(decoder)?;
            log!(File@"MOOV.UDTA.META TAGS {:#?}", meta.tags(decoder));
            Ok(meta)
          })
          .collect::<DecoderResult<Vec<_>>>()
      })
      .unwrap_or_else(|| Ok(Vec::new()))
  }

  pub fn decode_stbl<'a>(&mut self, trak: &'a mut TrakAtom) -> DecoderResult<&'a mut StblAtom> {
    let mdia = trak.mdia.decode(self)?;
    log!(File@"ROOT.TRAK.MDIA.HDLR {:#?}", mdia.hdlr.decode(self)?);
    let minf = mdia.minf.decode(self)?;
    let stbl = minf.stbl.decode(self)?;
    stbl.stts.decode(self)?;
    stbl.stsd.decode(self)?;
    for stsd in &mut stbl.stsd.decode(self)?.sample_description_table {
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSD {:#?}", stsd);
    }
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STCO {} {:#?}",
      stbl.stco.number_of_entries,
      stbl.stco.chunk_offset_table(self).take(10).collect::<Vec<_>>()
    );
    {
      let stsz = stbl.stsz.decode(self)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSZ {} {:#?}",
        stsz.number_of_entries,
        stsz.sample_size_table(self).take(10).collect::<Vec<_>>()
      );
    }
    {
      let stsc = stbl.stsc.decode(self)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSC {} {:#?}",
        stsc.number_of_entries,
        stsc.sample_to_chunk_table(self).take(10).collect::<Vec<_>>(),
      );
    }
    if let Some(stss) = &mut stbl.stss {
      let stss = stss.decode(self)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSS {} {:#?}",
        stss.number_of_entries,
        stss.sync_sample_table(self).take(10).collect::<Vec<_>>()
      );
    }
    if let Some(ctts) = &mut stbl.ctts {
      ctts.decode(self)?;
    }
    if let Some(sgpd) = &mut stbl.sgpd {
      sgpd.decode(self)?;
    }
    if let Some(sbgp) = &mut stbl.sbgp {
      sbgp.decode(self)?;
    }
    Ok(stbl)
  }
}

impl Read for Decoder {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.file.read(buf)
  }
}

impl Seek for Decoder {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    self.file.seek(pos)
  }
}
