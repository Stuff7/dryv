use crate::{
  byte::BitStream,
  video::{
    atom::{PictureParameterSet, SequenceParameterSet, SliceGroup},
    sample::{NALUnit, NALUnitType},
  },
};

#[derive(Debug)]
pub struct SliceHeader {
  /// The index of the first macroblock in the slice.
  pub first_mb_in_slice: u16,

  /// The type of slice (I, P, B, etc.).
  pub slice_type: SliceType,

  /// The index of the Picture Parameter Set (PPS) used for this slice.
  pub pps_id: u16,

  /// An optional identifier for the color plane (only applicable in certain formats).
  pub color_plane_id: Option<u8>,

  /// The frame number associated with the slice.
  /// This is used for identifying frames in video sequences.
  pub frame_num: u16,

  /// Indicates if the picture is a field picture.
  /// In field pictures, a single frame is divided into two fields.
  pub field_pic_flag: bool,

  /// Indicates if the picture is a bottom field.
  /// This flag specifies whether the field is the top or bottom field in a frame.
  pub bottom_field_flag: bool,

  /// An optional identifier for an Instantaneous Decoding Refresh (IDR) picture.
  /// IDR pictures are self-contained and do not rely on other reference pictures.
  pub idr_pic_id: Option<u16>,

  /// An optional value for the picture order count least significant bits (POC LSB).
  /// The POC is used for ordering pictures in the decoding process.
  pub pic_order_cnt_lsb: Option<u16>,

  /// An optional value for the delta picture order count bottom.
  /// It represents the difference between POC values of adjacent frames.
  pub delta_pic_order_cnt_bottom: Option<i16>,

  /// An optional tuple representing the delta picture order count (POC) values.
  /// The first element is the POC value for reference picture 0, and the second element is for reference picture 1.
  pub delta_pic_order_cnt: Option<(i16, Option<i16>)>,

  /// An optional value for redundant picture count.
  /// Redundant slices are used for error recovery in video coding.
  pub redundant_pic_cnt: Option<u16>,

  /// Indicates whether direct spatial motion vector prediction is used.
  /// Direct spatial motion vector prediction is a technique in video coding where motion vectors are predicted directly
  /// from spatial neighbors, typically in inter-coded frames (P-frames and B-frames).
  pub direct_spatial_mv_pred_flag: bool,

  /// Indicates if the number of reference indices is overridden.
  /// When overridden, the decoder uses a different number of reference pictures for prediction.
  pub num_ref_idx_active_override_flag: bool,

  /// The number of reference indices for List 0 minus 1.
  /// This value determines the number of reference pictures available for List 0 prediction.
  pub num_ref_idx_l0_active_minus1: u16,

  /// The number of reference indices for List 1 minus 1.
  /// This value determines the number of reference pictures available for List 1 prediction.
  pub num_ref_idx_l1_active_minus1: u16,

  /// An optional modification for reference picture list in MVC (Multi-View Coding).
  /// This field is used for reference picture list modifications in multi-view video coding.
  pub ref_pic_list_mvc_modification: Option<RefPicListMvcModification>,

  /// An optional modification for reference picture list.
  /// It specifies the modification of reference pictures used for prediction.
  pub ref_pic_list_modification_l0: Box<[RefPicListModification]>,
  pub ref_pic_list_modification_l1: Box<[RefPicListModification]>,

  /// An identifier for the chroma array type.
  /// This value describes the chroma sampling format used in the video stream.
  pub chroma_array_type: u16,

  /// An optional prediction weight table.
  /// This table contains information about prediction weights used in weighted prediction.
  pub pred_weight_table: Option<PredWeightTable>,

  /// An optional decoding reference picture marking.
  /// It contains information about reference picture management in the video stream.
  pub dec_ref_pic_marking: Option<DecRefPicMarking>,

  /// An optional identifier for the Context-based Adaptive Binary Arithmetic Coding (CABAC) initialization.
  /// CABAC is a technique for entropy coding used in video compression.
  pub cabac_init_idc: Option<u16>,

  /// The slice quantization parameter (QP) delta.
  /// QP affects the quantization of coefficients in video compression.
  pub slice_qp_delta: i16,

  /// Indicates if the slice is switching SP/SI frames.
  /// SP/SI frames are used in video coding for different prediction methods.
  pub sp_for_switch_flag: bool,

  /// An optional value for the slice quantization parameter (QP) delta.
  /// This field allows for finer control over quantization in certain cases.
  pub slice_qs_delta: Option<i16>,

  /// An optional control for the deblocking filter.
  /// The deblocking filter is used to reduce artifacts in compressed video.
  pub deblocking_filter_control: Option<DeblockingFilterControl>,

  /// An optional value for the slice group change cycle.
  /// This field is used in slice group-based video coding.
  pub slice_group_change_cycle: Option<u16>,

  pub max_pic_order_cnt_lsb: i16,

  pub max_frame_num: u16,

  pub curr_pic_num: i16,

  pub max_pic_num: i16,

  pub sub_width_c: i8,

  pub sub_height_c: i8,

  pub mb_width_c: u8,

  pub mb_height_c: u8,

  pub scaling_list4x4: [[isize; 16]; 6],

  pub scaling_list8x8: Box<[[isize; 64]]>,

  pub qp_bd_offset_c: isize,

  pub bit_depth_y: isize,

  pub bit_depth_c: isize,
}

impl SliceHeader {
  pub fn new(
    data: &mut BitStream,
    nal: &NALUnit,
    sps: &mut SequenceParameterSet,
    pps: &mut PictureParameterSet,
  ) -> Self {
    let mut field_pic_flag = false;
    let slice_type;
    let num_ref_idx_active_override_flag;
    let num_ref_idx_l0_active_minus1;
    let num_ref_idx_l1_active_minus1;
    let frame_num;
    let max_frame_num;
    let (sub_width_c, sub_height_c) = if sps.separate_color_plane_flag {
      (-1, -1)
    } else {
      match sps.chroma_format_idc {
        1 => (2, 2),
        2 => (2, 1),
        3 => (1, 1),
        _ => (-1, -1),
      }
    };
    let chroma_array_type = match nal.unit_type {
      NALUnitType::AuxiliaryCodedPicture => 0,
      _ => {
        if sps.separate_color_plane_flag {
          0
        } else {
          sps.chroma_format_idc
        }
      }
    };
    let (mb_width_c, mb_height_c) = if sps.chroma_format_idc == 0 || sps.separate_color_plane_flag {
      (0, 0)
    } else {
      (16 / sub_width_c as u8, 16 / sub_height_c as u8)
    };
    let (scaling_list4x4, scaling_list8x8) = Self::scaling_lists(sps, pps);

    Self {
      first_mb_in_slice: data.exponential_golomb(),
      slice_type: {
        slice_type = SliceType::new(data.exponential_golomb());
        slice_type
      },
      pps_id: data.exponential_golomb(),
      color_plane_id: sps.separate_color_plane_flag.then(|| data.bits_into(2)),
      frame_num: {
        frame_num = data.bits_into(sps.log2_max_frame_num_minus4 as usize + 4);
        frame_num
      },
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
      direct_spatial_mv_pred_flag: slice_type.is_bidirectional() && data.bit_flag(),
      num_ref_idx_active_override_flag: {
        num_ref_idx_active_override_flag = !slice_type.is_intra() && data.bit_flag();
        num_ref_idx_active_override_flag
      },
      num_ref_idx_l0_active_minus1: {
        num_ref_idx_l0_active_minus1 = num_ref_idx_active_override_flag
          .then(|| data.exponential_golomb())
          .unwrap_or(pps.num_ref_idx_l0_default_active_minus1);
        num_ref_idx_l0_active_minus1
      },
      num_ref_idx_l1_active_minus1: {
        num_ref_idx_l1_active_minus1 = (num_ref_idx_active_override_flag
          && slice_type.is_bidirectional())
        .then(|| data.exponential_golomb())
        .unwrap_or(pps.num_ref_idx_l1_default_active_minus1);
        num_ref_idx_l1_active_minus1
      },
      ref_pic_list_mvc_modification: RefPicListMvcModification::new(data, &nal.unit_type),
      ref_pic_list_modification_l0: RefPicListModification::new_list(
        data,
        &nal.unit_type,
        !slice_type.is_intra(),
      ),
      ref_pic_list_modification_l1: RefPicListModification::new_list(
        data,
        &nal.unit_type,
        slice_type.is_bidirectional(),
      ),
      chroma_array_type,
      pred_weight_table: PredWeightTable::new(
        data,
        chroma_array_type,
        num_ref_idx_l0_active_minus1,
        num_ref_idx_l1_active_minus1,
        pps.weighted_pred_flag,
        &slice_type,
        pps.weighted_bipred_idc,
      ),
      dec_ref_pic_marking: DecRefPicMarking::new(data, nal),
      cabac_init_idc: (pps.entropy_coding_mode_flag && !slice_type.is_intra())
        .then(|| data.exponential_golomb()),
      slice_qp_delta: data.signed_exponential_golomb(),
      sp_for_switch_flag: matches!(slice_type, SliceType::SP) && data.bit_flag(),
      slice_qs_delta: slice_type
        .is_switching()
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
      max_pic_order_cnt_lsb: sps
        .log2_max_pic_order_cnt_lsb_minus4
        .map(|x| 1 << (x + 4))
        .unwrap_or_default() as i16,
      max_frame_num: {
        max_frame_num = 1 << (sps.log2_max_frame_num_minus4 as i16 + 4);
        max_frame_num
      },
      curr_pic_num: if field_pic_flag {
        frame_num as i16
      } else {
        2 * frame_num as i16 + 1
      },
      max_pic_num: if !field_pic_flag {
        max_frame_num as i16
      } else {
        2 * max_frame_num as i16
      },
      sub_width_c,
      sub_height_c,
      mb_width_c,
      mb_height_c,
      scaling_list4x4,
      scaling_list8x8,
      qp_bd_offset_c: sps.bit_depth_chroma_minus8 as isize * 6,
      bit_depth_y: sps.bit_depth_luma_minus8 as isize + 8,
      bit_depth_c: sps.bit_depth_chroma_minus8 as isize + 8,
    }
  }

  fn scaling_lists(
    sps: &mut SequenceParameterSet,
    pps: &mut PictureParameterSet,
  ) -> ([[isize; 16]; 6], Box<[[isize; 64]]>) {
    if let Some(scaling_list) = sps.seq_scaling_matrix.take() {
      (scaling_list.l4x4, scaling_list.l8x8)
    } else if let Some(scaling_list) = pps
      .extra_rbsp_data
      .as_mut()
      .and_then(|pps| pps.pic_scaling_matrix.take())
    {
      (scaling_list.l4x4, scaling_list.l8x8)
    } else {
      ([[16; 16]; 6], [[16; 64]; 6].into())
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SliceType {
  /// P-slice (Predictive Coded Slice):
  /// A slice type containing inter-coded macroblocks for motion compensation,
  /// typically referring to previously coded P or I slices.
  P,

  /// B-slice (Bi-directional Predictive Coded Slice):
  /// A slice type containing inter-coded macroblocks for motion compensation,
  /// considering both past and future reference frames, allowing for more
  /// complex temporal predictions.
  B,

  /// I-slice (Intra-coded Slice):
  /// A slice type containing intra-coded macroblocks, meaning that each macroblock
  /// is independently coded without reference to other slices, providing a clean
  /// entry point for error resilience.
  I,

  /// SP-slice (Switching P-slice):
  /// A slice type that serves as a switch from P-slice to I-slice coding modes
  /// and vice versa, typically used for video streams with varying scene complexity.
  SP,

  /// SI-slice (Switching I-slice):
  /// A slice type that serves as a switch from I-slice to P-slice coding modes
  /// and vice versa, similarly used for video streams with changing scene conditions.
  SI,
}

impl SliceType {
  pub fn new(slice_type: u8) -> Self {
    match slice_type {
      0 | 5 => Self::P,
      1 | 6 => Self::B,
      2 | 7 => Self::I,
      3 | 8 => Self::SP,
      4 | 9 => Self::SI,
      n => panic!("Unknown slice type {n}"),
    }
  }

  /// Checks if the slice type is an intra-coded slice (I-slice or SI-slice).
  pub fn is_intra(&self) -> bool {
    matches!(self, SliceType::I | SliceType::SI)
  }

  /// Checks if the slice type is a predictive slice (P-slice or SP-slice).
  pub fn is_predictive(&self) -> bool {
    matches!(self, SliceType::P | SliceType::SP)
  }

  /// Checks if the slice type is a switching slice (SP-slice or SI-slice).
  pub fn is_switching(&self) -> bool {
    matches!(self, SliceType::SP | SliceType::SI)
  }

  pub fn is_switching_p(&self) -> bool {
    matches!(self, SliceType::SP)
  }

  /// Checks if the slice type is a bidirectional slice (B-slice).
  pub fn is_bidirectional(&self) -> bool {
    matches!(self, SliceType::B)
  }
}

#[derive(Debug)]
pub struct RefPicListModification {
  pub modification_of_pic_nums_idc: u16,
  pub abs_diff_pic_num_minus1: u16,
  pub long_term_pic_num: u16,
}

impl RefPicListModification {
  pub fn new_list(data: &mut BitStream, nal_type: &NALUnitType, condition: bool) -> Box<[Self]> {
    match !matches!(
      nal_type,
      NALUnitType::CodedSliceExtension | NALUnitType::DepthOrTextureViewComponent
    ) && condition
      && data.bit_flag()
    {
      true => std::iter::from_fn(|| {
        let modification_of_pic_nums_idc = data.exponential_golomb();
        match modification_of_pic_nums_idc {
          3 => None,
          0 | 1 | 2 => Some(Self {
            modification_of_pic_nums_idc,
            abs_diff_pic_num_minus1: match modification_of_pic_nums_idc {
              1 | 0 => data.exponential_golomb(),
              _ => 0,
            },
            long_term_pic_num: match modification_of_pic_nums_idc {
              2 => data.exponential_golomb(),
              _ => 0,
            },
          }),
          n => panic!("Unknown ref_pic_list_modification {n}"),
        }
      })
      .collect(),
      false => [].into(),
    }
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
pub struct PredWeightTableEntry {
  pub luma_weight: i16,
  pub luma_offset: i16,
  pub chroma_weight: [i16; 2],
  pub chroma_offset: [i16; 2],
}

impl PredWeightTableEntry {
  fn new(
    stream: &mut BitStream,
    luma_log2_weight_denom: u16,
    chroma_log2_weight_denom: u16,
  ) -> Self {
    let (luma_weight, luma_offset) = match stream.bit_flag() {
      true => (
        stream.signed_exponential_golomb(),
        stream.signed_exponential_golomb(),
      ),
      false => (1 << luma_log2_weight_denom, 0),
    };
    let (chroma_weight, chroma_offset) = match stream.bit_flag() {
      true => {
        let a = [stream.signed_exponential_golomb(); 4];
        ([a[0], a[2]], [a[1], a[3]])
      }
      false => ([1 << chroma_log2_weight_denom; 2], [0; 2]),
    };
    Self {
      luma_weight,
      luma_offset,
      chroma_weight,
      chroma_offset,
    }
  }
}

#[derive(Debug)]
pub struct PredWeightTable {
  pub luma_log2_weight_denom: u16,
  pub chroma_log2_weight_denom: u16,
  pub l0: Box<[PredWeightTableEntry]>,
  pub l1: Box<[PredWeightTableEntry]>,
}

impl PredWeightTable {
  pub fn new(
    stream: &mut BitStream,
    chroma_array_type: u16,
    num_ref_idx_l0_active_minus1: u16,
    num_ref_idx_l1_active_minus1: u16,
    weighted_pred_flag: bool,
    slice_type: &SliceType,
    weighted_bipred_idc: u8,
  ) -> Option<Self> {
    ((weighted_pred_flag && slice_type.is_predictive())
      || (weighted_bipred_idc == 1 && slice_type.is_bidirectional()))
    .then(|| {
      let luma_log2_weight_denom = stream.exponential_golomb();
      let chroma_log2_weight_denom = (chroma_array_type != 0)
        .then(|| stream.exponential_golomb())
        .unwrap_or_default();
      Self {
        luma_log2_weight_denom,
        chroma_log2_weight_denom,
        l0: (0..=num_ref_idx_l0_active_minus1)
          .map(|_| {
            PredWeightTableEntry::new(stream, luma_log2_weight_denom, chroma_log2_weight_denom)
          })
          .collect(),
        l1: if !slice_type.is_predictive() {
          (0..=num_ref_idx_l1_active_minus1)
            .map(|_| {
              PredWeightTableEntry::new(stream, luma_log2_weight_denom, chroma_log2_weight_denom)
            })
            .collect()
        } else {
          [].into()
        },
      }
    })
  }
}

#[derive(Debug)]
pub struct DecRefPicMarking {
  pub no_output_of_prior_pics_flag: bool,
  pub long_term_reference_flag: bool,
  pub adaptive_ref_pic_marking_mode_flag: bool,
  pub mmcos: Box<[Mmco]>,
}

#[derive(Debug)]
pub enum Mmco {
  ForgetShort {
    difference_of_pic_nums_minus1: u16,
  },
  ForgetLong {
    long_term_pic_num: u16,
  },
  ShortToLong {
    difference_of_pic_nums_minus1: u16,
    long_term_frame_idx: u16,
  },
  ForgetLongMany {
    max_long_term_frame_idx_plus1: u16,
  },
  ForgetAll,
  ThisToLong {
    long_term_frame_idx: u16,
  },
}

impl DecRefPicMarking {
  pub fn new(data: &mut BitStream, nal: &NALUnit) -> Option<Self> {
    (nal.idc != 0).then(|| {
      let (no_output_of_prior_pics_flag, long_term_reference_flag) = match nal.unit_type.is_idr() {
        true => (data.bit_flag(), data.bit_flag()),
        false => (false, false),
      };
      let adaptive_ref_pic_marking_mode_flag = !nal.unit_type.is_idr() && data.bit_flag();
      Self {
        no_output_of_prior_pics_flag,
        long_term_reference_flag,
        adaptive_ref_pic_marking_mode_flag,
        mmcos: match adaptive_ref_pic_marking_mode_flag {
          true => std::iter::from_fn(|| match data.exponential_golomb() {
            0 => None,
            1 => Some(Mmco::ForgetShort {
              difference_of_pic_nums_minus1: data.exponential_golomb(),
            }),
            2 => Some(Mmco::ForgetLong {
              long_term_pic_num: data.exponential_golomb(),
            }),
            3 => Some(Mmco::ShortToLong {
              difference_of_pic_nums_minus1: data.exponential_golomb(),
              long_term_frame_idx: data.exponential_golomb(),
            }),
            4 => Some(Mmco::ForgetLongMany {
              max_long_term_frame_idx_plus1: data.exponential_golomb(),
            }),
            5 => Some(Mmco::ForgetAll),
            6 => Some(Mmco::ThisToLong {
              long_term_frame_idx: data.exponential_golomb(),
            }),
            n => panic!("Unknown MMCO {n}"),
          })
          .collect(),
          false => [].into(),
        },
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
