use crate::{
  byte::BitStream,
  video::{
    atom::{PictureParameterSet, SequenceParameterSet, SliceGroup},
    sample::{NALUnit, NALUnitType},
  },
};

#[derive(Debug)]
pub struct SliceHeader {
  pub first_mb_in_slice: u16,
  pub slice_type: SliceType,
  pub pps_id: u16,
  pub color_plane_id: Option<u8>,
  pub frame_num: u16,
  pub field_pic_flag: bool,
  pub bottom_field_flag: bool,
  pub idr_pic_id: Option<u16>,
  pub pic_order_cnt_lsb: Option<u16>,
  pub delta_pic_order_cnt_bottom: Option<i16>,
  pub delta_pic_order_cnt: Option<(i16, Option<i16>)>,
  pub redundant_pic_cnt: Option<u16>,
  pub direct_spatial_mv_pref_flag: bool,
  pub num_ref_idx_active_override_flag: bool,
  pub num_ref_idx_l0_active_minus1: Option<u16>,
  pub num_ref_idx_l1_active_minus1: Option<u16>,
  pub ref_pic_list_mvc_modification: Option<RefPicListMvcModification>,
  pub ref_pic_list_modification: Option<RefPicListModification>,
  pub pred_weight_table: Option<PredWeightTable>,
  pub dec_ref_pic_marking: Option<DecRefPicMarking>,
  pub cabac_init_idc: Option<u16>,
  pub slice_qp_delta: i16,
  pub sp_for_switch_flag: bool,
  pub slice_qs_delta: Option<i16>,
  pub deblocking_filter_control: Option<DeblockingFilterControl>,
  pub slice_group_change_cycle: Option<u16>,
}

impl SliceHeader {
  pub fn new(
    data: &mut BitStream,
    nal: &NALUnit,
    sps: &SequenceParameterSet,
    pps: &PictureParameterSet,
  ) -> Self {
    let mut field_pic_flag = false;
    let slice_type;
    let num_ref_idx_active_override_flag;
    Self {
      first_mb_in_slice: data.exponential_golomb(),
      slice_type: {
        slice_type = SliceType::new(data.exponential_golomb());
        slice_type
      },
      pps_id: data.exponential_golomb(),
      color_plane_id: sps.separate_color_plane_flag.then(|| data.bits_into(2)),
      frame_num: data.bits_into(sps.log2_max_frame_num_minus4 as usize + 4),
      field_pic_flag: !sps.frame_mbs_only_flag && {
        field_pic_flag = data.bit_flag();
        field_pic_flag
      },
      bottom_field_flag: field_pic_flag && data.bit_flag(),
      idr_pic_id: nal.unit_type.is_idr().then(|| data.exponential_golomb()),
      pic_order_cnt_lsb: sps
        .log2_max_pic_order_cnt_lsb_minus4
        .map(|size| data.bits_into(size as usize + 4)),
      delta_pic_order_cnt_bottom: (sps.log2_max_pic_order_cnt_lsb_minus4.is_some()
        && pps.bottom_field_pic_order_in_frame_present_flag
        && !field_pic_flag)
        .then(|| data.signed_exponential_golomb()),
      delta_pic_order_cnt: sps
        .pic_order_cnt_type_one
        .as_ref()
        .is_some_and(|type_one| !type_one.delta_pic_order_always_zero_flag)
        .then(|| data.signed_exponential_golomb())
        .map(|a| {
          (
            a,
            (pps.bottom_field_pic_order_in_frame_present_flag && !field_pic_flag)
              .then(|| data.signed_exponential_golomb()),
          )
        }),
      redundant_pic_cnt: pps
        .redundant_pic_cnt_present_flag
        .then(|| data.exponential_golomb()),
      direct_spatial_mv_pref_flag: matches!(slice_type, SliceType::B) && data.bit_flag(),
      num_ref_idx_active_override_flag: {
        num_ref_idx_active_override_flag = match slice_type {
          SliceType::P | SliceType::SP | SliceType::B => data.bit_flag(),
          _ => false,
        };
        num_ref_idx_active_override_flag
      },
      num_ref_idx_l0_active_minus1: num_ref_idx_active_override_flag
        .then(|| data.exponential_golomb()),
      num_ref_idx_l1_active_minus1: (num_ref_idx_active_override_flag
        && matches!(slice_type, SliceType::B))
      .then(|| data.exponential_golomb()),
      ref_pic_list_mvc_modification: RefPicListMvcModification::new(data, &nal.unit_type),
      ref_pic_list_modification: RefPicListModification::new(data, &nal.unit_type, &slice_type),
      pred_weight_table: PredWeightTable::new(
        data,
        pps.weighted_pred_flag,
        &slice_type,
        pps.weighted_bipred_idc,
      ),
      dec_ref_pic_marking: DecRefPicMarking::new(data, nal),
      cabac_init_idc: (pps.entropy_coding_mode_flag
        && !matches!(slice_type, SliceType::I | SliceType::SI))
      .then(|| data.exponential_golomb()),
      slice_qp_delta: data.signed_exponential_golomb(),
      sp_for_switch_flag: matches!(slice_type, SliceType::SP) && data.bit_flag(),
      slice_qs_delta: matches!(slice_type, SliceType::SP | SliceType::I)
        .then(|| data.signed_exponential_golomb()),
      deblocking_filter_control: DeblockingFilterControl::new(
        data,
        pps.deblocking_filter_control_present_flag,
      ),
      slice_group_change_cycle: pps.slice_group.as_ref().and_then(|group| {
        let SliceGroup::Change { change_rate_minus1, .. } = group else {return None};
        let pic_width_in_mbs = sps.pic_width_in_mbs_minus1 + 1;
        let pic_height_in_map_units = sps.pic_height_in_map_units_minus1 + 1;
        let pic_size_in_map_units = pic_width_in_mbs * pic_height_in_map_units;
        let change_rate = change_rate_minus1 + 1;
        let size = (pic_size_in_map_units as f32 / change_rate as f32 + 1.)
          .log2()
          .ceil();
        Some(data.bits_into(size as usize))
      }),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SliceType {
  P,
  B,
  I,
  SP,
  SI,
  Unknown,
}

impl SliceType {
  pub fn new(slice_type: u8) -> Self {
    match slice_type {
      0 | 5 => Self::P,
      1 | 6 => Self::B,
      2 | 7 => Self::I,
      3 | 8 => Self::SP,
      4 | 9 => Self::SI,
      _ => Self::Unknown,
    }
  }

  pub fn is_intra(&self) -> bool {
    matches!(self, SliceType::I | SliceType::SI)
  }
}

#[derive(Debug)]
pub struct RefPicListModification {
  pub ref_pic_list_modification_flag_l0: bool,
  pub ref_pic_list_modification_flag_l1: bool,
  pub modification_of_pic_nums_idc: Option<u16>,
  pub abs_diff_pic_num_minus1: Option<u16>,
  pub long_term_pic_num: Option<u16>,
}

impl RefPicListModification {
  pub fn new(data: &mut BitStream, nal_type: &NALUnitType, slice_type: &SliceType) -> Option<Self> {
    (!matches!(
      nal_type,
      NALUnitType::CodedSliceExtension | NALUnitType::DepthOrTextureViewComponent
    ))
    .then(|| {
      let mut list = Self {
        ref_pic_list_modification_flag_l0: false,
        ref_pic_list_modification_flag_l1: false,
        modification_of_pic_nums_idc: None,
        abs_diff_pic_num_minus1: None,
        long_term_pic_num: None,
      };
      if !matches!(slice_type, SliceType::I | SliceType::SI) {
        list.ref_pic_list_modification_flag_l0 = list.create(data)
      }
      if matches!(slice_type, SliceType::B) {
        list.ref_pic_list_modification_flag_l1 = list.create(data)
      }
      list
    })
  }

  fn create(&mut self, data: &mut BitStream) -> bool {
    let ref_pic_list_modification_flag = data.bit_flag();
    if ref_pic_list_modification_flag {
      loop {
        let pic_nums_idc = data.exponential_golomb();
        match pic_nums_idc {
          0 | 1 => self.abs_diff_pic_num_minus1 = Some(data.exponential_golomb()),
          2 => self.long_term_pic_num = Some(data.exponential_golomb()),
          _ => (),
        }
        self.modification_of_pic_nums_idc = Some(pic_nums_idc);
        if pic_nums_idc == 3 {
          break;
        }
      }
    }
    ref_pic_list_modification_flag
  }
}

#[derive(Debug)]
pub struct RefPicListMvcModification;

impl RefPicListMvcModification {
  pub fn new(_: &mut BitStream, nal_type: &NALUnitType) -> Option<Self> {
    matches!(
      nal_type,
      NALUnitType::CodedSliceExtension | NALUnitType::DepthOrTextureViewComponent
    )
    .then(|| todo!("RefPicListMvcModification"))
  }
}

#[derive(Debug)]
pub struct PredWeightTable;

impl PredWeightTable {
  pub fn new(
    _: &mut BitStream,
    weighted_pred_flag: bool,
    slice_type: &SliceType,
    weighted_bipred_idc: u8,
  ) -> Option<Self> {
    ((weighted_pred_flag && matches!(slice_type, SliceType::P | SliceType::SP))
      || (weighted_bipred_idc == 1 && matches!(slice_type, SliceType::B)))
    .then(|| todo!("PredWeightTable"))
  }
}

#[derive(Debug)]
pub enum DecRefPicMarking {
  Idr {
    no_output_of_prior_pics_flag: bool,
    long_term_reference_flag: bool,
  },
  AdaptiveRefPic {
    memory_management_control_operation: u16,
    difference_of_pic_nums_minus1: Option<u16>,
    long_term_pic_num: Option<u16>,
    long_term_frame_idx: Option<u16>,
    max_long_term_frame_idx_plus1: Option<u16>,
  },
  None,
}

impl DecRefPicMarking {
  pub fn new(data: &mut BitStream, nal: &NALUnit) -> Option<Self> {
    (nal.idc != 0).then(|| {
      if nal.unit_type.is_idr() {
        Self::Idr {
          no_output_of_prior_pics_flag: data.bit_flag(),
          long_term_reference_flag: data.bit_flag(),
        }
      } else if data.bit_flag() {
        let memory_management_control_operation = data.exponential_golomb();
        let mut difference_of_pic_nums_minus1 = None;
        let mut long_term_pic_num = None;
        let mut long_term_frame_idx = None;
        let mut max_long_term_frame_idx_plus1 = None;
        loop {
          if matches!(memory_management_control_operation, 1 | 3) {
            difference_of_pic_nums_minus1 = Some(data.exponential_golomb());
          }
          match memory_management_control_operation {
            0 => break,
            1 | 3 => difference_of_pic_nums_minus1 = Some(data.exponential_golomb()),
            2 => long_term_pic_num = Some(data.exponential_golomb()),
            _ => (),
          }
          match memory_management_control_operation {
            3 | 6 => long_term_frame_idx = Some(data.exponential_golomb()),
            4 => max_long_term_frame_idx_plus1 = Some(data.exponential_golomb()),
            _ => (),
          }
        }
        Self::AdaptiveRefPic {
          memory_management_control_operation,
          difference_of_pic_nums_minus1,
          long_term_pic_num,
          long_term_frame_idx,
          max_long_term_frame_idx_plus1,
        }
      } else {
        Self::None
      }
    })
  }
}

#[derive(Debug)]
pub struct DeblockingFilterControl {
  pub disable_deblocking_filter_idc: u16,
  pub slice: Option<DeblockingFilterControlSlice>,
}

impl DeblockingFilterControl {
  pub fn new(data: &mut BitStream, deblocking_filter_control_present_flag: bool) -> Option<Self> {
    deblocking_filter_control_present_flag.then(|| {
      let disable_deblocking_filter_idc = data.exponential_golomb();
      Self {
        disable_deblocking_filter_idc,
        slice: DeblockingFilterControlSlice::new(data, disable_deblocking_filter_idc),
      }
    })
  }
}

#[derive(Debug)]
pub struct DeblockingFilterControlSlice {
  pub alpha_c0_offset_div2: i16,
  pub beta_offset_div2: i16,
}

impl DeblockingFilterControlSlice {
  pub fn new(data: &mut BitStream, disable_deblocking_filter_idc: u16) -> Option<Self> {
    (disable_deblocking_filter_idc != 1).then(|| Self {
      alpha_c0_offset_div2: data.signed_exponential_golomb(),
      beta_offset_div2: data.signed_exponential_golomb(),
    })
  }
}
