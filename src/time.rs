use core::time;
use std::{fmt::Display, ops::Deref};

#[derive(Debug, Default)]
pub struct Duration(time::Duration);

impl Duration {
  pub fn from_secs_f32(secs: f32) -> Self {
    Self(time::Duration::from_secs_f32(secs))
  }
}

impl Display for Duration {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let days = self.as_secs() / 86_400;
    let hours = (self.as_secs() % 86_400) / 3_600;
    let minutes = (self.as_secs() % 3_600) / 60;
    let seconds = self.as_secs() % 60;
    let microseconds = self.subsec_nanos() as f32 / 1000.;

    let mut formatted = String::new();

    if days > 0 {
      formatted.push_str(&format!("{}d ", days));
    }
    if hours > 0 || !formatted.is_empty() {
      formatted.push_str(&format!("{}h ", hours));
    }
    if minutes > 0 || !formatted.is_empty() {
      formatted.push_str(&format!("{}m ", minutes));
    }
    if seconds > 0 || !formatted.is_empty() {
      formatted.push_str(&format!("{}s ", seconds));
    }
    if microseconds > 0. || !formatted.is_empty() {
      formatted.push_str(&format!("{}μs ", microseconds));
    }

    write!(f, "{}", formatted)
  }
}

impl Deref for Duration {
  type Target = time::Duration;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
