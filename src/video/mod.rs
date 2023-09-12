pub mod quicktime;

use self::quicktime::atom::AtomError;
use crate::{
  ascii::{Color, RESET},
  log,
  math::Matrix3x3,
};
use quicktime::*;
use std::{fmt, path::Path, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
  #[error("Could not decode video\n{0}")]
  Decoding(#[from] QTError),
  #[error("Could not decode atom\n{0}")]
  AtomDecoding(#[from] AtomError),
}

pub type VideoResult<T = ()> = Result<T, VideoError>;

#[derive(Debug)]
pub struct Video {
  pub timescale: u32,
  pub duration_ms: f32,
  pub height: f32,
  pub width: f32,
  pub matrix: Matrix3x3,
}

impl Video {
  pub fn open<P: AsRef<Path>>(path: P) -> VideoResult<Self> {
    let mut decoder = QTDecoder::open(path)?;
    let mut root = decoder.decode()?;
    log!(File@"FTYP {:#?}", root.ftyp);

    let mut video = Self {
      timescale: 0,
      duration_ms: 0.,
      height: 0.,
      width: 0.,
      matrix: Matrix3x3::identity(),
    };

    if let Some(udta) = &mut root.moov.udta {
      let udta = udta.decode(&mut decoder.file)?;
      for meta in &mut udta.metas {
        meta
          .decode(&mut decoder.file)?
          .ilst
          .decode(&mut decoder.file)?;
      }
    }
    if let Some(meta) = &mut root.moov.meta {
      let meta = meta.decode(&mut decoder.file)?;
      meta.ilst.decode(&mut decoder.file)?;
      meta.hdlr.decode(&mut decoder.file)?;
      meta.keys.decode(&mut decoder.file)?;
    }
    for trak in &mut root.moov.trak {
      let trak = trak.decode(&mut decoder.file)?;
      let mdia = trak.mdia.decode(&mut decoder.file)?;
      let hdlr = mdia.hdlr.decode(&mut decoder.file)?;
      if let Some(edts) = &mut trak.edts {
        edts
          .decode(&mut decoder.file)?
          .elst
          .decode(&mut decoder.file)?;
      }
      let minf = mdia.minf.decode(&mut decoder.file)?;
      minf
        .dinf
        .decode(&mut decoder.file)?
        .dref
        .decode(&mut decoder.file)?;

      {
        let stbl = minf.stbl.decode(&mut decoder.file)?;
        stbl.stsd.decode(&mut decoder.file)?;
        stbl.stts.decode(&mut decoder.file)?;
        if let Some(stss) = &mut stbl.stss {
          stss.decode(&mut decoder.file)?;
        }
        if let Some(ctts) = &mut stbl.ctts {
          ctts.decode(&mut decoder.file)?;
        }
        stbl.stsc.decode(&mut decoder.file)?;
        stbl.stsz.decode(&mut decoder.file)?;
        stbl.stco.decode(&mut decoder.file)?;
        if let Some(sgpd) = &mut stbl.sgpd {
          sgpd.decode(&mut decoder.file)?;
        }
        if let Some(sbgp) = &mut stbl.sbgp {
          sbgp.decode(&mut decoder.file)?;
        }
      }

      if *hdlr.component_type == *b"vide" {
        let tkhd = trak.tkhd.decode(&mut decoder.file)?;
        let mdhd = mdia.mdhd.decode(&mut decoder.file)?;
        video.timescale = mdhd.timescale;
        video.duration_ms = mdhd.duration as f32 / video.timescale as f32 * 1000.;
        video.width = tkhd.width;
        video.height = tkhd.height;
        video.matrix = tkhd.matrix;
      }
      log!(File@"{:-^100}", hdlr.component_type.as_string());
      log!(File@"TRAK.MEDIA.HDLR {:#?}", hdlr);
    }
    log!(File@"MOOV.META {:#?}", root.moov.meta);
    Ok(video)
  }
}

impl fmt::Display for Video {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{title}VIDEO INFO{RESET}\n\
      - {title}Matrix:{RESET}\n{}\
      - {title}Rotation:{RESET} {}°\n\
      - {title}Width:{RESET} {}\n\
      - {title}Height:{RESET} {}\n\
      - {title}Duration:{RESET} {}ms\n\
      - {title}Timescale:{RESET} {:?}",
      self.matrix,
      self.matrix.rotation(),
      self.width,
      self.height,
      self.duration_ms,
      self.timescale,
      title = "".rgb(75, 205, 94).bold(),
    )
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SeekPosition {
  Seconds(i64),
  Milliseconds(i64),
  Percentage(f64),
  TimeBase(i64),
}

impl FromStr for SeekPosition {
  type Err = Box<dyn std::error::Error>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if let Some(s) = s.strip_suffix("ms") {
      Self::Milliseconds(s.parse()?)
    } else if let Some(s) = s.strip_suffix('%') {
      Self::Percentage(s.parse::<f64>()? / 100.)
    } else if let Some(s) = s.strip_suffix("ts") {
      Self::TimeBase(s.parse()?)
    } else if let Some(s) = s.strip_suffix('s') {
      Self::Seconds(s.parse()?)
    } else {
      Self::Seconds(s.parse()?)
    })
  }
}

impl Default for SeekPosition {
  fn default() -> Self {
    Self::TimeBase(0)
  }
}
