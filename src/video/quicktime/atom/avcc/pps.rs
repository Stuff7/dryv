use crate::byte::BitData;

#[derive(Debug)]
pub struct PictureParameterSet {
  pub length: u16,
  pub id: u16,
  pub seq_parameter_set_id: u16,
  pub entropy_coding_mode_flag: bool,
  pub bottom_field_pic_order_in_frame_present_flag: bool,
  pub slice_group: Option<SliceGroup>,
  pub num_ref_idx_10_default_active_minus1: u16,
  pub num_ref_idx_11_default_active_minus1: u16,
  pub weighted_pred_flag: bool,
  pub weighted_bipred_idc: u8,
  pub pic_init_qp_minus26: i16,
  pub pic_init_qs_minus26: i16,
  pub chroma_qp_index_offset: i16,
  pub deblocking_filter_control_present_flag: bool,
  pub constrained_intra_pred_flag: bool,
  pub redundant_pic_cnt_present_flag: bool,
}

impl PictureParameterSet {
  pub fn decode(data: &mut BitData) -> Self {
    Self {
      length: data.next_into(),
      id: data.exponential_golomb(),
      seq_parameter_set_id: data.exponential_golomb(),
      entropy_coding_mode_flag: data.bit_flag(),
      bottom_field_pic_order_in_frame_present_flag: data.bit_flag(),
      slice_group: SliceGroup::new(data.exponential_golomb(), data),
      num_ref_idx_10_default_active_minus1: data.exponential_golomb(),
      num_ref_idx_11_default_active_minus1: data.exponential_golomb(),
      weighted_pred_flag: data.bit_flag(),
      weighted_bipred_idc: data.bits_into(2),
      pic_init_qp_minus26: data.exponential_golomb(),
      pic_init_qs_minus26: data.exponential_golomb(),
      chroma_qp_index_offset: data.exponential_golomb(),
      deblocking_filter_control_present_flag: data.bit_flag(),
      constrained_intra_pred_flag: data.bit_flag(),
      redundant_pic_cnt_present_flag: data.bit_flag(),
    }
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
  pub fn new(num_slice_groups_minus1: u16, data: &mut BitData) -> Option<Self> {
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
}
