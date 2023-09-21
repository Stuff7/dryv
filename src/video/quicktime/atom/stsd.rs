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
      sps: SequenceParameterSet::decode(data.into())?,
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
  pub separate_color_plane_flag: u8,
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
  pub frame_cropping: Option<FrameCropping>,
  pub vui_parameters: Option<VuiParameters>,
}

impl SequenceParameterSet {
  pub fn decode(mut data: AtomBitData) -> AtomResult<Self> {
    // use std::io::Write;
    // let mut img = std::fs::File::create("temp/img.264").expect("IMG CREATION");
    // let mut d = vec![0, 0, 1];
    // d.extend_from_slice(&data[2..]);
    // img.write_all(&d).expect("SAVING");
    let pic_order_cnt_type;
    let frame_mbs_only_flag;
    let profile_idc;
    let frame_cropping_flag;

    let mut chroma_format_idc = 1;
    let mut separate_color_plane_flag = 0;
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
        profile_idc = data.byte()?;
        profile_idc
      },
      constraint_set0_flag: data.bit() != 0,
      constraint_set1_flag: data.bit() != 0,
      constraint_set2_flag: data.bit() != 0,
      constraint_set3_flag: data.bit() != 0,
      constraint_set4_flag: data.bit() != 0,
      constraint_set5_flag: data.bit() != 0,
      reserved_zero_2bits: data.bits(2) != 0,
      level_idc: data.byte()?,
      id: data.exponential_golomb(),
      chroma_format_idc: {
        match profile_idc {
          100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
            chroma_format_idc = data.exponential_golomb();
            if chroma_format_idc == 3 {
              separate_color_plane_flag = data.bit();
            }
            bit_depth_luma_minus8 = data.exponential_golomb();
            bit_depth_chroma_minus8 = data.exponential_golomb();
            qpprime_y_zero_transform_bypass_flag = data.bit();
            seq_scaling_matrix_present_flag = data.bit();
            // if seq_scaling_matrix_present_flag == 1 {
            //   let end = if chroma_format_idc != 3 {8} else {12};
            //   for i in 0..end {
            //     seq_scaling_list_present_flag[ i ] 0 u(1)
            //     if( seq_scaling_list_present_flag[ i ] )
            //     if( i < 6 )
            //     scaling_list( ScalingList4x4[ i ], 16,
            //     UseDefaultScalingMatrix4x4Flag[ i ] )
            //     0
            //     else
            //     scaling_list( ScalingList8x8[ i − 6 ], 64,
            //     UseDefaultScalingMatrix8x8Flag[ i − 6 ] )
            //     0
            //   }
            // }
          }
          _ => (),
        }
        chroma_format_idc
      },
      separate_color_plane_flag,
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
      frame_cropping_flag: {
        frame_cropping_flag = data.bit() != 0;
        frame_cropping_flag
      },
      frame_cropping: frame_cropping_flag.then(|| FrameCropping::decode(&mut data)),
      vui_parameters: VuiParameters::decode(data.bit() != 0, &mut data)?,
    })
  }
}

#[derive(Debug)]
pub struct FrameCropping {
  pub left: u32,
  pub right: u32,
  pub top: u32,
  pub bottom: u32,
}

impl FrameCropping {
  pub fn decode(data: &mut AtomBitData) -> Self {
    Self {
      left: data.exponential_golomb(),
      right: data.exponential_golomb(),
      top: data.exponential_golomb(),
      bottom: data.exponential_golomb(),
    }
  }
}

#[derive(Debug)]
pub struct VuiParameters {
  pub aspect_ratio_info_present_flag: bool,
  pub aspect_ratio_idc: u8,
  pub sample_aspect_ratio: Option<SampleAspectRatio>,
  pub overscan_info_present_flag: bool,
  pub overscan_appropriate_flag: bool,
  pub video_signal_type: Option<VideoSignalType>,
  pub chroma_loc_info: Option<ChromaLocInfo>,
  pub timing_info: Option<TimingInfo>,
  pub nal_hrd_parameters: Option<HrdParameters>,
  pub vcl_hrd_parameters: Option<HrdParameters>,
  pub low_delay_hrd_flag: bool,
  pub pic_struct_present_flag: bool,
  pub bitstream_restriction: Option<BitstreamRestriction>,
}

impl VuiParameters {
  pub fn decode(
    vui_parameters_present_flag: bool,
    data: &mut AtomBitData,
  ) -> AtomResult<Option<Self>> {
    vui_parameters_present_flag
      .then(|| -> AtomResult<Self> {
        let aspect_ratio_info_present_flag;
        let aspect_ratio_idc;
        let overscan_info_present_flag;
        let nal_hrd_parameters_present_flag;
        let vcl_hrd_parameters_present_flag;
        Ok(Self {
          aspect_ratio_info_present_flag: {
            aspect_ratio_info_present_flag = data.bit() != 0;
            aspect_ratio_info_present_flag
          },
          aspect_ratio_idc: {
            aspect_ratio_idc = aspect_ratio_info_present_flag
              .then(|| data.byte())
              .transpose()?
              .unwrap_or_default();
            aspect_ratio_idc
          },
          sample_aspect_ratio: SampleAspectRatio::decode(aspect_ratio_idc, data)?,
          overscan_info_present_flag: {
            overscan_info_present_flag = data.bit() != 0;
            overscan_info_present_flag
          },
          overscan_appropriate_flag: overscan_info_present_flag && data.bit() != 0,
          video_signal_type: VideoSignalType::decode(data.bit() != 0, data)?,
          chroma_loc_info: ChromaLocInfo::decode(data.bit() != 0, data),
          timing_info: TimingInfo::decode(data.bit() != 0, data)?,
          nal_hrd_parameters: {
            nal_hrd_parameters_present_flag = data.bit() != 0;
            HrdParameters::decode(nal_hrd_parameters_present_flag, data)
          },
          vcl_hrd_parameters: {
            vcl_hrd_parameters_present_flag = data.bit() != 0;
            HrdParameters::decode(vcl_hrd_parameters_present_flag, data)
          },
          low_delay_hrd_flag: (nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag)
            && data.bit() != 0,
          pic_struct_present_flag: data.bit() != 0,
          bitstream_restriction: BitstreamRestriction::decode(data.bit() != 0, data),
        })
      })
      .transpose()
  }
}

#[derive(Debug)]
pub struct SampleAspectRatio {
  pub width: u16,
  pub height: u16,
}

impl SampleAspectRatio {
  const EXTENDED_SAR: u8 = 255;
  pub fn decode(aspect_ratio_idc: u8, data: &mut AtomBitData) -> AtomResult<Option<Self>> {
    (aspect_ratio_idc == Self::EXTENDED_SAR)
      .then(|| -> AtomResult<Self> {
        Ok(Self {
          width: data.next_into()?,
          height: data.next_into()?,
        })
      })
      .transpose()
  }
}

#[derive(Debug)]
pub struct VideoSignalType {
  pub video_format: u8,
  pub video_full_range_flag: u8,
  pub color_description: Option<ColorDescription>,
}

impl VideoSignalType {
  pub fn decode(
    video_signal_type_present_flag: bool,
    data: &mut AtomBitData,
  ) -> AtomResult<Option<Self>> {
    video_signal_type_present_flag
      .then(|| -> AtomResult<Self> {
        Ok(Self {
          video_format: data.bits(3),
          video_full_range_flag: data.bit(),
          color_description: ColorDescription::decode(data.bit() != 0, data)?,
        })
      })
      .transpose()
  }
}

#[derive(Debug)]
pub struct ColorDescription {
  pub primaries: u8,
  pub transfer_characteristics: u8,
  pub matrix_coefficients: u8,
}

impl ColorDescription {
  pub fn decode(
    color_description_present_flag: bool,
    data: &mut AtomBitData,
  ) -> AtomResult<Option<Self>> {
    color_description_present_flag
      .then(|| -> AtomResult<Self> {
        Ok(Self {
          primaries: data.byte()?,
          transfer_characteristics: data.byte()?,
          matrix_coefficients: data.byte()?,
        })
      })
      .transpose()
  }
}

#[derive(Debug)]
pub struct ChromaLocInfo {
  pub top_field: u32,
  pub bottom_field: u32,
}

impl ChromaLocInfo {
  pub fn decode(chroma_loc_info_present_flag: bool, data: &mut AtomBitData) -> Option<Self> {
    chroma_loc_info_present_flag.then(|| Self {
      top_field: data.exponential_golomb(),
      bottom_field: data.exponential_golomb(),
    })
  }
}

#[derive(Debug)]
pub struct TimingInfo {
  pub num_units_in_tick: u32,
  pub time_scale: u32,
  pub fixed_frame_rate_flag: u8,
}

impl TimingInfo {
  pub fn decode(
    timing_info_present_flag: bool,
    data: &mut AtomBitData,
  ) -> AtomResult<Option<Self>> {
    timing_info_present_flag
      .then(|| -> AtomResult<Self> {
        Ok(Self {
          num_units_in_tick: data.next_into()?,
          time_scale: data.next_into()?,
          fixed_frame_rate_flag: data.bit(),
        })
      })
      .transpose()
  }
}

#[derive(Debug)]
pub struct HrdParameters {
  pub cpb_cnt_minus1: u32,
  pub bit_rate_scale: u8,
  pub cpb_size_scale: u8,
  pub bit_rate_value_minus1: Box<[u32]>,
  pub cpb_size_value_minus1: Box<[u32]>,
  pub cbr_flag: Box<[u8]>,
  pub initial_cpb_removal_delay_length_minus1: u8,
  pub cpb_removal_delay_length_minus1: u8,
  pub dpb_output_delay_length_minus1: u8,
  pub time_offset_length: u8,
}

impl HrdParameters {
  pub fn decode(nal_hrd_parameters_present_flag: bool, data: &mut AtomBitData) -> Option<Self> {
    nal_hrd_parameters_present_flag.then(|| {
      let cpb_cnt_minus1;
      let mut bit_rate_value_minus1 = Vec::new();
      let mut cpb_size_value_minus1 = Vec::new();
      let mut cbr_flag = Vec::new();
      Self {
        cpb_cnt_minus1: {
          cpb_cnt_minus1 = data.exponential_golomb();
          cpb_cnt_minus1
        },
        bit_rate_scale: data.bits(4),
        cpb_size_scale: {
          let cpb_size_scale = data.bits(4);
          for _ in 0..=cpb_cnt_minus1 {
            bit_rate_value_minus1.push(data.exponential_golomb());
            cpb_size_value_minus1.push(data.exponential_golomb());
            cbr_flag.push(data.bit());
          }
          cpb_size_scale
        },
        bit_rate_value_minus1: bit_rate_value_minus1.into_boxed_slice(),
        cpb_size_value_minus1: cpb_size_value_minus1.into_boxed_slice(),
        cbr_flag: cbr_flag.into_boxed_slice(),
        initial_cpb_removal_delay_length_minus1: data.bits(5),
        cpb_removal_delay_length_minus1: data.bits(5),
        dpb_output_delay_length_minus1: data.bits(5),
        time_offset_length: data.bits(5),
      }
    })
  }
}

#[derive(Debug)]
pub struct BitstreamRestriction {
  pub motion_vectors_over_pic_boundaries_flag: bool,
  pub max_bytes_per_pic_denom: u32,
  pub max_bits_per_mb_denom: u32,
  pub log2_max_mv_length_horizontal: u32,
  pub log2_max_mv_length_vertical: u32,
  pub max_num_reorder_frames: u32,
  pub max_dec_frame_buffering: u32,
}

impl BitstreamRestriction {
  pub fn decode(bitstream_restriction_flag: bool, data: &mut AtomBitData) -> Option<Self> {
    bitstream_restriction_flag.then(|| Self {
      motion_vectors_over_pic_boundaries_flag: data.bit() != 0,
      max_bytes_per_pic_denom: data.exponential_golomb(),
      max_bits_per_mb_denom: data.exponential_golomb(),
      log2_max_mv_length_horizontal: data.exponential_golomb(),
      log2_max_mv_length_vertical: data.exponential_golomb(),
      max_num_reorder_frames: data.exponential_golomb(),
      max_dec_frame_buffering: data.exponential_golomb(),
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
