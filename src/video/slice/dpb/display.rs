use std::fmt::Display;

use super::*;

impl Display for Picture {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("PicSizeInMbs: {}\n", self.header.pic_size_in_mbs))?;
    f.write_str(&format!("PicWidthInSamplesL: {}\n", self.header.pic_width_in_samples_l))?;
    f.write_str(&format!("PicHeightInSamplesL: {}\n", self.header.pic_height_in_samples_l))?;
    f.write_str(&format!("PicWidthInSamplesC: {}\n", self.header.pic_width_in_samples_c))?;
    f.write_str(&format!("PicHeightInSamplesC: {}\n", self.header.pic_height_in_samples_c))?;
    f.write_str(&format!("slice_type: {:?}\n", self.header.slice_type))?;
    f.write_str(&format!("FrameNumOffset: {}\n", self.frame_num_offset))?;
    f.write_str(&format!("pic_order_cnt_lsb: {}\n", self.header.pic_order_cnt_lsb.unwrap_or_default()))?;
    f.write_str(&format!("frame_num: {}\n", self.header.frame_num))?;
    f.write_str(&format!("field_pic_flag: {}\n", self.header.field_pic_flag as u8))?;
    f.write_str(&format!("PicOrderCntMsb: {}\n", self.pic_order_cnt_msb))?;
    f.write_str(&format!("PicOrderCntLsb: {}\n", self.pic_order_cnt_lsb))?;
    f.write_str(&format!("TopFieldOrderCnt: {}\n", self.top_field_order_cnt))?;
    f.write_str(&format!("BottomFieldOrderCnt: {}\n", self.bottom_field_order_cnt))?;
    f.write_str(&format!("PicOrderCnt: {}\n", self.pic_order_cnt))?;
    f.write_str(&format!("MaxFrameNum: {}\n", self.header.max_frame_num))?;
    f.write_str(&format!("FrameNum: {}\n", self.header.frame_num))?;
    f.write_str(&format!("FrameNumWrap: {}\n", self.frame_num_wrap))?;
    f.write_str(&format!("PicNum: {}\n", self.pic_num))?;
    f.write_str(&format!("LongTermPicNum: {}\n", self.long_term_pic_num))?;
    f.write_str(&format!("MaxLongTermFrameIdx: {}\n", self.max_long_term_frame_idx))?;
    f.write_str(&format!("LongTermFrameIdx: {}\n", self.long_term_frame_idx))?;
    f.write_str(&format!("reference_marked_type: {:?}\n", self.reference_marked_type))?;
    f.write_str(&format!(
      "memory_management_control_operation_5_flag: {}\n",
      self.memory_management_control_operation_5_flag as u8
    ))?;
    f.write_str(&format!(
      "memory_management_control_operation_6_flag: {}\n",
      self.memory_management_control_operation_6_flag as u8
    ))?;
    Ok(())
  }
}

impl Display for DecodedPictureBuffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in 0..self.buffer.len() {
      f.write_str(&format!("# dpb[{i}]\n{}\n", self.buffer[i]))?;
    }
    for i in 0..self.ref_pic_list0.len() {
      f.write_str(&format!("# RefPicList0[{i}]\n{}\n", self.ref_pic_list0(i)))?;
    }
    for i in 0..self.ref_pic_list1.len() {
      f.write_str(&format!("# RefPicList1[{i}]\n{}\n", self.ref_pic_list1(i)))?;
    }
    Ok(())
  }
}
