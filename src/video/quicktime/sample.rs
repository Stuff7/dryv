use crate::byte::BitData;

#[derive(Debug)]
pub struct SeiMessage {
  pub size: usize,
  pub payload: SeiPayload,
  pub payload_size: u32,
}

impl SeiMessage {
  pub fn decode(size: usize, data: &mut BitData) -> Self {
    let mut payload_type = 0;
    while data.peek_bits(8) == 0xFF {
      payload_type += data.byte() as u32;
    }
    payload_type += data.byte() as u32;

    let mut payload_size = 0;
    while data.peek_bits(8) == 0xFF {
      payload_size += data.byte() as u32;
    }
    payload_size += data.byte() as u32;

    Self {
      size,
      payload: SeiPayload::new(payload_type, data),
      payload_size,
    }
  }
}

#[derive(Debug)]
pub enum SeiPayload {
  Unknown(u32),
  UserDataUnregistered { uuid_iso_iec_11578: u128 },
}

impl SeiPayload {
  pub fn new(payload_type: u32, data: &mut BitData) -> Self {
    match payload_type {
      5 => Self::UserDataUnregistered {
        uuid_iso_iec_11578: data.next_into(),
      },
      n => Self::Unknown(n),
    }
  }
}
