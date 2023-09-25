use super::*;

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
  pub id: u16,
  pub chroma_format_idc: u16,
  pub separate_color_plane_flag: u8,
  pub bit_depth_luma_minus8: u16,
  pub bit_depth_chroma_minus8: u16,
  pub qpprime_y_zero_transform_bypass_flag: u8,
  pub seq_scaling_matrix_present_flag: u8,
  pub scaling_list_4x4: Option<Box<[ScalingList<16>]>>,
  pub scaling_list_16x16: Option<Box<[ScalingList<64>]>>,
  pub log2_max_frame_num_minus4: u16,
  pub pic_order_cnt_type: u16,
  pub log2_max_pic_order_cnt_lsb_minus4: Option<u16>,
  pub max_num_ref_frames: u16,
  pub gaps_in_frame_num_value_allowed_flag: bool,
  pub pic_width_in_mbs_minus1: u16,
  pub pic_height_in_map_units_minus1: u16,
  pub frame_mbs_only_flag: bool,
  pub mb_adaptive_frame_field_flag: bool,
  pub direct_8x8_inference_flag: bool,
  pub frame_cropping: Option<FrameCropping>,
  pub vui_parameters: Option<VuiParameters>,
}

impl SequenceParameterSet {
  pub fn decode(data: &mut AtomBitData) -> AtomResult<Self> {
    let pic_order_cnt_type;
    let frame_mbs_only_flag;
    let profile_idc;

    let mut chroma_format_idc = 1;
    let mut separate_color_plane_flag = 0;
    let mut bit_depth_luma_minus8 = 0;
    let mut bit_depth_chroma_minus8 = 0;
    let mut qpprime_y_zero_transform_bypass_flag = 0;
    let mut seq_scaling_matrix_present_flag = 0;
    let mut scaling_list_4x4 = None;
    let mut scaling_list_16x16 = None;
    Ok(Self {
      length: data.next_into()?,
      forbidden_zero_bit: data.bit(),
      nal_ref_idc: data.bits(2),
      nal_unit_type: data.bits(5),
      profile_idc: {
        profile_idc = data.byte()?;
        profile_idc
      },
      constraint_set0_flag: data.bit_flag(),
      constraint_set1_flag: data.bit_flag(),
      constraint_set2_flag: data.bit_flag(),
      constraint_set3_flag: data.bit_flag(),
      constraint_set4_flag: data.bit_flag(),
      constraint_set5_flag: data.bit_flag(),
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
            if seq_scaling_matrix_present_flag == 1 {
              let size = if chroma_format_idc != 3 { 8 } else { 12 };
              scaling_list_4x4 = Some(
                (0..6)
                  .filter_map(|_| (data.bit() == 1).then(|| ScalingList::new(data)))
                  .collect(),
              );
              scaling_list_16x16 = Some(
                (6..size)
                  .filter_map(|_| (data.bit() == 1).then(|| ScalingList::new(data)))
                  .collect(),
              );
            }
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
      scaling_list_4x4,
      scaling_list_16x16,
      log2_max_frame_num_minus4: data.exponential_golomb(),
      pic_order_cnt_type: {
        pic_order_cnt_type = data.exponential_golomb();
        pic_order_cnt_type
      },
      log2_max_pic_order_cnt_lsb_minus4: (pic_order_cnt_type == 0)
        .then(|| data.exponential_golomb()),
      max_num_ref_frames: data.exponential_golomb(),
      gaps_in_frame_num_value_allowed_flag: data.bit_flag(),
      pic_width_in_mbs_minus1: data.exponential_golomb(),
      pic_height_in_map_units_minus1: data.exponential_golomb(),
      frame_mbs_only_flag: {
        frame_mbs_only_flag = data.bit_flag();
        frame_mbs_only_flag
      },
      mb_adaptive_frame_field_flag: !frame_mbs_only_flag && data.bit_flag(),
      direct_8x8_inference_flag: data.bit_flag(),
      frame_cropping: FrameCropping::decode(data.bit_flag(), data),
      vui_parameters: {
        let vui = VuiParameters::decode(data.bit_flag(), data)?;
        if data.bit() == 1 {
          data.skip_trailing_bits();
        }
        vui
      },
    })
  }
}

#[derive(Debug)]
pub struct ScalingList<const S: usize> {
  pub data: [i16; S],
  pub use_default_scaling_matrix_flag: bool,
}

impl<const S: usize> ScalingList<S> {
  pub fn new(bits: &mut AtomBitData) -> Self {
    let mut data = [0; S];
    let mut use_default_scaling_matrix_flag = false;
    let mut last_scale = 8;
    let mut next_scale = 8;
    for (i, scale) in data.iter_mut().enumerate() {
      if next_scale != 0 {
        let delta_scale: i16 = bits.exponential_golomb();
        next_scale = (last_scale + delta_scale + 256) % 256;
        use_default_scaling_matrix_flag = i == 0 && next_scale == 0;
      }
      *scale = if next_scale == 0 {
        last_scale
      } else {
        next_scale
      };
      last_scale = *scale;
    }
    Self {
      data,
      use_default_scaling_matrix_flag,
    }
  }
}

#[derive(Debug)]
pub struct FrameCropping {
  pub left: u16,
  pub right: u16,
  pub top: u16,
  pub bottom: u16,
}

impl FrameCropping {
  pub fn decode(frame_cropping_flag: bool, data: &mut AtomBitData) -> Option<Self> {
    frame_cropping_flag.then(|| Self {
      left: data.exponential_golomb(),
      right: data.exponential_golomb(),
      top: data.exponential_golomb(),
      bottom: data.exponential_golomb(),
    })
  }
}
