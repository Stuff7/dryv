mod display;
mod picture;
mod poc_type;
mod ref_pic;

pub use display::*;
pub use picture::*;
pub use poc_type::*;
pub use ref_pic::*;

use super::{
  header::{Mmco, SliceHeader},
  macroblock::Macroblock,
  Slice,
};
use crate::{
  byte::BitStream,
  log,
  math::OffsetArray,
  video::{
    atom::{PicOrderCntTypeOne, PictureParameterSet, SequenceParameterSet},
    cabac::CabacResult,
    frame::Frame,
    sample::{NALUnit, NALUnitType},
  },
};

#[derive(Debug)]
pub struct DecodedPictureBuffer {
  pub previous: PreviousPicture,
  pub top_field_order_cnt: isize,
  pub bottom_field_order_cnt: isize,
  pub pic_order_cnt: isize,
  pub buffer: Vec<Picture>,
  pub ref_pic_list0: Vec<usize>,
  pub ref_pic_list1: Vec<usize>,
}

impl DecodedPictureBuffer {
  pub fn new() -> Self {
    Self {
      previous: PreviousPicture::default(),
      top_field_order_cnt: 0,
      bottom_field_order_cnt: 0,
      pic_order_cnt: 0,
      buffer: Vec::with_capacity(16),
      ref_pic_list0: Vec::with_capacity(16),
      ref_pic_list1: Vec::with_capacity(16),
    }
  }

  pub fn push(&mut self, num: usize, data: &[u8], nal: &NALUnit, sps: &SequenceParameterSet, pps: &PictureParameterSet) -> CabacResult {
    let mut stream = BitStream::new(data);
    let mut pic = Picture::new(&mut stream, nal, sps, pps);
    self.init_pic(&mut pic);
    let mut slice = Slice::new(num, stream, &pic.header, nal, sps, pps, &mut pic.macroblocks);
    slice.data(self, &mut pic.frame)?;
    self.previous.copy_pic(&pic);
    if pic.nal_idc != 0 {
      self.buffer.push(pic);
    }
    Ok(())
  }

  /// 8.2.5 Decoded reference picture marking process
  pub fn init_pic(&mut self, pic: &mut Picture) {
    self.decode_pic_order_cnt_type(pic);
    if pic.header.slice_type.is_predictive() || pic.header.slice_type.is_bidirectional() {
      self.reference_picture_lists_construction(pic);
    }
    if pic.nal_unit_type.is_idr() {
      self.buffer.clear();
      self.ref_pic_list0.clear();
      self.ref_pic_list1.clear();
      if pic.header.dec_ref_pic_marking.as_ref().is_some_and(|drpm| drpm.long_term_reference_flag) {
        pic.reference_marked_type = PictureMarking::LongTermReference;
        pic.long_term_frame_idx = 0;
        pic.max_long_term_frame_idx = 0;
      } else {
        pic.reference_marked_type = PictureMarking::ShortTermReference;
        pic.max_long_term_frame_idx = -1;
      }
    } else if pic
      .header
      .dec_ref_pic_marking
      .as_ref()
      .is_some_and(|drpm| drpm.adaptive_ref_pic_marking_mode_flag)
    {
      self.adaptive_memory_control(pic);
    } else {
      self.sliding_window(pic.max_num_ref_frames);
    }

    if !pic.nal_unit_type.is_idr() && !pic.memory_management_control_operation_6_flag {
      pic.reference_marked_type = PictureMarking::ShortTermReference;
      pic.max_long_term_frame_idx = -1;
    }

    self.top_field_order_cnt = pic.top_field_order_cnt;
    self.bottom_field_order_cnt = pic.bottom_field_order_cnt;
    self.pic_order_cnt = pic.pic_order_cnt;
  }

  pub fn ref_pic_list0(&self, idx: usize) -> &Picture {
    &self.buffer[self.ref_pic_list0[idx]]
  }

  pub fn ref_pic_list1(&self, idx: usize) -> &Picture {
    &self.buffer[self.ref_pic_list1[idx]]
  }

  /// 8.2.5.4 Adaptive memory control decoded reference picture marking process
  pub fn adaptive_memory_control(&mut self, pic: &mut Picture) {
    let mmcos = pic
      .header
      .dec_ref_pic_marking
      .as_ref()
      .expect("Tried to start adaptive memory control decoded reference picture marking process with dec_ref_pic_marking as None")
      .mmcos
      .clone();
    for mmco in mmcos.iter() {
      match mmco {
        Mmco::ForgetShort {
          difference_of_pic_nums_minus1,
        } => {
          let pic_num_x = pic.header.curr_pic_num - (*difference_of_pic_nums_minus1 as isize + 1);
          for j in 0..self.buffer.len() {
            if self.buffer[j].reference_marked_type.is_short_term_reference() && self.buffer[j].pic_num == pic_num_x {
              self.buffer.remove(j);
            }
          }
        }
        Mmco::ForgetLong { long_term_pic_num } => {
          for j in 0..self.buffer.len() {
            if self.buffer[j].reference_marked_type.is_long_term_reference() && self.buffer[j].long_term_pic_num == *long_term_pic_num as isize {
              self.buffer.remove(j);
            }
          }
        }
        Mmco::ShortToLong {
          difference_of_pic_nums_minus1,
          long_term_frame_idx,
        } => {
          let pic_num_x = pic.header.curr_pic_num - (*difference_of_pic_nums_minus1 as isize + 1);
          for j in 0..self.buffer.len() {
            if self.buffer[j].reference_marked_type.is_long_term_reference() && self.buffer[j].long_term_frame_idx == *long_term_frame_idx as isize {
              self.buffer.remove(j);
            }
          }

          for j in 0..self.buffer.len() {
            if self.buffer[j].reference_marked_type.is_short_term_reference() && self.buffer[j].pic_num == pic_num_x {
              self.buffer[j].reference_marked_type = PictureMarking::LongTermReference;
            }
          }
        }
        Mmco::ForgetLongMany {
          max_long_term_frame_idx_plus1,
        } => {
          for j in 0..self.buffer.len() {
            if (self.buffer[j].long_term_frame_idx > *max_long_term_frame_idx_plus1 as isize - 1)
              && self.buffer[j].reference_marked_type.is_long_term_reference()
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
        Mmco::ThisToLong { long_term_frame_idx } => {
          for j in 0..self.buffer.len() {
            if self.buffer[j].long_term_frame_idx == *long_term_frame_idx as isize && self.buffer[j].reference_marked_type.is_long_term_reference() {
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

    if num_short_term + num_long_term == std::cmp::max(max_num_ref_frames as isize, 1isize) && num_short_term > 0 {
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
}

#[derive(Debug, Default)]
pub struct PreviousPicture {
  pub memory_management_control_operation_5_flag: bool,
  pub top_field_order_cnt: isize,
  pub pic_order_cnt_msb: isize,
  pub header_pic_order_cnt_lsb: isize,
  pub frame_num_offset: isize,
  pub header_frame_num: isize,
}

impl PreviousPicture {
  pub fn copy_pic(&mut self, pic: &Picture) {
    self.memory_management_control_operation_5_flag = pic.memory_management_control_operation_5_flag;
    self.top_field_order_cnt = pic.top_field_order_cnt;
    self.pic_order_cnt_msb = pic.pic_order_cnt_msb;
    self.header_pic_order_cnt_lsb = pic.header.pic_order_cnt_lsb.unwrap_or_default() as isize;
    self.frame_num_offset = pic.frame_num_offset;
    self.header_frame_num = pic.header.frame_num as isize;
  }
}
