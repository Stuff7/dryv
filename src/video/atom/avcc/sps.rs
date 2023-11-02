use super::*;
use crate::byte::BitStream;

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
  pub separate_color_plane_flag: bool,
  pub bit_depth_luma_minus8: u16,
  pub bit_depth_chroma_minus8: u16,
  pub qpprime_y_zero_transform_bypass_flag: bool,
  pub seq_scaling_matrix: Option<ScalingLists>,
  pub log2_max_frame_num_minus4: u16,
  pub pic_order_cnt_type: u16,
  pub log2_max_pic_order_cnt_lsb_minus4: Option<u16>,
  pub pic_order_cnt_type_one: Option<PicOrderCntTypeOne>,
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
  pub fn decode(data: &mut BitStream) -> Self {
    let pic_order_cnt_type;
    let frame_mbs_only_flag;
    let profile_idc;

    let mut chroma_format_idc = 1;
    let mut separate_color_plane_flag = false;
    let mut bit_depth_luma_minus8 = 0;
    let mut bit_depth_chroma_minus8 = 0;
    let mut qpprime_y_zero_transform_bypass_flag = false;
    Self {
      length: data.next_into(),
      forbidden_zero_bit: data.bit(),
      nal_ref_idc: data.bits(2),
      nal_unit_type: data.bits(5),
      profile_idc: {
        profile_idc = data.byte();
        profile_idc
      },
      constraint_set0_flag: data.bit_flag(),
      constraint_set1_flag: data.bit_flag(),
      constraint_set2_flag: data.bit_flag(),
      constraint_set3_flag: data.bit_flag(),
      constraint_set4_flag: data.bit_flag(),
      constraint_set5_flag: data.bit_flag(),
      reserved_zero_2bits: data.bits(2) != 0,
      level_idc: data.byte(),
      id: data.exponential_golomb(),
      chroma_format_idc: {
        match profile_idc {
          100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135 => {
            chroma_format_idc = data.exponential_golomb();
            if chroma_format_idc == 3 {
              separate_color_plane_flag = data.bit_flag();
            }
            bit_depth_luma_minus8 = data.exponential_golomb();
            bit_depth_chroma_minus8 = data.exponential_golomb();
            qpprime_y_zero_transform_bypass_flag = data.bit_flag();
          }
          _ => (),
        }
        chroma_format_idc
      },
      separate_color_plane_flag,
      bit_depth_luma_minus8,
      bit_depth_chroma_minus8,
      qpprime_y_zero_transform_bypass_flag,
      seq_scaling_matrix: match profile_idc {
        66 | 77 | 88 => false,
        _ => data.bit_flag(),
      }
      .then(|| ScalingLists::new(data, if chroma_format_idc != 3 { 8 } else { 12 })),
      log2_max_frame_num_minus4: data.exponential_golomb(),
      pic_order_cnt_type: {
        pic_order_cnt_type = data.exponential_golomb();
        pic_order_cnt_type
      },
      log2_max_pic_order_cnt_lsb_minus4: (pic_order_cnt_type == 0)
        .then(|| data.exponential_golomb()),
      pic_order_cnt_type_one: PicOrderCntTypeOne::new(data, pic_order_cnt_type),
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
        let vui = VuiParameters::decode(data.bit_flag(), data);
        if data.bit() == 1 {
          data.skip_trailing_bits();
        }
        vui
      },
    }
  }
}

#[derive(Debug)]
pub struct PicOrderCntTypeOne {
  pub delta_pic_order_always_zero_flag: bool,
  pub offset_for_non_ref_pic: i16,
  pub offset_for_top_to_bottom_field: i16,
  pub num_ref_frames_in_pic_order_cnt_cycle: u16,
  pub offset_for_ref_frame: Box<[i16]>,
  pub expected_delta_per_pic_order_cnt_cycle: i16,
}

impl PicOrderCntTypeOne {
  pub fn new(data: &mut BitStream, pic_order_cnt_type: u16) -> Option<Self> {
    (pic_order_cnt_type == 1).then(|| {
      let num_ref_frames_in_pic_order_cnt_cycle;
      let offset_for_ref_frame: Box<[i16]>;
      let expected_delta_per_pic_order_cnt_cycle;
      Self {
        delta_pic_order_always_zero_flag: data.bit_flag(),
        offset_for_non_ref_pic: data.signed_exponential_golomb(),
        offset_for_top_to_bottom_field: data.signed_exponential_golomb(),
        num_ref_frames_in_pic_order_cnt_cycle: {
          num_ref_frames_in_pic_order_cnt_cycle = data.exponential_golomb();
          num_ref_frames_in_pic_order_cnt_cycle
        },
        offset_for_ref_frame: {
          offset_for_ref_frame = (0..num_ref_frames_in_pic_order_cnt_cycle)
            .map(|_| data.signed_exponential_golomb())
            .collect();
          expected_delta_per_pic_order_cnt_cycle = offset_for_ref_frame.iter().sum();
          offset_for_ref_frame
        },
        expected_delta_per_pic_order_cnt_cycle,
      }
    })
  }
}

const DEFAULT_4X4_INTRA: [i16; 16] = [
  6, 13, 13, 20, 20, 20, 28, 28, 28, 28, 32, 32, 32, 37, 37, 42,
];
const DEFAULT_4X4_INTER: [i16; 16] = [
  10, 14, 14, 20, 20, 20, 24, 24, 24, 24, 27, 27, 27, 30, 30, 34,
];

const DEFAULT_8X8_INTRA: [i16; 64] = [
  6, 10, 10, 13, 11, 13, 16, 16, 16, 16, 18, 18, 18, 18, 18, 23, 23, 23, 23, 23, 23, 25, 25, 25,
  25, 25, 25, 25, 27, 27, 27, 27, 27, 27, 27, 27, 29, 29, 29, 29, 29, 29, 29, 31, 31, 31, 31, 31,
  31, 33, 33, 33, 33, 33, 36, 36, 36, 36, 38, 38, 38, 40, 40, 42,
];
const DEFAULT_8X8_INTER: [i16; 64] = [
  9, 13, 13, 15, 13, 15, 17, 17, 17, 17, 19, 19, 19, 19, 19, 21, 21, 21, 21, 21, 21, 22, 22, 22,
  22, 22, 22, 22, 24, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 27, 27, 27, 27, 27,
  27, 28, 28, 28, 28, 28, 30, 30, 30, 30, 32, 32, 32, 33, 33, 35,
];

pub fn scaling_list<const S: usize>(bits: &mut BitStream, data: &mut [i16; S]) -> bool {
  let mut use_default_scaling_matrix_flag = false;
  let mut last_scale = 8;
  let mut next_scale = 8;
  for (i, scale) in data.iter_mut().enumerate() {
    if next_scale != 0 {
      let delta_scale: i16 = bits.signed_exponential_golomb();
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

  use_default_scaling_matrix_flag
}

#[derive(Debug)]
pub struct ScalingLists {
  pub l4x4: [[i16; 16]; 6],
  pub l8x8: Box<[[i16; 64]]>,
}

impl ScalingLists {
  pub fn new(bits: &mut BitStream, length: usize) -> Self {
    let mut scaling_list4x4 = [[0; 16]; 6];
    let mut scaling_list8x8 = vec![[0; 64]; length - 6];
    for i in 0..length {
      let scaling_list_present_flag = bits.bit_flag();
      if scaling_list_present_flag {
        if i < 6 {
          let use_default_scaling_matrix_4x4_flag = scaling_list(bits, &mut scaling_list4x4[i]);
          if use_default_scaling_matrix_4x4_flag {
            match i {
              0 | 1 | 2 => scaling_list4x4[i] = DEFAULT_4X4_INTRA,
              _ => scaling_list4x4[i] = DEFAULT_4X4_INTER,
            }
          }
        } else {
          let use_default_scaling_matrix_8x8_flag = scaling_list(bits, &mut scaling_list8x8[i - 6]);
          if use_default_scaling_matrix_8x8_flag {
            match i {
              6 | 8 | 10 => scaling_list8x8[i - 6] = DEFAULT_8X8_INTRA,
              _ => scaling_list8x8[i - 6] = DEFAULT_8X8_INTER,
            }
          }
        }
      } else if i < 6 {
        match i {
          0 | 1 | 2 => scaling_list4x4[i] = DEFAULT_4X4_INTRA,
          _ => scaling_list4x4[i] = DEFAULT_4X4_INTER,
        }
      } else {
        match i {
          6 | 8 | 10 => scaling_list8x8[i - 6] = DEFAULT_8X8_INTRA,
          7 | 9 | 11 => scaling_list8x8[i - 6] = DEFAULT_8X8_INTER,
          n => panic!("Scaling list 8x8 out of bounds {n}"),
        }
      }
    }

    Self {
      l4x4: scaling_list4x4,
      l8x8: scaling_list8x8.into(),
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
  pub fn decode(frame_cropping_flag: bool, data: &mut BitStream) -> Option<Self> {
    frame_cropping_flag.then(|| Self {
      left: data.exponential_golomb(),
      right: data.exponential_golomb(),
      top: data.exponential_golomb(),
      bottom: data.exponential_golomb(),
    })
  }
}
