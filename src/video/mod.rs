pub mod quicktime;

use self::quicktime::atom::AtomError;
use crate::{
  ascii::{Color, RESET},
  byte::Str,
  log,
  math::Matrix3x3,
  time::Duration,
};
use quicktime::*;
use std::{fmt, path::Path, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
  #[error("Could not decode video\n{0}")]
  Decoding(#[from] DecoderError),
  #[error("Could not decode atom\n{0}")]
  AtomDecoding(#[from] AtomError),
  #[error("Could not find video codec")]
  VideoCodec,
}

pub type VideoResult<T = ()> = Result<T, VideoError>;

#[derive(Debug)]
pub enum VideoCodec {
  H264,
  Unknown(Str<4>),
}

impl From<Str<4>> for VideoCodec {
  fn from(value: Str<4>) -> Self {
    match &*value {
      b"avc1" => Self::H264,
      _ => Self::Unknown(value),
    }
  }
}

#[derive(Debug)]
pub struct Video {
  pub timescale: u32,
  pub duration: Duration,
  pub height: f32,
  pub width: f32,
  pub matrix: Matrix3x3,
  pub video_codec: VideoCodec,
}

impl Video {
  pub fn open<P: AsRef<Path>>(path: P) -> VideoResult<Self> {
    let mut decoder = Decoder::open(path)?;
    let mut root = decoder.decode_root()?;
    log!(File@"ROOT {:#?}", root);

    let mut timescale = 0;
    let mut duration = None;
    let mut height = 0.;
    let mut width = 0.;
    let mut matrix = None;
    let mut video_codec = None;

    decoder.decode_udta_meta(&mut root)?;
    decoder.decode_moov_meta(&mut root)?;
    for trak in &mut root.moov.trak {
      let trak = trak.decode(&mut decoder)?;
      let mdia = trak.mdia.decode(&mut decoder)?;
      let hdlr = mdia.hdlr.decode(&mut decoder)?;
      log!(File@"{:-^100}", hdlr.component_subtype.as_string());
      let minf = mdia.minf.decode(&mut decoder)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.DINF.DREF {:#?}", minf.dinf.decode(&mut decoder)?.dref.decode(&mut decoder));
      decoder.decode_stbl(minf)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.MHD {:#?}", minf.mhd);

      if *hdlr.component_subtype == *b"vide" {
        let tkhd = trak.tkhd.decode(&mut decoder)?;
        let mdhd = mdia.mdhd.decode(&mut decoder)?;

        timescale = mdhd.timescale;
        duration = Some(Duration::from_secs_f32(
          mdhd.duration as f32 / timescale as f32,
        ));
        width = tkhd.width;
        height = tkhd.height;
        matrix = Some(tkhd.matrix);
        video_codec = minf
          .stbl
          .decode(&mut decoder)?
          .stsd
          .decode(&mut decoder)?
          .sample_description_table
          .get(0)
          .map(|sample| VideoCodec::from(sample.data_format));
      }

      log!(File@"TRAK.MDIA.MDHD {:#?}", mdia.mdhd);
    }
    Ok(Self {
      timescale,
      duration: duration.unwrap_or_default(),
      width,
      height,
      matrix: matrix.unwrap_or_default(),
      video_codec: video_codec.ok_or(VideoError::VideoCodec)?,
    })
  }
}

impl fmt::Display for Video {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{title}VIDEO INFO{RESET}\n\
      - {title}Video Codec:{RESET} {:?}\n\
      - {title}Matrix:{RESET}\n{}\
      - {title}Rotation:{RESET} {}°\n\
      - {title}Width:{RESET} {}\n\
      - {title}Height:{RESET} {}\n\
      - {title}Duration:{RESET} {}\n\
      - {title}Timescale:{RESET} {:?}",
      self.video_codec,
      self.matrix,
      self.matrix.rotation(),
      self.width,
      self.height,
      self.duration,
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
