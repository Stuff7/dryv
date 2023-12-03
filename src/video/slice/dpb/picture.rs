use super::*;

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum PictureMarking {
  #[default]
  ShortTermReference,
  LongTermReference,
}

impl PictureMarking {
  pub fn is_short_term_reference(&self) -> bool {
    matches!(self, PictureMarking::ShortTermReference)
  }

  pub fn is_long_term_reference(&self) -> bool {
    matches!(self, PictureMarking::LongTermReference)
  }
}

#[derive(Debug)]
pub struct Picture {
  pub pic_order_cnt: isize,
  pub pic_order_cnt_msb: isize,
  pub pic_order_cnt_lsb: isize,
  pub top_field_order_cnt: isize,
  pub bottom_field_order_cnt: isize,
  pub frame_num_offset: isize,
  pub memory_management_control_operation_5_flag: bool,
  pub memory_management_control_operation_6_flag: bool,
  pub pic_order_cnt_type: u16,
  pub pic_order_cnt_type_one: Option<PicOrderCntTypeOne>,
  pub header: SliceHeader,
  pub frame: Frame,
  pub macroblocks: Box<[Macroblock]>,
  pub nal_idc: u8,
  pub nal_unit_type: NALUnitType,
  pub max_num_ref_frames: u16,
  pub reference_marked_type: PictureMarking,
  pub long_term_frame_idx: isize,
  pub max_long_term_frame_idx: isize,
  pub pic_num: isize,
  pub long_term_pic_num: isize,
  pub frame_num_wrap: isize,
}

impl Picture {
  pub fn new(stream: &mut BitStream, nal: &NALUnit, sps: &SequenceParameterSet, pps: &PictureParameterSet) -> Self {
    let header = SliceHeader::new(stream, nal, sps, pps);
    Self {
      pic_order_cnt: 0,
      pic_order_cnt_msb: 0,
      pic_order_cnt_lsb: 0,
      top_field_order_cnt: 0,
      bottom_field_order_cnt: 0,
      frame_num_offset: 0,
      memory_management_control_operation_5_flag: false,
      memory_management_control_operation_6_flag: false,
      pic_order_cnt_type: sps.pic_order_cnt_type,
      pic_order_cnt_type_one: sps.pic_order_cnt_type_one.clone(),
      frame: Frame::new(
        header.pic_width_in_samples_l,
        header.pic_height_in_samples_l,
        header.pic_width_in_samples_c,
        header.pic_height_in_samples_c,
      ),
      macroblocks: (0..header.pic_size_in_mbs).map(|_| Macroblock::empty()).collect(),
      header,
      nal_idc: nal.idc,
      nal_unit_type: nal.unit_type,
      reference_marked_type: PictureMarking::ShortTermReference,
      max_num_ref_frames: sps.max_num_ref_frames,
      long_term_frame_idx: 0,
      max_long_term_frame_idx: 0,
      frame_num_wrap: 0,
      pic_num: 0,
      long_term_pic_num: 0,
    }
  }
}

impl OffsetArray for Picture {}
