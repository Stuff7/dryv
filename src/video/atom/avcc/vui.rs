use crate::byte::BitStream;

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
  pub fn decode(vui_parameters_present_flag: bool, data: &mut BitStream) -> Option<Self> {
    vui_parameters_present_flag.then(|| {
      let aspect_ratio_info_present_flag;
      let aspect_ratio_idc;
      let overscan_info_present_flag;
      let nal_hrd_parameters_present_flag;
      let vcl_hrd_parameters_present_flag;
      Self {
        aspect_ratio_info_present_flag: {
          aspect_ratio_info_present_flag = data.bit_flag();
          aspect_ratio_info_present_flag
        },
        aspect_ratio_idc: {
          aspect_ratio_idc = aspect_ratio_info_present_flag
            .then(|| data.byte())
            .unwrap_or_default();
          aspect_ratio_idc
        },
        sample_aspect_ratio: SampleAspectRatio::decode(aspect_ratio_idc, data),
        overscan_info_present_flag: {
          overscan_info_present_flag = data.bit_flag();
          overscan_info_present_flag
        },
        overscan_appropriate_flag: overscan_info_present_flag && data.bit_flag(),
        video_signal_type: VideoSignalType::decode(data.bit_flag(), data),
        chroma_loc_info: ChromaLocInfo::decode(data.bit_flag(), data),
        timing_info: TimingInfo::decode(data.bit_flag(), data),
        nal_hrd_parameters: {
          nal_hrd_parameters_present_flag = data.bit_flag();
          HrdParameters::decode(nal_hrd_parameters_present_flag, data)
        },
        vcl_hrd_parameters: {
          vcl_hrd_parameters_present_flag = data.bit_flag();
          HrdParameters::decode(vcl_hrd_parameters_present_flag, data)
        },
        low_delay_hrd_flag: (nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag)
          && data.bit_flag(),
        pic_struct_present_flag: data.bit_flag(),
        bitstream_restriction: BitstreamRestriction::decode(data.bit_flag(), data),
      }
    })
  }
}

#[derive(Debug)]
pub struct SampleAspectRatio {
  pub width: u16,
  pub height: u16,
}

impl SampleAspectRatio {
  const EXTENDED_SAR: u8 = 255;
  pub fn decode(aspect_ratio_idc: u8, data: &mut BitStream) -> Option<Self> {
    (aspect_ratio_idc == Self::EXTENDED_SAR).then(|| Self {
      width: data.next_into(),
      height: data.next_into(),
    })
  }
}

#[derive(Debug)]
pub struct VideoSignalType {
  pub video_format: u8,
  pub video_full_range_flag: bool,
  pub color_description: Option<ColorDescription>,
}

impl VideoSignalType {
  pub fn decode(video_signal_type_present_flag: bool, data: &mut BitStream) -> Option<Self> {
    video_signal_type_present_flag.then(|| Self {
      video_format: data.bits(3),
      video_full_range_flag: data.bit_flag(),
      color_description: ColorDescription::decode(data.bit_flag(), data),
    })
  }
}

#[derive(Debug)]
pub struct ColorDescription {
  pub primaries: u8,
  pub transfer_characteristics: u8,
  pub matrix_coefficients: u8,
}

impl ColorDescription {
  pub fn decode(color_description_present_flag: bool, data: &mut BitStream) -> Option<Self> {
    color_description_present_flag.then(|| Self {
      primaries: data.byte(),
      transfer_characteristics: data.byte(),
      matrix_coefficients: data.byte(),
    })
  }
}

#[derive(Debug)]
pub struct ChromaLocInfo {
  pub top_field: u16,
  pub bottom_field: u16,
}

impl ChromaLocInfo {
  pub fn decode(chroma_loc_info_present_flag: bool, data: &mut BitStream) -> Option<Self> {
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
  pub fixed_frame_rate_flag: bool,
}

impl TimingInfo {
  pub fn decode(timing_info_present_flag: bool, data: &mut BitStream) -> Option<Self> {
    timing_info_present_flag.then(|| Self {
      num_units_in_tick: data.next_into(),
      time_scale: data.next_into(),
      fixed_frame_rate_flag: data.bit_flag(),
    })
  }
}

#[derive(Debug)]
pub struct HrdParameters {
  pub cpb_cnt_minus1: u16,
  pub bit_rate_scale: u8,
  pub cpb_size_scale: u8,
  pub bit_rate_value_minus1: Box<[u16]>,
  pub cpb_size_value_minus1: Box<[u16]>,
  pub cbr_flag: Box<[bool]>,
  pub initial_cpb_removal_delay_length_minus1: u8,
  pub cpb_removal_delay_length_minus1: u8,
  pub dpb_output_delay_length_minus1: u8,
  pub time_offset_length: u8,
}

impl HrdParameters {
  pub fn decode(nal_hrd_parameters_present_flag: bool, data: &mut BitStream) -> Option<Self> {
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
            cbr_flag.push(data.bit_flag());
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
  pub max_bytes_per_pic_denom: u16,
  pub max_bits_per_mb_denom: u16,
  pub log2_max_mv_length_horizontal: u16,
  pub log2_max_mv_length_vertical: u16,
  pub max_num_reorder_frames: u16,
  pub max_dec_frame_buffering: u16,
}

impl BitstreamRestriction {
  pub fn decode(bitstream_restriction_flag: bool, data: &mut BitStream) -> Option<Self> {
    bitstream_restriction_flag.then(|| Self {
      motion_vectors_over_pic_boundaries_flag: data.bit_flag(),
      max_bytes_per_pic_denom: data.exponential_golomb(),
      max_bits_per_mb_denom: data.exponential_golomb(),
      log2_max_mv_length_horizontal: data.exponential_golomb(),
      log2_max_mv_length_vertical: data.exponential_golomb(),
      max_num_reorder_frames: data.exponential_golomb(),
      max_dec_frame_buffering: data.exponential_golomb(),
    })
  }
}
