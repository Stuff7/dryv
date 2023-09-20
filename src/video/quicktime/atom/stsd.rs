use super::*;
use crate::ascii::LogDisplay;
use crate::byte::pascal_string;
use crate::log;

#[derive(Debug)]
pub struct StsdAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sample_description_table: Box<[StsdCodec]>,
}

impl AtomDecoder for StsdAtom {
  const NAME: [u8; 4] = *b"stsd";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;

    Ok(Self {
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
      sample_description_table: data
        .atoms()
        .filter_map(|atom| match atom {
          Ok((atom, data)) => Some(StsdCodec::new(atom.name, AtomData::new(data, atom.offset))),
          Err(e) => {
            log!(err@"#[stsd] {e}");
            None
          }
        })
        .collect::<AtomResult<_>>()?,
    })
  }
}

#[derive(Debug)]
pub struct StsdCodec {
  pub data_format: Str<4>,
  pub dref_index: u16,
  pub data: CodecData,
}

impl StsdCodec {
  pub fn new(data_format: Str<4>, mut data: AtomData) -> AtomResult<Self> {
    data.reserved(6);
    Ok(Self {
      data_format,
      dref_index: data.next_into()?,
      data: CodecData::new(data_format, data)?,
    })
  }
}

#[derive(Debug)]
pub enum CodecData {
  Avc1(Avc1Atom),
  Mp4a(Mp4aAtom),
  Unknown(Str<4>),
}

impl CodecData {
  fn new(hdlr: Str<4>, data: AtomData) -> AtomResult<Self> {
    Ok(match &*hdlr {
      b"avc1" => Self::Avc1(Avc1Atom::decode(data)?),
      b"mp4a" => Self::Mp4a(Mp4aAtom::decode(data)?),
      _ => Self::Unknown(hdlr),
    })
  }
}

#[derive(Debug)]
pub struct Avc1Atom {
  pub revision_level: u16,
  pub version: u16,
  pub vendor: u32,
  pub temporal_quality: u32,
  pub spatial_quality: u32,
  pub width: u16,
  pub height: u16,
  pub horizontal_resolution: f32,
  pub vertical_resolution: f32,
  pub data_size: u32,
  pub frame_count: u16,
  pub compressor_name: Box<str>,
  pub depth: i16,
  pub color_table_id: i16,
  pub avcc: AvcCAtom,
}

impl Avc1Atom {
  pub fn decode(mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      revision_level: data.next_into()?,
      version: data.next_into()?,
      vendor: data.next_into()?,
      temporal_quality: data.next_into()?,
      spatial_quality: data.next_into()?,
      width: data.next_into()?,
      height: data.next_into()?,
      horizontal_resolution: data.fixed_point_16()?,
      vertical_resolution: data.fixed_point_16()?,
      data_size: data.next_into()?,
      frame_count: data.next_into()?,
      compressor_name: pascal_string(data.next(32)),
      depth: data.next_into()?,
      color_table_id: data.next_into()?,
      avcc: {
        let (atom, data) = data
          .atoms()
          .find_map(|res| {
            res
              .map(|(atom, data)| (*atom.name == AvcCAtom::TYPE).then_some((atom, data)))
              .transpose()
          })
          .ok_or(AtomError::Required(AvcCAtom::TYPE))??;
        AvcCAtom::decode(AtomData::new(data, atom.offset))?
      },
    })
  }
}

#[derive(Debug)]
pub struct AvcCAtom {
  pub configuration_version: u8,
  pub profile_indication: u8,
  pub profile_compatibility: u8,
  pub level_indication: u8,
  pub nal_length_size_minus_one: u8,
  pub num_sps: u8,
  pub sps: SequenceParameterSet,
}

impl AvcCAtom {
  const TYPE: [u8; 4] = *b"avcC";
  pub fn decode(mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      configuration_version: data.byte(),
      profile_indication: data.byte(),
      profile_compatibility: data.byte(),
      level_indication: data.byte(),
      nal_length_size_minus_one: data.byte() & 0b0000_0011,
      num_sps: data.byte() & 0b0001_1111,
      sps: SequenceParameterSet::decode(data)?,
    })
  }
}

#[derive(Debug)]
pub struct SequenceParameterSet {
  pub length: u16,
  pub forbidden_zero_bit: u8,
  pub nal_ref_idc: u8,
  pub nal_unit_type: u8,
  pub profile_idc: u8,
  pub constraint_set0_flag: bool,
  pub constraint_set1_flag: bool,
  pub constraint_set2_flag: bool,
  pub constraint_set3_flag: bool,
  pub constraint_set4_flag: bool,
  pub constraint_set5_flag: bool,
  pub reserved_zero_2bits: bool,
  pub level_idc: u8,
  pub id: u32,
  pub chroma_format_idc: u32,
  pub separate_colour_plane_flag: u8,
  pub bit_depth_luma_minus8: u32,
  pub bit_depth_chroma_minus8: u32,
  pub qpprime_y_zero_transform_bypass_flag: u8,
  pub seq_scaling_matrix_present_flag: u8,
  pub log2_max_frame_num_minus4: u32,
  pub pic_order_cnt_type: u32,
  pub log2_max_pic_order_cnt_lsb_minus4: Option<u32>,
  pub max_num_ref_frames: u32,
  pub gaps_in_frame_num_value_allowed_flag: bool,
  pub pic_width_in_mbs_minus1: u32,
  pub pic_height_in_map_units_minus1: u32,
  pub frame_mbs_only_flag: bool,
  pub mb_adaptive_frame_field_flag: bool,
  pub direct_8x8_inference_flag: bool,
  pub frame_cropping_flag: bool,
  pub vui_parameters_present_flag: bool,
}

impl SequenceParameterSet {
  pub fn decode(mut data: AtomData) -> AtomResult<Self> {
    let pic_order_cnt_type;
    let frame_mbs_only_flag;
    let profile_idc;

    let mut chroma_format_idc = 1;
    let mut separate_colour_plane_flag = 0;
    let mut bit_depth_luma_minus8 = 0;
    let mut bit_depth_chroma_minus8 = 0;
    let mut qpprime_y_zero_transform_bypass_flag = 0;
    let mut seq_scaling_matrix_present_flag = 0;
    Ok(Self {
      length: data.next_into()?,
      forbidden_zero_bit: data.bit(),
      nal_ref_idc: data.bits(2),
      nal_unit_type: data.bits(5),
      profile_idc: {
        profile_idc = data.byte();
        profile_idc
      },
      constraint_set0_flag: data.bit() != 0,
      constraint_set1_flag: data.bit() != 0,
      constraint_set2_flag: data.bit() != 0,
      constraint_set3_flag: data.bit() != 0,
      constraint_set4_flag: data.bit() != 0,
      constraint_set5_flag: data.bit() != 0,
      reserved_zero_2bits: data.bits(2) != 0,
      level_idc: data.byte(),
      id: data.exponential_golomb(),
      chroma_format_idc: {
        match profile_idc {
          100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
            chroma_format_idc = data.exponential_golomb();
            if chroma_format_idc == 3 {
              separate_colour_plane_flag = data.bit();
            }
            bit_depth_luma_minus8 = data.exponential_golomb();
            bit_depth_chroma_minus8 = data.exponential_golomb();
            qpprime_y_zero_transform_bypass_flag = data.bit();
            seq_scaling_matrix_present_flag = data.bit();
          }
          _ => (),
        }
        chroma_format_idc
      },
      separate_colour_plane_flag,
      bit_depth_luma_minus8,
      bit_depth_chroma_minus8,
      qpprime_y_zero_transform_bypass_flag,
      seq_scaling_matrix_present_flag,
      log2_max_frame_num_minus4: data.exponential_golomb(),
      pic_order_cnt_type: {
        pic_order_cnt_type = data.exponential_golomb();
        pic_order_cnt_type
      },
      log2_max_pic_order_cnt_lsb_minus4: (pic_order_cnt_type == 0)
        .then(|| data.exponential_golomb()),
      max_num_ref_frames: data.exponential_golomb(),
      gaps_in_frame_num_value_allowed_flag: data.bit() != 0,
      pic_width_in_mbs_minus1: data.exponential_golomb(),
      pic_height_in_map_units_minus1: data.exponential_golomb(),
      frame_mbs_only_flag: {
        frame_mbs_only_flag = data.bit() != 0;
        frame_mbs_only_flag
      },
      mb_adaptive_frame_field_flag: !frame_mbs_only_flag && data.bit() != 0,
      direct_8x8_inference_flag: data.bit() != 0,
      frame_cropping_flag: data.bit() != 0,
      vui_parameters_present_flag: data.bit() != 0,
    })
  }
}

#[derive(Debug)]
pub struct Mp4aAtom {
  pub version: u16,
  pub revision_level: u16,
  pub vendor: u32,
  pub number_of_channels: u16,
  pub sample_size: u16,
  pub compression_id: u16,
  pub packet_size: u16,
  pub sample_rate: f32,
}

impl Mp4aAtom {
  pub fn decode(mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      version: data.next_into()?,
      revision_level: data.next_into()?,
      vendor: data.next_into()?,
      number_of_channels: data.next_into()?,
      sample_size: data.next_into()?,
      compression_id: data.next_into()?,
      packet_size: data.next_into()?,
      sample_rate: data.fixed_point_16()?,
    })
  }
}
