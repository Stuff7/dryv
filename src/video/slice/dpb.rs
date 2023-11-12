use super::{
  header::{Mmco, SliceHeader},
  Slice,
};
use crate::{math::OffsetArray, video::atom::PicOrderCntTypeOne};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct DecodedPictureBuffer {
  pub poc: PictureOrderCount,
  pub buffer: Vec<Picture>,
  pub ref_pic_list0: Vec<usize>,
  pub ref_pic_list1: Vec<usize>,
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

  pub fn push(&mut self, slice: &Slice) {
    if slice.nal_idc != 0 {
      let pic = self.new_picture(slice);
      self.buffer.push(pic);
    }
  }

  pub fn previous(&self) -> &Picture {
    self.buffer.last().unwrap_or(&DEFAULT_PIC)
  }

  pub fn ref_pic_list0(&self, idx: usize) -> &Picture {
    &self.buffer[self.ref_pic_list0[idx]]
  }

  pub fn ref_pic_list1(&self, idx: usize) -> &Picture {
    &self.buffer[self.ref_pic_list1[idx]]
  }

  /// 8.2.4 Decoding process for reference picture lists construction
  pub fn reference_picture_lists_construction(&mut self, slice: &Slice) {
    self.ref_pic_list0.clear();
    self.ref_pic_list1.clear();

    self.picture_numbers(&slice.header);
    self.reference_picture_lists(slice);
    self.modification_for_reference_picture_lists(&slice.header);
  }

  /// 8.2.4.1 Decoding process for picture numbers
  pub fn picture_numbers(&mut self, header: &SliceHeader) {
    for dpb in &mut self.buffer {
      if dpb.reference_marked_type.is_short_term_reference() {
        if dpb.frame_num > header.frame_num as isize {
          dpb.frame_num_wrap -= dpb.max_frame_num;
        } else {
          dpb.frame_num_wrap = dpb.frame_num;
        }
      }
    }

    for dpb in &mut self.buffer {
      if dpb.reference_marked_type.is_short_term_reference() {
        dpb.pic_num = dpb.frame_num_wrap;
      }

      if dpb.reference_marked_type.is_long_term_reference() {
        dpb.long_term_pic_num = dpb.long_term_frame_idx;
      }
    }
  }

  /// 8.2.4.2 Initialization process for reference picture lists
  pub fn reference_picture_lists(&mut self, header: &SliceHeader) {
    if header.slice_type.is_predictive() {
      self.reference_picture_list_for_p_and_sp_slices_in_frames();
    } else if header.slice_type.is_bidirectional() {
      self.reference_picture_lists_for_b_slices_in_frames(self.poc.pic_order_cnt);
    }

    let num_ref_idx_l0_active = header.num_ref_idx_l0_active_minus1 as usize + 1;
    if self.ref_pic_list0.len() > num_ref_idx_l0_active {
      self.ref_pic_list0.drain(num_ref_idx_l0_active..);
    }

    let num_ref_idx_l1_active = header.num_ref_idx_l1_active_minus1 as usize + 1;
    if self.ref_pic_list1.len() > num_ref_idx_l1_active {
      self.ref_pic_list1.drain(num_ref_idx_l1_active..);
    }
  }

  /// 8.2.4.2.1 Initialization process for the reference picture list for P and SP slices in frames
  pub fn reference_picture_list_for_p_and_sp_slices_in_frames(&mut self) {
    let (mut short_term, mut long_term): (Vec<&_>, Vec<&_>) = self
      .buffer
      .iter()
      .filter(|dpb| {
        matches!(
          dpb.reference_marked_type,
          PictureMarking::ShortTermReference | PictureMarking::LongTermReference
        )
      })
      .partition(|dpb| dpb.reference_marked_type.is_short_term_reference());

    if !short_term.is_empty() {
      for i in 0..short_term.len() - 1 {
        for j in 0..short_term.len() - i - 1 {
          if short_term[j].pic_num < short_term[j + 1].pic_num {
            short_term.swap(j, j + 1);
          }
        }
      }
    }

    if !long_term.is_empty() {
      for i in 0..long_term.len() - 1 {
        for j in 0..long_term.len() - i - 1 {
          if long_term[j].long_term_pic_num > long_term[j + 1].long_term_pic_num {
            long_term.swap(j, j + 1);
          }
        }
      }
    }

    for i in 0..short_term.len() {
      self
        .ref_pic_list0
        .push(short_term[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self
        .ref_pic_list0
        .push(long_term[i].offset_array(&self.buffer));
    }
  }

  /// 8.2.4.2.4 Initialization process for reference picture lists for B slices in fields
  pub fn reference_picture_lists_for_b_slices_in_frames(&mut self, poc: isize) {
    let (mut short_term_left, mut short_term_right): (Vec<&_>, Vec<&_>) = self
      .buffer
      .iter()
      .filter(|dpb| dpb.reference_marked_type.is_short_term_reference())
      .partition(|dpb| dpb.pic_order_cnt < poc);
    let mut long_term: Vec<&_> = self
      .buffer
      .iter()
      .filter(|dpb| dpb.reference_marked_type.is_long_term_reference())
      .collect();

    if !short_term_left.is_empty() {
      for i in 0..short_term_left.len() - 1 {
        for j in 0..short_term_left.len() - i - 1 {
          if short_term_left[j].pic_order_cnt < short_term_left[j + 1].pic_order_cnt {
            short_term_left.swap(j, j + 1);
          }
        }
      }
    }

    if !short_term_right.is_empty() {
      for i in 0..short_term_right.len() - 1 {
        for j in 0..short_term_right.len() - i - 1 {
          if short_term_right[j].pic_order_cnt > short_term_right[j + 1].pic_order_cnt {
            short_term_right.swap(j, j + 1);
          }
        }
      }
    }

    if !long_term.is_empty() {
      for i in 0..long_term.len() - 1 {
        for j in 0..long_term.len() - i - 1 {
          if long_term[j].long_term_pic_num > long_term[j + 1].long_term_pic_num {
            long_term.swap(j, j + 1);
          }
        }
      }
    }

    for i in 0..short_term_left.len() {
      self
        .ref_pic_list0
        .push(short_term_left[i].offset_array(&self.buffer));
    }

    for i in 0..short_term_right.len() {
      self
        .ref_pic_list0
        .push(short_term_right[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self
        .ref_pic_list0
        .push(long_term[i].offset_array(&self.buffer));
    }

    let (mut short_term_left, mut short_term_right): (Vec<&_>, Vec<&_>) = self
      .buffer
      .iter()
      .filter(|dpb| dpb.reference_marked_type.is_short_term_reference())
      .partition(|dpb| dpb.pic_order_cnt > poc);
    let mut long_term: Vec<&_> = self
      .buffer
      .iter()
      .filter(|dpb| dpb.reference_marked_type.is_long_term_reference())
      .collect();

    if !short_term_left.is_empty() {
      for i in 0..short_term_left.len() - 1 {
        for j in 0..short_term_left.len() - i - 1 {
          if short_term_left[j].pic_order_cnt > short_term_left[j + 1].pic_order_cnt {
            short_term_left.swap(j, j + 1);
          }
        }
      }
    }

    if !short_term_right.is_empty() {
      for i in 0..short_term_right.len() - 1 {
        for j in 0..short_term_right.len() - i - 1 {
          if short_term_right[j].pic_order_cnt < short_term_right[j + 1].pic_order_cnt {
            short_term_right.swap(j, j + 1);
          }
        }
      }
    }

    if !long_term.is_empty() {
      for i in 0..long_term.len() - 1 {
        for j in 0..long_term.len() - i - 1 {
          if long_term[j].long_term_pic_num > long_term[j + 1].long_term_pic_num {
            long_term.swap(j, j + 1);
          }
        }
      }
    }

    for i in 0..short_term_left.len() {
      self
        .ref_pic_list1
        .push(short_term_left[i].offset_array(&self.buffer));
    }

    for i in 0..short_term_right.len() {
      self
        .ref_pic_list1
        .push(short_term_right[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self
        .ref_pic_list1
        .push(long_term[i].offset_array(&self.buffer));
    }

    let mut flag = false;

    if self.ref_pic_list1.len() > 1 && self.ref_pic_list1.len() == self.ref_pic_list0.len() {
      let length = self.ref_pic_list1.len();

      for i in 0..length {
        if self.ref_pic_list1[i] == self.ref_pic_list0[i] {
          flag = true;
        } else {
          flag = false;
          break;
        }
      }
    }

    if flag {
      self.ref_pic_list1.swap(0, 1);
    }
  }

  /// 8.2.4.3 Modification process for reference picture lists
  pub fn modification_for_reference_picture_lists(&mut self, header: &SliceHeader) {
    if !header.ref_pic_list_modification_l0.is_empty() && !self.ref_pic_list0.is_empty() {
      let mut ref_idx_l0 = 0usize;
      let mut pic_num_l0_pred = header.curr_pic_num;
      for ref_pic_list_mod in &*header.ref_pic_list_modification_l0 {
        if ref_pic_list_mod.modification_of_pic_nums_idc == 0
          || ref_pic_list_mod.modification_of_pic_nums_idc == 1
        {
          self.modification_of_reference_picture_lists_for_short_term_reference_pictures(
            &mut ref_idx_l0,
            &mut pic_num_l0_pred,
            ref_pic_list_mod.abs_diff_pic_num_minus1 as isize,
            ref_pic_list_mod.modification_of_pic_nums_idc as isize,
            header.num_ref_idx_l0_active_minus1 as isize,
            header,
          );
        } else if ref_pic_list_mod.modification_of_pic_nums_idc == 2 {
          self.modification_of_reference_picture_lists_for_long_term_reference_pictures(
            &mut ref_idx_l0,
            ref_pic_list_mod.long_term_pic_num as isize,
            header.num_ref_idx_l0_active_minus1 as isize,
          );
        } else {
          break;
        }
      }
    }
  }

  /// 8.2.4.3.1 Modification process of reference picture lists for short-term reference pictures
  pub fn modification_of_reference_picture_lists_for_short_term_reference_pictures(
    &mut self,
    ref_idx_lx: &mut usize,
    pic_num_lx_pred: &mut isize,
    abs_diff_pic_num_minus1: isize,
    modification_of_pic_nums_idc: isize,
    num_ref_idx_lx_active_minus1: isize,
    header: &SliceHeader,
  ) {
    let pic_num_lx_no_wrap;

    if modification_of_pic_nums_idc == 0 {
      if *pic_num_lx_pred - (abs_diff_pic_num_minus1 + 1) < 0 {
        pic_num_lx_no_wrap = *pic_num_lx_pred - (abs_diff_pic_num_minus1 + 1) + header.max_pic_num;
      } else {
        pic_num_lx_no_wrap = *pic_num_lx_pred - (abs_diff_pic_num_minus1 + 1);
      }
    } else if *pic_num_lx_pred + (abs_diff_pic_num_minus1 + 1) >= header.max_pic_num {
      pic_num_lx_no_wrap = *pic_num_lx_pred + (abs_diff_pic_num_minus1 + 1) - header.max_pic_num;
    } else {
      pic_num_lx_no_wrap = *pic_num_lx_pred + (abs_diff_pic_num_minus1 + 1);
    }
    *pic_num_lx_pred = pic_num_lx_no_wrap;

    let pic_num_lx = if pic_num_lx_no_wrap > header.curr_pic_num {
      pic_num_lx_no_wrap - header.max_pic_num
    } else {
      pic_num_lx_no_wrap
    };

    let length = if (num_ref_idx_lx_active_minus1 + 1) < self.ref_pic_list0.len() as isize {
      num_ref_idx_lx_active_minus1 as usize + 1
    } else {
      self.ref_pic_list0.len()
    };

    let mut c_idx = length;
    while c_idx > *ref_idx_lx {
      self.ref_pic_list0[c_idx] = self.ref_pic_list0[c_idx - 1];
      c_idx -= 1;
    }

    let mut idx = 0;
    while idx < length {
      if self.ref_pic_list0(idx).pic_num == pic_num_lx
        && self
          .ref_pic_list0(idx)
          .reference_marked_type
          .is_short_term_reference()
      {
        break;
      }
      idx += 1;
    }
    self.ref_pic_list0[*ref_idx_lx] = self.ref_pic_list0[idx];
    *ref_idx_lx += 1;
    let mut n_idx = *ref_idx_lx;
    for c_idx in *ref_idx_lx..length {
      let pic_num_f = if self
        .ref_pic_list0(c_idx)
        .reference_marked_type
        .is_short_term_reference()
      {
        self.ref_pic_list0(c_idx).pic_num
      } else {
        header.max_pic_num
      };
      if pic_num_f != pic_num_lx {
        self.ref_pic_list0[n_idx] = self.ref_pic_list0[c_idx];
        n_idx += 1;
      }
    }

    self
      .ref_pic_list0
      .drain(num_ref_idx_lx_active_minus1 as usize + 1..);
  }

  /// 8.2.4.3.2 Modification process of reference picture lists for long-term reference pictures
  pub fn modification_of_reference_picture_lists_for_long_term_reference_pictures(
    &mut self,
    ref_idx_lx: &mut usize,
    long_term_pic_num: isize,
    num_ref_idx_lx_active_minus1: isize,
  ) {
    let length = if (num_ref_idx_lx_active_minus1 as usize + 1) < self.ref_pic_list0.len() {
      num_ref_idx_lx_active_minus1 as usize + 1
    } else {
      self.ref_pic_list0.len()
    };
    let mut c_idx = length;
    while c_idx > *ref_idx_lx {
      self.ref_pic_list0[c_idx] = self.ref_pic_list0[c_idx - 1];
      c_idx -= 1;
    }

    let mut idx = 0;
    while idx < length {
      if self.ref_pic_list0(idx).long_term_pic_num == long_term_pic_num {
        break;
      }
      idx += 1;
    }
    self.ref_pic_list0[*ref_idx_lx] = self.ref_pic_list0[idx];
    *ref_idx_lx += 1;

    let mut n_idx = *ref_idx_lx;

    for c_idx in *ref_idx_lx..length {
      let long_term_pic_num_f = if self
        .ref_pic_list0(c_idx)
        .reference_marked_type
        .is_long_term_reference()
      {
        self.ref_pic_list0(c_idx).long_term_pic_num
      } else {
        0
      };

      if long_term_pic_num_f != long_term_pic_num {
        self.ref_pic_list0[n_idx] = self.ref_pic_list0[c_idx];
        n_idx += 1;
      }
    }

    self
      .ref_pic_list0
      .drain(num_ref_idx_lx_active_minus1 as usize + 1..);
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
          let pic_num_x = slice.curr_pic_num - (*difference_of_pic_nums_minus1 as isize + 1);
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
              && self.buffer[j].long_term_pic_num == *long_term_pic_num as isize
            {
              self.buffer.remove(j);
            }
          }
        }
        Mmco::ShortToLong {
          difference_of_pic_nums_minus1,
          long_term_frame_idx,
        } => {
          let pic_num_x = slice.curr_pic_num - (*difference_of_pic_nums_minus1 as isize + 1);
          for j in 0..self.buffer.len() {
            if self.buffer[j]
              .reference_marked_type
              .is_long_term_reference()
              && self.buffer[j].long_term_frame_idx == *long_term_frame_idx as isize
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
            if (self.buffer[j].long_term_frame_idx > *max_long_term_frame_idx_plus1 as isize - 1)
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
            pic.max_long_term_frame_idx = *max_long_term_frame_idx_plus1 as isize - 1;
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
            if self.buffer[j].long_term_frame_idx == *long_term_frame_idx as isize
              && self.buffer[j]
                .reference_marked_type
                .is_long_term_reference()
            {
              self.buffer.remove(j);
            }
          }

          pic.reference_marked_type = PictureMarking::LongTermReference;
          pic.long_term_frame_idx = *long_term_frame_idx as isize;
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
      }
    }

    if num_short_term + num_long_term == std::cmp::max(max_num_ref_frames as isize, 1isize)
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

    let pic_order_cnt_lsb = slice.pic_order_cnt_lsb.unwrap_or_default() as isize;
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
    } else if previous.frame_num > slice.frame_num as isize {
      self.poc.frame_num_offset = prev_frame_num_offset + slice.max_frame_num as isize;
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
      abs_frame_num = self.poc.frame_num_offset + slice.frame_num as isize;
    } else {
      abs_frame_num = 0;
    }

    if slice.nal_idc == 0 && abs_frame_num > 0 {
      abs_frame_num -= 1;
    }
    let mut pic_order_cnt_cycle_cnt = 0;
    let mut frame_num_in_pic_order_cnt_cycle = 0;
    if abs_frame_num > 0 {
      pic_order_cnt_cycle_cnt =
        (abs_frame_num - 1) / *num_ref_frames_in_pic_order_cnt_cycle as isize;
      frame_num_in_pic_order_cnt_cycle =
        (abs_frame_num - 1) % *num_ref_frames_in_pic_order_cnt_cycle as isize;
    }

    let mut expected_pic_order_cnt;
    if abs_frame_num > 0 {
      expected_pic_order_cnt = pic_order_cnt_cycle_cnt * expected_delta_per_pic_order_cnt_cycle;
      for i in 0..frame_num_in_pic_order_cnt_cycle as usize {
        expected_pic_order_cnt += offset_for_ref_frame[i];
      }
    } else {
      expected_pic_order_cnt = 0;
    }

    if slice.nal_idc == 0 {
      expected_pic_order_cnt += offset_for_non_ref_pic;
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
    } else if previous.frame_num > slice.frame_num as isize {
      self.poc.frame_num_offset = prev_frame_num_offset + slice.max_frame_num as isize;
    } else {
      self.poc.frame_num_offset = prev_frame_num_offset;
    }

    let temp_pic_order_cnt;
    if slice.nal_unit_type.is_idr() {
      temp_pic_order_cnt = 0;
    } else if slice.nal_idc == 0 {
      temp_pic_order_cnt = 2 * (self.poc.frame_num_offset + slice.frame_num as isize) - 1;
    } else {
      temp_pic_order_cnt = 2 * (self.poc.frame_num_offset + slice.frame_num as isize);
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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct PictureOrderCount {
  pub pic_order_cnt: isize,
  pub pic_order_cnt_msb: isize,
  pub pic_order_cnt_lsb: isize,
  pub top_field_order_cnt: isize,
  pub bottom_field_order_cnt: isize,
  pub frame_num_offset: isize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Picture {
  pub poc: PictureOrderCount,
  pub reference_marked_type: PictureMarking,
  pub frame_num: isize,
  pub max_frame_num: isize,
  pub long_term_frame_idx: isize,
  pub max_long_term_frame_idx: isize,
  pub pic_num: isize,
  pub long_term_pic_num: isize,
  pub frame_num_wrap: isize,
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
      reference_marked_type: PictureMarking::ShortTermReference,
      frame_num: 0,
      max_frame_num: 0,
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

impl OffsetArray for Picture {}
