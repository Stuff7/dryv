use crate::log;

use super::*;

impl DecodedPictureBuffer {
  /// 8.2.1 Decoding process for picture order count
  pub fn decode_pic_order_cnt_type(&mut self, pic: &mut Picture) {
    if pic.pic_order_cnt_type == 0 {
      self.poc_type_0(pic);
    } else if pic.pic_order_cnt_type == 1 {
      self.poc_type_1(pic);
    } else if pic.pic_order_cnt_type == 2 {
      self.poc_type_2(pic);
    }

    pic.pic_order_cnt = std::cmp::min(pic.top_field_order_cnt, pic.bottom_field_order_cnt);
  }

  /// 8.2.1.1 Decoding process for picture order count type 0
  pub fn poc_type_0(&mut self, pic: &mut Picture) {
    let prev_pic_order_cnt_msb;
    let prev_pic_order_cnt_lsb;

    if pic.nal_unit_type.is_idr() {
      prev_pic_order_cnt_msb = 0;
      prev_pic_order_cnt_lsb = 0;
    } else if let Some(previous) = self.buffer.last() {
      if previous.memory_management_control_operation_5_flag {
        prev_pic_order_cnt_msb = 0;
        prev_pic_order_cnt_lsb = previous.top_field_order_cnt;
      } else {
        prev_pic_order_cnt_msb = previous.pic_order_cnt_msb;
        prev_pic_order_cnt_lsb = previous.header.pic_order_cnt_lsb.unwrap_or_default() as isize;
      }
    } else {
      unreachable!();
    }

    let h_pic_order_cnt_lsb = pic.header.pic_order_cnt_lsb.unwrap_or_default() as isize;
    if h_pic_order_cnt_lsb < prev_pic_order_cnt_lsb && ((prev_pic_order_cnt_lsb - h_pic_order_cnt_lsb) >= (pic.header.max_pic_order_cnt_lsb / 2)) {
      pic.pic_order_cnt_msb = prev_pic_order_cnt_msb + pic.header.max_pic_order_cnt_lsb;
    } else if (h_pic_order_cnt_lsb > prev_pic_order_cnt_lsb)
      && ((pic.pic_order_cnt_lsb - prev_pic_order_cnt_lsb) > (pic.header.max_pic_order_cnt_lsb / 2))
    {
      pic.pic_order_cnt_msb = prev_pic_order_cnt_msb - pic.header.max_pic_order_cnt_lsb;
    } else {
      pic.pic_order_cnt_msb = prev_pic_order_cnt_msb;
    }

    pic.top_field_order_cnt = pic.pic_order_cnt_msb + h_pic_order_cnt_lsb;
    pic.bottom_field_order_cnt = pic.top_field_order_cnt + pic.header.delta_pic_order_cnt_bottom.unwrap_or_default();
  }

  /// 8.2.1.2 Decoding process for picture order count type 1
  pub fn poc_type_1(&mut self, pic: &mut Picture) {
    let frame_num_offset;
    let memory_management_control_operation_5_flag;
    let frame_num;
    if let Some(previous) = self.buffer.last() {
      frame_num_offset = previous.frame_num_offset;
      memory_management_control_operation_5_flag = previous.memory_management_control_operation_5_flag;
      frame_num = previous.header.frame_num;
    } else {
      frame_num_offset = 0;
      memory_management_control_operation_5_flag = false;
      frame_num = 0;
    }
    let mut prev_frame_num_offset = 0;
    if !pic.nal_unit_type.is_idr() {
      if memory_management_control_operation_5_flag {
        prev_frame_num_offset = 0;
      } else {
        prev_frame_num_offset = frame_num_offset;
      }
    }

    if pic.nal_unit_type.is_idr() {
      pic.frame_num_offset = 0;
    } else if frame_num > pic.header.frame_num {
      pic.frame_num_offset = prev_frame_num_offset + pic.header.max_frame_num;
    } else {
      pic.frame_num_offset = prev_frame_num_offset;
    }

    let PicOrderCntTypeOne {
      num_ref_frames_in_pic_order_cnt_cycle,
      expected_delta_per_pic_order_cnt_cycle,
      offset_for_ref_frame,
      offset_for_non_ref_pic,
      offset_for_top_to_bottom_field,
      ..
    } = pic
      .pic_order_cnt_type_one
      .as_ref()
      .expect("Picture order count type 1 decoding process started but pic_order_cnt_type is not 1");
    let mut abs_frame_num;
    if *num_ref_frames_in_pic_order_cnt_cycle != 0 {
      abs_frame_num = pic.frame_num_offset + pic.header.frame_num as isize;
    } else {
      abs_frame_num = 0;
    }

    if pic.nal_idc == 0 && abs_frame_num > 0 {
      abs_frame_num -= 1;
    }
    let mut pic_order_cnt_cycle_cnt = 0;
    let mut frame_num_in_pic_order_cnt_cycle = 0;
    if abs_frame_num > 0 {
      pic_order_cnt_cycle_cnt = (abs_frame_num - 1) / *num_ref_frames_in_pic_order_cnt_cycle as isize;
      frame_num_in_pic_order_cnt_cycle = (abs_frame_num - 1) % *num_ref_frames_in_pic_order_cnt_cycle as isize;
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

    if pic.nal_idc == 0 {
      expected_pic_order_cnt += offset_for_non_ref_pic;
    }

    let delta_pic_order_cnt = pic
      .header
      .delta_pic_order_cnt
      .expect("No delta_pic_order_cnt found for pic_order_cnt_type 1");
    if !pic.header.field_pic_flag {
      pic.bottom_field_order_cnt = pic.top_field_order_cnt + offset_for_top_to_bottom_field + delta_pic_order_cnt.1.unwrap_or_default();
      pic.top_field_order_cnt = expected_pic_order_cnt + delta_pic_order_cnt.0;
    } else if !pic.header.bottom_field_flag {
      pic.top_field_order_cnt = expected_pic_order_cnt + delta_pic_order_cnt.0;
    } else {
      pic.bottom_field_order_cnt = expected_pic_order_cnt + offset_for_top_to_bottom_field + delta_pic_order_cnt.0;
    }
  }

  /// 8.2.1.3 Decoding process for picture order count type 2
  pub fn poc_type_2(&mut self, pic: &mut Picture) {
    let frame_num_offset;
    let memory_management_control_operation_5_flag;
    let frame_num;
    if let Some(previous) = self.buffer.last() {
      frame_num_offset = previous.frame_num_offset;
      memory_management_control_operation_5_flag = previous.memory_management_control_operation_5_flag;
      frame_num = previous.header.frame_num;
    } else {
      frame_num_offset = 0;
      memory_management_control_operation_5_flag = false;
      frame_num = 0;
    }
    let mut prev_frame_num_offset = 0;
    if !pic.nal_unit_type.is_idr() {
      if memory_management_control_operation_5_flag {
        prev_frame_num_offset = 0;
      } else {
        prev_frame_num_offset = frame_num_offset;
      }
    }

    if pic.nal_unit_type.is_idr() {
      pic.frame_num_offset = 0;
    } else if frame_num > pic.header.frame_num {
      pic.frame_num_offset = prev_frame_num_offset + pic.header.max_frame_num;
    } else {
      pic.frame_num_offset = prev_frame_num_offset;
    }

    let temp_pic_order_cnt;
    if pic.nal_unit_type.is_idr() {
      temp_pic_order_cnt = 0;
    } else if pic.nal_idc == 0 {
      temp_pic_order_cnt = 2 * (pic.frame_num_offset + pic.header.frame_num as isize) - 1;
    } else {
      temp_pic_order_cnt = 2 * (pic.frame_num_offset + pic.header.frame_num as isize);
    }

    if !pic.header.field_pic_flag {
      pic.top_field_order_cnt = temp_pic_order_cnt;
      pic.bottom_field_order_cnt = temp_pic_order_cnt;
    } else if pic.header.bottom_field_flag {
      pic.bottom_field_order_cnt = temp_pic_order_cnt;
    } else {
      pic.top_field_order_cnt = temp_pic_order_cnt;
    }
  }
}
