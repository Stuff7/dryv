use super::{ScalingLists, SequenceParameterSet};
use crate::byte::BitStream;

#[derive(Debug)]
pub struct PictureParameterSet {
  pub length: u16,
  pub forbidden_zero_bit: u8,
  pub nal_ref_idc: u8,
  pub nal_unit_type: u8,
  pub id: u16,
  pub seq_parameter_set_id: u16,
  pub entropy_coding_mode_flag: bool,
  pub bottom_field_pic_order_in_frame_present_flag: bool,
  pub num_slice_groups_minus1: u16,
  pub slice_group: Option<SliceGroup>,
  pub num_ref_idx_l0_default_active_minus1: u16,
  pub num_ref_idx_l1_default_active_minus1: u16,
  pub weighted_pred_flag: bool,
  pub weighted_bipred_idc: u8,
  pub pic_init_qp_minus26: i16,
  pub pic_init_qs_minus26: i16,
  pub chroma_qp_index_offset: i16,
  pub deblocking_filter_control_present_flag: bool,
  pub constrained_intra_pred_flag: bool,
  pub redundant_pic_cnt_present_flag: bool,
  pub extra_rbsp_data: Option<ExtraRbspData>,
}

impl PictureParameterSet {
  pub fn decode(data: &mut BitStream, chroma_format_idc: u16) -> Self {
    let num_slice_groups_minus1;
    Self {
      length: data.next_into(),
      forbidden_zero_bit: data.bit(),
      nal_ref_idc: data.bits(2),
      nal_unit_type: data.bits(5),
      id: data.exponential_golomb(),
      seq_parameter_set_id: data.exponential_golomb(),
      entropy_coding_mode_flag: data.bit_flag(),
      bottom_field_pic_order_in_frame_present_flag: data.bit_flag(),
      num_slice_groups_minus1: {
        num_slice_groups_minus1 = data.exponential_golomb();
        num_slice_groups_minus1
      },
      slice_group: SliceGroup::new(num_slice_groups_minus1, data),
      num_ref_idx_l0_default_active_minus1: data.exponential_golomb(),
      num_ref_idx_l1_default_active_minus1: data.exponential_golomb(),
      weighted_pred_flag: data.bit_flag(),
      weighted_bipred_idc: data.bits_into(2),
      pic_init_qp_minus26: data.signed_exponential_golomb(),
      pic_init_qs_minus26: data.signed_exponential_golomb(),
      chroma_qp_index_offset: data.signed_exponential_golomb(),
      deblocking_filter_control_present_flag: data.bit_flag(),
      constrained_intra_pred_flag: data.bit_flag(),
      redundant_pic_cnt_present_flag: data.bit_flag(),
      extra_rbsp_data: ExtraRbspData::new(data, chroma_format_idc),
    }
  }
}

#[derive(Debug)]
pub struct ExtraRbspData {
  pub transform_8x8_mode_flag: bool,
  pub pic_scaling_matrix: Option<ScalingLists>,
  pub second_chroma_qp_index_offset: i16,
}

impl ExtraRbspData {
  pub fn new(data: &mut BitStream, chroma_format_idc: u16) -> Option<Self> {
    data.has_bits().then(|| {
      let transform_8x8_mode;
      Self {
        transform_8x8_mode_flag: {
          transform_8x8_mode = data.bit();
          transform_8x8_mode != 0
        },
        pic_scaling_matrix: ScalingLists::new(
          data.bit_flag(),
          data,
          6 + if chroma_format_idc != 3 { 2 } else { 6 } * transform_8x8_mode,
        ),
        second_chroma_qp_index_offset: data.signed_exponential_golomb(),
      }
    })
  }
}

#[derive(Debug)]
pub enum SliceGroup {
  Unknown(u16),
  Interleaved {
    run_length_minus1: Box<[u16]>,
  },
  Dispersed,
  ForegroundWithLeftOver {
    top_left: Box<[u16]>,
    bottom_right: Box<[u16]>,
  },
  Change {
    map_type: u16,
    change_direction_flag: bool,
    change_rate_minus1: u16,
  },
  Explicit {
    pic_size_in_map_units_minus1: u16,
    id: Box<[u8]>,
  },
}

impl SliceGroup {
  pub fn new(num_slice_groups_minus1: u16, data: &mut BitStream) -> Option<Self> {
    (num_slice_groups_minus1 > 0).then(|| match data.exponential_golomb() {
      0 => Self::Interleaved {
        run_length_minus1: (0..num_slice_groups_minus1)
          .map(|_| data.exponential_golomb())
          .collect(),
      },
      1 => Self::Dispersed,
      2 => {
        let mut top_left = Vec::with_capacity(num_slice_groups_minus1 as usize);
        let mut bottom_right = Vec::with_capacity(num_slice_groups_minus1 as usize);
        for _ in 0..num_slice_groups_minus1 {
          top_left.push(data.exponential_golomb());
          bottom_right.push(data.exponential_golomb());
        }
        Self::ForegroundWithLeftOver {
          top_left: top_left.into_boxed_slice(),
          bottom_right: bottom_right.into_boxed_slice(),
        }
      }
      n if n == 3 || n == 4 || n == 5 => Self::Change {
        map_type: n,
        change_direction_flag: data.bit_flag(),
        change_rate_minus1: data.exponential_golomb(),
      },
      6 => Self::Explicit {
        pic_size_in_map_units_minus1: data.exponential_golomb(),
        id: (0..num_slice_groups_minus1).map(|_| data.bit()).collect(),
      },
      n => Self::Unknown(n),
    })
  }

  /// 8.2.2 - 8.2.2.7 Decoding process for macroblock to slice group map
  pub fn init_sgmap(
    slice_group_change_cycle: u16,
    pic_width_in_mbs: u16,
    sps: &SequenceParameterSet,
    pps: &PictureParameterSet,
  ) -> Box<[u8]> {
    let Some(ref slice_group) = pps.slice_group else {
      return [].into();
    };
    let width = sps.pic_width_in_mbs_minus1 as isize + 1;
    let height = sps.pic_height_in_map_units_minus1 as isize + 1;
    let num = pps.num_slice_groups_minus1 as isize + 1;
    let (change_direction_flag, change_rate_minus1) = match slice_group {
      SliceGroup::Change {
        change_direction_flag,
        change_rate_minus1,
        ..
      } => (*change_direction_flag, *change_rate_minus1),
      _ => (false, 0),
    };
    let mut musg0 = (slice_group_change_cycle * (change_rate_minus1 + 1)) as isize;
    if musg0 > width * height {
      musg0 = width * height;
    }
    let sulg = if change_direction_flag {
      width * height - musg0
    } else {
      musg0
    };

    let mut k = 0isize;
    let mut j = 0isize;
    let mut sgmap = (0..width * height)
      .map(|i| {
        let x = i % width;
        let y = i / width;
        match slice_group {
          SliceGroup::Interleaved { run_length_minus1 } => {
            let ret = j;
            if k == run_length_minus1[j as usize] as isize {
              k = 0;
              j += 1;
              j %= num;
            } else {
              k += 1;
            }
            ret as u8
          }
          SliceGroup::Dispersed => ((x + ((y * num) / 2)) % num) as u8,
          SliceGroup::ForegroundWithLeftOver {
            top_left,
            bottom_right,
          } => {
            let mut ret = num - 1;
            j = num - 2;
            while j >= 0 {
              let xtl = (top_left[j as usize] % pic_width_in_mbs) as isize;
              let ytl = (top_left[j as usize] / pic_width_in_mbs) as isize;
              let xbr = (bottom_right[j as usize] % pic_width_in_mbs) as isize;
              let ybr = (bottom_right[j as usize] / pic_width_in_mbs) as isize;
              if x >= xtl && x <= xbr && y >= ytl && y <= ybr {
                ret = j;
              }
              j -= 1;
            }
            ret as u8
          }
          SliceGroup::Change {
            map_type,
            change_direction_flag,
            ..
          } => {
            let change_direction_flag = *change_direction_flag as isize;
            (match map_type {
              3 => 1,
              4 => change_direction_flag ^ (i >= sulg) as isize,
              _ => {
                k = x * height + y;
                change_direction_flag ^ (k >= sulg) as isize
              }
            }) as u8
          }
          SliceGroup::Explicit {
            pic_size_in_map_units_minus1,
            id,
          } => {
            if width * height != *pic_size_in_map_units_minus1 as isize + 1 {
              panic!("pic_size_in_map_units_minus1 mismatch!");
            }
            id[i as usize]
          }
          Self::Unknown(n) => panic!("Unknown slice group map {n}"),
        }
      })
      .collect::<Box<_>>();
    if let SliceGroup::Change {
      map_type,
      change_direction_flag,
      ..
    } = slice_group
    {
      if *map_type == 3 {
        let cdf = *change_direction_flag as isize;
        let width = width;
        let height = height;
        let mut x = (width - cdf) / 2;
        let mut y = (height - cdf) / 2;
        let mut xmin = x;
        let mut xmax = x;
        let mut ymin = y;
        let mut ymax = y;
        let mut xdir = cdf - 1;
        let mut ydir = cdf;
        let mut muv;
        while k < musg0 {
          muv = sgmap[(y * width + x) as usize];
          sgmap[(y * width + x) as usize] = 0;
          if xdir == -1 && x == xmin {
            if xmin != 0 {
              xmin -= 1;
            }
            x = xmin;
            xdir = 0;
            ydir = 2 * cdf - 1;
          } else if xdir == 1 && x == xmax {
            if xmax != width - 1 {
              xmax += 1;
            }
            x = xmax;
            xdir = 0;
            ydir = 1 - 2 * cdf;
          } else if ydir == -1 && y == ymin {
            if ymin != 0 {
              ymin -= 1;
            }
            y = ymin;
            xdir = 1 - 2 * cdf;
            ydir = 0;
          } else if ydir == 1 && y == ymax {
            if ymax != height - 1 {
              ymax += 1;
            }
            y = ymax;
            xdir = 2 * cdf - 1;
            ydir = 0;
          } else {
            x += xdir;
            y += ydir;
            k += muv as isize;
          }
        }
      }
    }
    sgmap
  }
}
