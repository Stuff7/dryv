use super::*;

impl DecodedPictureBuffer {
  /// 8.2.4 Decoding process for reference picture lists construction
  pub fn reference_picture_lists_construction(&mut self, pic: &Picture) {
    self.ref_pic_list0.clear();
    self.ref_pic_list1.clear();

    self.picture_numbers(pic);
    self.reference_picture_lists(pic);
    self.modification_for_reference_picture_lists(pic);
  }

  /// 8.2.4.1 Decoding process for picture numbers
  pub fn picture_numbers(&mut self, pic: &Picture) {
    for dpb in &mut self.buffer {
      if dpb.reference_marked_type.is_short_term_reference() {
        if dpb.header.frame_num > pic.header.frame_num {
          dpb.frame_num_wrap = dpb.header.frame_num as isize - dpb.header.max_frame_num;
        } else {
          dpb.frame_num_wrap = dpb.header.frame_num as isize;
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
  pub fn reference_picture_lists(&mut self, pic: &Picture) {
    if pic.header.slice_type.is_predictive() {
      self.reference_picture_list_for_p_and_sp_slices_in_frames();
    } else if pic.header.slice_type.is_bidirectional() {
      self.reference_picture_lists_for_b_slices_in_frames(pic.pic_order_cnt);
    }

    let num_ref_idx_l0_active = pic.header.num_ref_idx_l0_active_minus1 as usize + 1;
    if self.ref_pic_list0.len() > num_ref_idx_l0_active {
      self.ref_pic_list0.drain(num_ref_idx_l0_active..);
    }

    let num_ref_idx_l1_active = pic.header.num_ref_idx_l1_active_minus1 as usize + 1;
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
      self.ref_pic_list0.push(short_term[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self.ref_pic_list0.push(long_term[i].offset_array(&self.buffer));
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
      self.ref_pic_list0.push(short_term_left[i].offset_array(&self.buffer));
    }

    for i in 0..short_term_right.len() {
      self.ref_pic_list0.push(short_term_right[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self.ref_pic_list0.push(long_term[i].offset_array(&self.buffer));
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
      self.ref_pic_list1.push(short_term_left[i].offset_array(&self.buffer));
    }

    for i in 0..short_term_right.len() {
      self.ref_pic_list1.push(short_term_right[i].offset_array(&self.buffer));
    }

    for i in 0..long_term.len() {
      self.ref_pic_list1.push(long_term[i].offset_array(&self.buffer));
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
  pub fn modification_for_reference_picture_lists(&mut self, pic: &Picture) {
    if !pic.header.ref_pic_list_modification_l0.is_empty() && !self.ref_pic_list0.is_empty() {
      let mut ref_idx_l0 = 0usize;
      let mut pic_num_l0_pred = pic.header.curr_pic_num;
      for ref_pic_list_mod in &*pic.header.ref_pic_list_modification_l0 {
        if ref_pic_list_mod.modification_of_pic_nums_idc == 0 || ref_pic_list_mod.modification_of_pic_nums_idc == 1 {
          self.modification_of_reference_picture_lists_for_short_term_reference_pictures(
            &mut ref_idx_l0,
            &mut pic_num_l0_pred,
            ref_pic_list_mod.abs_diff_pic_num_minus1 as isize,
            ref_pic_list_mod.modification_of_pic_nums_idc as isize,
            pic.header.num_ref_idx_l0_active_minus1 as isize,
            &pic.header,
          );
        } else if ref_pic_list_mod.modification_of_pic_nums_idc == 2 {
          self.modification_of_reference_picture_lists_for_long_term_reference_pictures(
            &mut ref_idx_l0,
            ref_pic_list_mod.long_term_pic_num as isize,
            pic.header.num_ref_idx_l0_active_minus1 as isize,
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

    self.ref_pic_list0.push(usize::MAX);
    let mut c_idx = length;
    while c_idx > *ref_idx_lx {
      self.ref_pic_list0[c_idx] = self.ref_pic_list0[c_idx - 1];
      c_idx -= 1;
    }

    let mut idx = 0;
    while idx < length {
      if self.ref_pic_list0(idx).pic_num == pic_num_lx && self.ref_pic_list0(idx).reference_marked_type.is_short_term_reference() {
        break;
      }
      idx += 1;
    }
    self.ref_pic_list0[*ref_idx_lx] = self.ref_pic_list0[idx];
    *ref_idx_lx += 1;
    let mut n_idx = *ref_idx_lx;
    for c_idx in *ref_idx_lx..length {
      let pic_num_f = if self.ref_pic_list0(c_idx).reference_marked_type.is_short_term_reference() {
        self.ref_pic_list0(c_idx).pic_num
      } else {
        header.max_pic_num
      };
      if pic_num_f != pic_num_lx {
        self.ref_pic_list0[n_idx] = self.ref_pic_list0[c_idx];
        n_idx += 1;
      }
    }

    self.ref_pic_list0.drain(num_ref_idx_lx_active_minus1 as usize + 1..);
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

    self.ref_pic_list0.push(0);
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
      let long_term_pic_num_f = if self.ref_pic_list0(c_idx).reference_marked_type.is_long_term_reference() {
        self.ref_pic_list0(c_idx).long_term_pic_num
      } else {
        0
      };

      if long_term_pic_num_f != long_term_pic_num {
        self.ref_pic_list0[n_idx] = self.ref_pic_list0[c_idx];
        n_idx += 1;
      }
    }

    self.ref_pic_list0.drain(num_ref_idx_lx_active_minus1 as usize + 1..);
  }
}
