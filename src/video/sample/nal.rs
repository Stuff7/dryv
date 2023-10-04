use crate::byte::{padded_array_from_slice, BitStream};

#[derive(Debug)]
pub struct SeiMessage {
  pub size: usize,
  pub payload: SeiPayload,
}

impl SeiMessage {
  pub fn decode(size: usize, data: &mut BitStream) -> Self {
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
  pub fn new(payload_type: u32, payload_size: u32, data: &mut BitStream) -> Self {
    match payload_type {
      5 => Self::UserDataUnregistered {
        uuid_iso_iec_11578: data.next_into(),
        data: (&**data)[..payload_size as usize - 16].into(),
      },
      n => Self::Unknown(n),
    }
  }
}

/// Represents the type of a Network Abstraction Layer (NAL) unit in an H.264 video stream.
#[derive(Debug, Clone, Copy)]
pub enum NALUnitType {
  /// Unspecified NAL unit type.
  /// This type is used when the specific NAL unit type is not identified or does not fall into any defined category.
  Unspecified,

  /// Non-IDR (Instantaneous Decoding Refresh) picture.
  /// Non-IDR pictures are reference frames in the video stream that are not keyframes (I-frames).
  /// They depend on previously decoded frames for reconstruction and are used for predictive coding.
  NonIDRPicture,

  /// Data Partition A.
  /// Data Partition A contains a portion of a slice's data and is used for parallel processing
  /// or transmission over different network channels.
  DataPartitionA,

  /// Data Partition B.
  /// Data Partition B contains a portion of a slice's data and is used for parallel processing
  /// or transmission over different network channels.
  DataPartitionB,

  /// Data Partition C.
  /// Data Partition C contains a portion of a slice's data and is used for parallel processing
  /// or transmission over different network channels.
  DataPartitionC,

  /// IDR (Instantaneous Decoding Refresh) picture.
  /// IDR pictures are keyframes in the video stream that allow decoding to start independently.
  /// They are self-contained frames and do not depend on other frames for reconstruction.
  IDRPicture,

  /// Supplemental Enhancement Information (SEI) data.
  /// SEI data contains extra information about the video stream, such as metadata, user data,
  /// and custom signaling messages. It can convey details like display orientation and timestamps.
  Sei,

  /// Sequence Parameter Set (SPS).
  /// SPS contains essential information about the video sequence, including frame dimensions,
  /// bit depth, color representation, and more. It is crucial for proper video decoding.
  Sps,

  /// Picture Parameter Set (PPS).
  /// PPS contains parameters related to individual pictures or frames, including settings
  /// for reference frame usage, slice group configurations, and entropy coding modes.
  Pps,

  /// Access Unit Delimiter.
  /// Access Unit Delimiter is a special NAL unit used to separate access units within the video stream.
  /// It helps identify the start of a new access unit, which may include one or more NAL units.
  AccessUnitDelimeter,

  /// End of Sequence.
  /// This NAL unit type marks the end of a video sequence, indicating the conclusion of a specific video sequence.
  EndOfSequence,

  /// End of Stream.
  /// End of Stream is used to indicate the conclusion of the entire video stream.
  /// It marks the end of the video playback.
  EndOfStream,

  /// Filler Data.
  /// Filler Data NAL units are used to insert extra bits for byte or word alignment purposes.
  /// They ensure that the bitstream adheres to specified byte or word boundaries.
  FillerData,

  /// SPS Extension.
  /// SPS Extension contains additional parameters or configurations beyond the standard SPS.
  /// It is used for special cases or profiles that require extended settings.
  SPSExtension,

  /// Prefix NAL Unit.
  /// Prefix NAL Units are used in certain contexts to signal changes in the Advanced Video Coding (AVC) configuration.
  /// They may precede other NAL units and provide important information about the codec configuration.
  PrefixNALUnit,

  /// Subset Sequence Parameter Set (Subset SPS).
  /// Subset SPS is a simplified version of the SPS, containing a subset of parameters.
  /// It is used in cases where a reduced set of settings is sufficient for decoding.
  SubsetSPS,

  /// Depth Parameter Set (DPS).
  /// DPS contains depth information for 3D video streams, enabling stereoscopic rendering.
  /// It provides depth data for each pixel, allowing for the creation of 3D effects.
  DepthParameterSet,

  /// Reserved NAL unit type.
  /// This NAL unit type is reserved for future use. It does not have a defined purpose in the current standard
  /// but may be utilized for new features or extensions in future video coding standards.
  Reserved,

  /// Auxiliary Coded Picture.
  /// Auxiliary Coded Picture NAL units contain additional coded picture information that may not fit
  /// within the regular picture structure. They provide extra data for specific purposes or use cases.
  AuxiliaryCodedPicture,

  /// Coded Slice Extension.
  /// Coded Slice Extension NAL units are extensions of coded slices in certain contexts.
  /// They provide additional data or options for a specific slice, offering advanced coding features
  /// or customizations beyond the standard slice structure.
  CodedSliceExtension,

  /// Depth or Texture View Component.
  /// Depth or Texture View Component NAL units are used in 3D video streams to represent either depth information
  /// or texture views, depending on the context. They are crucial for stereoscopic rendering and 3D visualization.
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
