mod atom;
mod decoder;

pub use decoder::*;

fn bytes_to_i32(bytes: &[u8]) -> i32 {
  assert!(bytes.len() == 4, "Input slice must have exactly 4 bytes");

  let mut result: i32 = 0;

  for i in 0..4 {
    result |= (bytes[i] as i32) << (i * 8);
  }

  result
}

fn bytes_to_i64(bytes: &[u8]) -> i64 {
  assert!(bytes.len() == 8, "Input slice must have exactly 8 bytes");

  let mut result: i64 = 0;

  for i in 0..8 {
    result |= (bytes[i] as i64) << (i * 8);
  }

  result
}

#[derive(Debug, Clone, Copy)]
struct Fixed16(i32);

impl Fixed16 {
  fn from_float(value: f32) -> Self {
    Fixed16((value * (1 << 16) as f32) as i32)
  }

  fn to_float(self) -> f32 {
    self.0 as f32 / (1 << 16) as f32
  }
}

#[derive(Debug, Clone, Copy)]
struct Fixed32(i64);

impl Fixed32 {
  fn from_float(value: f64) -> Self {
    Fixed32((value * (1i64 << 32) as f64) as i64)
  }

  fn to_float(self) -> f64 {
    self.0 as f64 / (1i64 << 32) as f64
  }
}
