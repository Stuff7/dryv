use crate::byte::{padded_array_from_slice, BitData};

#[derive(Debug)]
pub struct SeiMessage {
  pub size: usize,
  pub payload: SeiPayload,
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
      payload: SeiPayload::new(payload_type, payload_size, data),
    }
  }
}

#[derive(Debug)]
pub enum SeiPayload {
  Unknown(u32),
  UserDataUnregistered {
    uuid_iso_iec_11578: u128,
    data: Box<[u8]>,
  },
}

impl SeiPayload {
  pub fn new(payload_type: u32, payload_size: u32, data: &mut BitData) -> Self {
    match payload_type {
      5 => Self::UserDataUnregistered {
        uuid_iso_iec_11578: data.next_into(),
        data: (&**data)[..payload_size as usize - 16].into(),
      },
      n => Self::Unknown(n),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum NALUnitType {
  Unspecified,
  NonIDRPicture,
  DataPartitionA,
  DataPartitionB,
  DataPartitionC,
  IDRPicture,
  Sei,
  Sps,
  Pps,
  AccessUnitDelimeter,
  EndOfSequence,
  EndOfStream,
  FillerData,
  SPSExtension,
  PrefixNALUnit,
  SubsetSPS,
  DepthParameterSet,
  Reserved,
  AuxiliaryCodedPicture,
  CodedSliceExtension,
  DepthOrTextureViewComponent,
}

impl NALUnitType {
  pub fn new(nal_unit_type: u8) -> Self {
    match nal_unit_type {
      1 => Self::NonIDRPicture,
      2 => Self::DataPartitionA,
      3 => Self::DataPartitionB,
      4 => Self::DataPartitionC,
      5 => Self::IDRPicture,
      6 => Self::Sei,
      7 => Self::Sps,
      8 => Self::Pps,
      9 => Self::AccessUnitDelimeter,
      10 => Self::EndOfSequence,
      11 => Self::EndOfStream,
      12 => Self::FillerData,
      13 => Self::SPSExtension,
      14 => Self::PrefixNALUnit,
      15 => Self::SubsetSPS,
      16 => Self::DepthParameterSet,
      17 | 18 | 22 | 23 => Self::Reserved,
      19 => Self::AuxiliaryCodedPicture,
      20 => Self::CodedSliceExtension,
      21 => Self::DepthOrTextureViewComponent,
      _ => Self::Unspecified,
    }
  }

  pub fn is_idr(&self) -> bool {
    matches!(self, NALUnitType::IDRPicture)
  }
}

#[derive(Debug)]
pub struct NALUnit<'a> {
  pub idc: u8,
  pub unit_type: NALUnitType,
  pub size: usize,
  pub data: &'a [u8],
}

#[derive(Debug)]
pub struct NALUnitIter<'a> {
  nal_length_size: usize,
  data: &'a [u8],
  offset: usize,
}

impl<'a> NALUnitIter<'a> {
  pub fn new(data: &'a [u8], nal_length_size: usize) -> Self {
    Self {
      data,
      nal_length_size,
      offset: 0,
    }
  }
}

impl<'a> Iterator for NALUnitIter<'a> {
  type Item = NALUnit<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    (self.offset + self.nal_length_size < self.data.len()).then(|| {
      let s = self.offset;
      self.offset += self.nal_length_size;
      let mut nal_size = usize::from_be_bytes(padded_array_from_slice(&self.data[s..self.offset]));
      nal_size >>= (std::mem::size_of::<usize>() - self.nal_length_size) * 8;
      let idc = self.data[self.offset] & 0x60 >> 5;
      let nal_type = self.data[self.offset] & 0x1F;
      let nal_unit = NALUnit {
        idc,
        unit_type: NALUnitType::new(nal_type),
        size: nal_size,
        data: &self.data[self.offset + 1..],
      };
      self.offset += nal_size;
      nal_unit
    })
  }
}
