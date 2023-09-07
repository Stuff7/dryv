use std::str::FromStr;

pub mod mp4;

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
