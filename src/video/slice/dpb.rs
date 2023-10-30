use super::{header::Mmco, Slice};
use crate::video::atom::PicOrderCntTypeOne;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct DecodedPictureBuffer {
  pub poc: PictureOrderCount,
  pub buffer: Vec<Picture>,
  pub ref_pic_list0: Vec<Picture>,
  pub ref_pic_list1: Vec<Picture>,
}

impl DecodedPictureBuffer {
  pub fn new() -> Self {
    Self {
      poc: PictureOrderCount::default(),
      buffer: Vec::with_capacity(16),
      ref_pic_list0: Vec::with_capacity(16),
      ref_pic_list1: Vec::with_capacity(16),
    }
  }
  pub fn push(&mut self, slice: &Slice) -> Self {
    if slice.nal_idc != 0 {
      let pic = self.new_picture(slice);
      self.buffer.push(pic);
    }
    todo!()
  }

  pub fn previous(&self) -> &Picture {
    self.buffer.last().unwrap_or(&DEFAULT_PIC)
  }

  /// 8.2.5 Decoded reference picture marking process
  pub fn new_picture(&mut self, slice: &Slice) -> Picture {
    let mut pic = Picture::from_poc(&self.poc);
    if slice.nal_unit_type.is_idr() {
      self.buffer.clear();
      self.ref_pic_list0.clear();
      self.ref_pic_list1.clear();
      if slice
        .dec_ref_pic_marking
        .as_ref()
        .is_some_and(|drpm| drpm.long_term_reference_flag)
      {
        pic.reference_marked_type = PictureMarking::LongTermReference;
        pic.long_term_frame_idx = 0;
        pic.max_long_term_frame_idx = 0;
      } else {
        pic.reference_marked_type = PictureMarking::ShortTermReference;
        pic.max_long_term_frame_idx = -1;
      }
    } else if slice
      .dec_ref_pic_marking
      .as_ref()
      .is_some_and(|drpm| drpm.adaptive_ref_pic_marking_mode_flag)
    {
      self.adaptive_memory_control(&mut pic, slice);
    } else {
      self.sliding_window(slice.sps.max_num_ref_frames);
    }

    if !slice.nal_unit_type.is_idr() && !pic.memory_management_control_operation_6_flag {
      pic.reference_marked_type = PictureMarking::ShortTermReference;
      pic.max_long_term_frame_idx = -1;
    }
    pic
  }

  /// 8.2.5.4 Adaptive memory control decoded reference picture marking process
  pub fn adaptive_memory_control(&mut self, pic: &mut Picture, slice: &Slice) {
    let drpm = slice.dec_ref_pic_marking.as_ref().expect("Tried to start adaptive memory control decoded reference picture marking process with dec_ref_pic_marking as None");
    for mmco in &*drpm.mmcos {
      match mmco {
        Mmco::ForgetShort {
          difference_of_pic_nums_minus1,
        } => {
          let pic_num_x = slice.curr_pic_num - (*difference_of_pic_nums_minus1 as i16 + 1);
          for j in 0..self.buffer.len() {
            if self.buffer[j]
              .reference_marked_type
              .is_short_term_reference()
              && self.buffer[j].pic_num == pic_num_x
            {
              self.buffer.remove(j);
            }
          }
        }
        Mmco::ForgetLong { long_term_pic_num } => {
          for j in 0..self.buffer.len() {
            if self.buffer[j]
              .reference_marked_type
              .is_long_term_reference()
              && self.buffer[j].long_term_pic_num == *long_term_pic_num as i16
            {
              self.buffer.remove(j);
            }
          }
        }
        Mmco::ShortToLong {
          difference_of_pic_nums_minus1,
          long_term_frame_idx,
        } => {
          let pic_num_x = slice.curr_pic_num - (*difference_of_pic_nums_minus1 as i16 + 1);
          for j in 0..self.buffer.len() {
            if self.buffer[j]
              .reference_marked_type
              .is_long_term_reference()
              && self.buffer[j].long_term_frame_idx == *long_term_frame_idx as i16
            {
              self.buffer.remove(j);
            }
          }

          for j in 0..self.buffer.len() {
            if self.buffer[j]
              .reference_marked_type
              .is_short_term_reference()
              && self.buffer[j].pic_num == pic_num_x
            {
              self.buffer[j].reference_marked_type = PictureMarking::LongTermReference;
            }
          }
        }
        Mmco::ForgetLongMany {
          max_long_term_frame_idx_plus1,
        } => {
          for j in 0..self.buffer.len() {
            if (self.buffer[j].long_term_frame_idx > *max_long_term_frame_idx_plus1 as i16 - 1)
              && self.buffer[j]
                .reference_marked_type
                .is_long_term_reference()
            {
              self.buffer.remove(j);
            }
          }
          if *max_long_term_frame_idx_plus1 == 0 {
            pic.max_long_term_frame_idx = -1;
          } else {
            pic.max_long_term_frame_idx = *max_long_term_frame_idx_plus1 as i16 - 1;
          }
        }
        Mmco::ForgetAll => {
          self.buffer.clear();
          pic.max_long_term_frame_idx = -1;
          pic.memory_management_control_operation_5_flag = true;
        }
        Mmco::ThisToLong {
          long_term_frame_idx,
        } => {
          for j in 0..self.buffer.len() {
            if self.buffer[j].long_term_frame_idx == *long_term_frame_idx as i16
              && self.buffer[j]
                .reference_marked_type
                .is_long_term_reference()
            {
              self.buffer.remove(j);
            }
          }

          pic.reference_marked_type = PictureMarking::LongTermReference;
          pic.long_term_frame_idx = *long_term_frame_idx as i16;
          pic.memory_management_control_operation_6_flag = true;
        }
      }
    }
  }

  /// 8.2.5.3 Sliding window decoded reference picture marking process
  pub fn sliding_window(&mut self, max_num_ref_frames: u16) {
    let mut num_short_term = 0;
    let mut num_long_term = 0;

    for dpb in &self.buffer {
      match dpb.reference_marked_type {
        PictureMarking::ShortTermReference => num_short_term += 1,
        PictureMarking::LongTermReference => num_long_term += 1,
        _ => (),
      }
    }

    if num_short_term + num_long_term == std::cmp::max(max_num_ref_frames as i16, 1i16)
      && num_short_term > 0
    {
      let mut frame_num_wrap = -1;
      let mut pic = false;
      let mut idx = -1;
      for (i, dpb) in self.buffer.iter().enumerate() {
        if dpb.reference_marked_type.is_short_term_reference() {
          if frame_num_wrap == -1 {
            idx = i as isize;
            pic = true;
            frame_num_wrap = dpb.frame_num_wrap;
          }

          if dpb.frame_num_wrap < frame_num_wrap {
            idx = i as isize;
            pic = true;
            frame_num_wrap = dpb.frame_num_wrap;
          }
        }
      }

      if pic && idx >= 0 {
        self.buffer.remove(idx as usize);
      }
    }
  }

  /// 8.2.1 Decoding process for picture order count
  pub fn decode_pic_order_cnt_type(&mut self, slice: &Slice) {
    if slice.sps.pic_order_cnt_type == 0 {
      self.poc_type_0(slice);
    } else if slice.sps.pic_order_cnt_type == 1 {
      self.poc_type_1(slice);
    } else if slice.sps.pic_order_cnt_type == 2 {
      self.poc_type_2(slice);
    }

    self.poc.pic_order_cnt = std::cmp::min(
      self.poc.top_field_order_cnt,
      self.poc.bottom_field_order_cnt,
    );
  }

  /// 8.2.1.1 Decoding process for picture order count type 0
  pub fn poc_type_0(&mut self, slice: &Slice) {
    let previous = self.previous();
    let prev_pic_order_cnt_msb;
    let prev_pic_order_cnt_lsb;

    if slice.nal_unit_type.is_idr() {
      prev_pic_order_cnt_msb = 0;
      prev_pic_order_cnt_lsb = 0;
    } else if previous.memory_management_control_operation_5_flag {
      prev_pic_order_cnt_msb = 0;
      prev_pic_order_cnt_lsb = previous.top_field_order_cnt;
    } else {
      prev_pic_order_cnt_msb = previous.pic_order_cnt_msb;
      prev_pic_order_cnt_lsb = previous.pic_order_cnt_lsb;
    }

    let pic_order_cnt_lsb = slice.pic_order_cnt_lsb.unwrap_or_default() as i16;
    if pic_order_cnt_lsb < prev_pic_order_cnt_lsb
      && ((prev_pic_order_cnt_lsb - pic_order_cnt_lsb) >= (slice.max_pic_order_cnt_lsb / 2))
    {
      self.poc.pic_order_cnt_msb = prev_pic_order_cnt_msb + slice.max_pic_order_cnt_lsb;
    } else if (pic_order_cnt_lsb > prev_pic_order_cnt_lsb)
      && ((pic_order_cnt_lsb - prev_pic_order_cnt_lsb) > (slice.max_pic_order_cnt_lsb / 2))
    {
      self.poc.pic_order_cnt_msb = prev_pic_order_cnt_msb - slice.max_pic_order_cnt_lsb;
    } else {
      self.poc.pic_order_cnt_msb = prev_pic_order_cnt_msb;
    }

    self.poc.top_field_order_cnt = self.poc.pic_order_cnt_msb + pic_order_cnt_lsb;
    self.poc.bottom_field_order_cnt =
      self.poc.top_field_order_cnt + slice.delta_pic_order_cnt_bottom.unwrap_or_default();
  }

  /// 8.2.1.2 Decoding process for picture order count type 1
  pub fn poc_type_1(&mut self, slice: &Slice) {
    let previous = self.previous();
    let mut prev_frame_num_offset = 0;
    if !slice.nal_unit_type.is_idr() {
      if previous.memory_management_control_operation_5_flag {
        prev_frame_num_offset = 0;
      } else {
        prev_frame_num_offset = previous.frame_num_offset;
      }
    }

    if slice.nal_unit_type.is_idr() {
      self.poc.frame_num_offset = 0;
    } else if previous.frame_num > slice.frame_num as i16 {
      self.poc.frame_num_offset = prev_frame_num_offset + slice.max_frame_num as i16;
    } else {
      self.poc.frame_num_offset = prev_frame_num_offset;
    }

    let PicOrderCntTypeOne {
      num_ref_frames_in_pic_order_cnt_cycle,
      expected_delta_per_pic_order_cnt_cycle,
      offset_for_ref_frame,
      offset_for_non_ref_pic,
      offset_for_top_to_bottom_field,
      ..
    } = slice.sps.pic_order_cnt_type_one.as_ref().expect(
      "Picture order count type 1 decoding process started but pic_order_cnt_type is not 1",
    );
    let mut abs_frame_num;
    if *num_ref_frames_in_pic_order_cnt_cycle != 0 {
      abs_frame_num = self.poc.frame_num_offset + slice.frame_num as i16;
    } else {
      abs_frame_num = 0;
    }

    if slice.nal_idc == 0 && abs_frame_num > 0 {
      abs_frame_num = abs_frame_num - 1;
    }
    let mut pic_order_cnt_cycle_cnt = 0;
    let mut frame_num_in_pic_order_cnt_cycle = 0;
    if abs_frame_num > 0 {
      pic_order_cnt_cycle_cnt = (abs_frame_num - 1) / *num_ref_frames_in_pic_order_cnt_cycle as i16;
      frame_num_in_pic_order_cnt_cycle =
        (abs_frame_num - 1) % *num_ref_frames_in_pic_order_cnt_cycle as i16;
    }

    let mut expected_pic_order_cnt;
    if abs_frame_num > 0 {
      expected_pic_order_cnt =
        pic_order_cnt_cycle_cnt as i16 * expected_delta_per_pic_order_cnt_cycle;
      for i in 0..frame_num_in_pic_order_cnt_cycle as usize {
        expected_pic_order_cnt = expected_pic_order_cnt + offset_for_ref_frame[i];
      }
    } else {
      expected_pic_order_cnt = 0;
    }

    if slice.nal_idc == 0 {
      expected_pic_order_cnt = expected_pic_order_cnt + offset_for_non_ref_pic;
    }

    let delta_pic_order_cnt = slice
      .delta_pic_order_cnt
      .expect("No delta_pic_order_cnt found for pic_order_cnt_type 1");
    if !slice.field_pic_flag {
      self.poc.top_field_order_cnt = expected_pic_order_cnt + delta_pic_order_cnt.0;
      self.poc.bottom_field_order_cnt = self.poc.top_field_order_cnt
        + offset_for_top_to_bottom_field
        + delta_pic_order_cnt.1.unwrap_or_default();
    } else if !slice.bottom_field_flag {
      self.poc.top_field_order_cnt = expected_pic_order_cnt + delta_pic_order_cnt.0;
    } else {
      self.poc.bottom_field_order_cnt =
        expected_pic_order_cnt + offset_for_top_to_bottom_field + delta_pic_order_cnt.0;
    }
  }

  /// 8.2.1.3 Decoding process for picture order count type 2
  pub fn poc_type_2(&mut self, slice: &Slice) {
    let previous = self.previous();
    let mut prev_frame_num_offset = 0;
    if !slice.nal_unit_type.is_idr() {
      if previous.memory_management_control_operation_5_flag {
        prev_frame_num_offset = 0;
      } else {
        prev_frame_num_offset = previous.frame_num_offset;
      }
    }

    if slice.nal_unit_type.is_idr() {
      self.poc.frame_num_offset = 0;
    } else if previous.frame_num > slice.frame_num as i16 {
      self.poc.frame_num_offset = prev_frame_num_offset + slice.max_frame_num as i16;
    } else {
      self.poc.frame_num_offset = prev_frame_num_offset;
    }

    let temp_pic_order_cnt;
    if slice.nal_unit_type.is_idr() {
      temp_pic_order_cnt = 0;
    } else if slice.nal_idc == 0 {
      temp_pic_order_cnt = 2 * (self.poc.frame_num_offset + slice.frame_num as i16) - 1;
    } else {
      temp_pic_order_cnt = 2 * (self.poc.frame_num_offset + slice.frame_num as i16);
    }

    if !slice.field_pic_flag {
      self.poc.top_field_order_cnt = temp_pic_order_cnt;
      self.poc.bottom_field_order_cnt = temp_pic_order_cnt;
    } else if slice.bottom_field_flag {
      self.poc.bottom_field_order_cnt = temp_pic_order_cnt;
    } else {
      self.poc.top_field_order_cnt = temp_pic_order_cnt;
    }
  }
}

const DEFAULT_PIC: Picture = Picture::unknown();

#[derive(Debug, Default)]
pub enum PictureMarking {
  #[default]
  Unknown,
  UsedForReference,
  ShortTermReference,
  LongTermReference,
  UnusedForReference,
}

impl PictureMarking {
  pub fn is_used_for_reference(&self) -> bool {
    matches!(self, PictureMarking::UsedForReference)
  }

  pub fn is_short_term_reference(&self) -> bool {
    matches!(self, PictureMarking::ShortTermReference)
  }

  pub fn is_long_term_reference(&self) -> bool {
    matches!(self, PictureMarking::LongTermReference)
  }

  pub fn is_unused_for_reference(&self) -> bool {
    matches!(self, PictureMarking::UnusedForReference)
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PictureOrderCount {
  pub pic_order_cnt: i16,
  pub pic_order_cnt_msb: i16,
  pub pic_order_cnt_lsb: i16,
  pub top_field_order_cnt: i16,
  pub bottom_field_order_cnt: i16,
  pub frame_num_offset: i16,
}

#[derive(Debug, Default)]
pub struct Picture {
  pub poc: PictureOrderCount,
  pub reference_marked_type: PictureMarking,
  pub frame_num: i16,
  pub long_term_frame_idx: i16,
  pub max_long_term_frame_idx: i16,
  pub pic_num: i16,
  pub long_term_pic_num: i16,
  pub frame_num_wrap: i16,
  pub memory_management_control_operation_5_flag: bool,
  pub memory_management_control_operation_6_flag: bool,
}

impl Deref for Picture {
  type Target = PictureOrderCount;
  fn deref(&self) -> &Self::Target {
    &self.poc
  }
}

impl DerefMut for Picture {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.poc
  }
}

impl Picture {
  pub const fn unknown() -> Self {
    Self {
      poc: PictureOrderCount {
        pic_order_cnt: 0,
        pic_order_cnt_msb: 0,
        pic_order_cnt_lsb: 0,
        top_field_order_cnt: 0,
        bottom_field_order_cnt: 0,
        frame_num_offset: 0,
      },
      reference_marked_type: PictureMarking::Unknown,
      frame_num: 0,
      long_term_frame_idx: 0,
      max_long_term_frame_idx: 0,
      pic_num: 0,
      long_term_pic_num: 0,
      frame_num_wrap: 0,
      memory_management_control_operation_5_flag: false,
      memory_management_control_operation_6_flag: false,
    }
  }

  pub const fn from_poc(poc: &PictureOrderCount) -> Self {
    Self {
      poc: *poc,
      ..Self::unknown()
    }
  }
}
