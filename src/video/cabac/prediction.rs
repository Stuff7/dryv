use super::*;

impl CabacContext {
  pub fn mb_pred(&mut self, slice: &mut Slice) -> CabacResult {
    if *slice.mb().mb_type < MB_TYPE_P_L0_16X16 {
      if !slice.mb().mb_type.is_intra_16x16() {
        if slice.mb().transform_size_8x8_flag == 0 {
          for i in 0..16 {
            slice.mb_mut().prev_intra4x4_pred_mode_flag[i] = self.prev_intra_pred_mode_flag(slice)?;
            if slice.mb().prev_intra4x4_pred_mode_flag[i] == 0 {
              slice.mb_mut().rem_intra4x4_pred_mode[i] = self.rem_intra_pred_mode(slice)?;
            }
          }
        } else {
          for i in 0..4 {
            slice.mb_mut().prev_intra8x8_pred_mode_flag[i] = self.prev_intra_pred_mode_flag(slice)?;
            if slice.mb().prev_intra8x8_pred_mode_flag[i] == 0 {
              slice.mb_mut().rem_intra8x8_pred_mode[i] = self.rem_intra_pred_mode(slice)?;
            }
          }
        }
      }
      if slice.chroma_array_type == 1 || slice.chroma_array_type == 2 {
        slice.mb_mut().intra_chroma_pred_mode = self.intra_chroma_pred_mode(slice)?;
      } else {
        slice.mb_mut().intra_chroma_pred_mode = 0;
      }
      slice.infer_intra(0);
      slice.infer_intra(1);
    } else if !slice.mb().mb_type.is_b_direct_16x16() {
      let mut ifrom = [0; 4];
      let mut pmode = [0; 4];
      ifrom[0] = -1isize as usize;
      pmode[0] = MB_PART_INFO[*slice.mb().mb_type as usize][1];
      let mb_type = *slice.mb().mb_type as usize;
      match MB_PART_INFO[mb_type][0] {
        0 => {
          // 16x16
          ifrom[1] = 0;
          ifrom[2] = 0;
          ifrom[3] = 0;
        }
        1 => {
          // 16x8
          ifrom[1] = 0;
          ifrom[2] = -1isize as usize;
          ifrom[3] = 2;
          pmode[2] = MB_PART_INFO[mb_type][2];
        }
        2 => {
          ifrom[1] = -1isize as usize;
          ifrom[2] = 0;
          ifrom[3] = 1;
          pmode[1] = MB_PART_INFO[mb_type][2];
        }
        _ => unreachable!(),
      }
      let mut max = slice.num_ref_idx_l0_active_minus1;
      if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
        max *= 2;
        max += 1;
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 1) != 0 {
            slice.mb_mut().ref_idx[0][i] = self.ref_idx(slice, i as isize, 0, max)?;
          } else {
            slice.mb_mut().ref_idx[0][i] = 0;
          }
        } else {
          slice.mb_mut().ref_idx[0][i] = slice.mb().ref_idx[0][ifrom[i]];
        }
      }
      max = slice.num_ref_idx_l1_active_minus1;
      if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
        max *= 2;
        max += 1;
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 2) != 0 {
            slice.mb_mut().ref_idx[1][i] = self.ref_idx(slice, i as isize, 1, max)?;
          } else {
            slice.mb_mut().ref_idx[1][i] = 0;
          }
        } else {
          slice.mb_mut().ref_idx[1][i] = slice.mb().ref_idx[1][ifrom[i]];
        }
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 1) != 0 {
            slice.mb_mut().mvd[0][i * 4][0] = self.mvd(slice, i as isize * 4, 0, 0)?;
            slice.mb_mut().mvd[0][i * 4][1] = self.mvd(slice, i as isize * 4, 1, 0)?;
          } else {
            slice.mb_mut().mvd[0][i * 4][0] = 0;
            slice.mb_mut().mvd[0][i * 4][1] = 0;
          }
        } else {
          slice.mb_mut().mvd[0][i * 4][0] = slice.mb().mvd[0][ifrom[i] * 4][0];
          slice.mb_mut().mvd[0][i * 4][1] = slice.mb().mvd[0][ifrom[i] * 4][1];
        }
        for j in 1..4 {
          slice.mb_mut().mvd[0][i * 4 + j][0] = slice.mb().mvd[0][i * 4][0];
          slice.mb_mut().mvd[0][i * 4 + j][1] = slice.mb().mvd[0][i * 4][1];
        }
      }
      for i in 0..4 {
        if ifrom[i] == -1isize as usize {
          if (pmode[i] & 2) != 0 {
            slice.mb_mut().mvd[1][i * 4][0] = self.mvd(slice, i as isize * 4, 0, 1)?;
            slice.mb_mut().mvd[1][i * 4][1] = self.mvd(slice, i as isize * 4, 1, 1)?;
          } else {
            slice.mb_mut().mvd[1][i * 4][0] = 0;
            slice.mb_mut().mvd[1][i * 4][1] = 0;
          }
        } else {
          slice.mb_mut().mvd[1][i * 4][0] = slice.mb().mvd[1][ifrom[i] * 4][0];
          slice.mb_mut().mvd[1][i * 4][1] = slice.mb().mvd[1][ifrom[i] * 4][1];
        }
        for j in 1..4 {
          slice.mb_mut().mvd[1][i * 4 + j][0] = slice.mb().mvd[1][i * 4][0];
          slice.mb_mut().mvd[1][i * 4 + j][1] = slice.mb().mvd[1][i * 4][1];
        }
      }
      slice.mb_mut().intra_chroma_pred_mode = 0;
    } else {
      slice.mb_mut().intra_chroma_pred_mode = 0;
      slice.infer_intra(0);
      slice.infer_intra(1);
    }
    Ok(())
  }

  pub fn sub_mb_pred(&mut self, slice: &mut Slice) -> CabacResult {
    let mut pmode = [0; 4];
    let mut ifrom = [0; 16];
    for i in 0..4 {
      let sub_mb_type = self.sub_mb_type(slice)?;
      slice.mb_mut().set_sub_mb_type(i, sub_mb_type);
      pmode[i] = SUB_MB_PART_INFO[*slice.mb().sub_mb_type[i] as usize][1];
      let sm = SUB_MB_PART_INFO[*slice.mb().sub_mb_type[i] as usize][0];
      ifrom[i * 4] = -1isize as usize;
      match sm {
        0 => {
          ifrom[i * 4 + 1] = i * 4;
          ifrom[i * 4 + 2] = i * 4;
          ifrom[i * 4 + 3] = i * 4;
        }
        1 => {
          ifrom[i * 4 + 1] = i * 4;
          ifrom[i * 4 + 2] = -1isize as usize;
          ifrom[i * 4 + 3] = i * 4 + 2;
        }
        2 => {
          ifrom[i * 4 + 1] = -1isize as usize;
          ifrom[i * 4 + 2] = i * 4;
          ifrom[i * 4 + 3] = i * 4 + 1;
        }
        3 => {
          ifrom[i * 4 + 1] = -1isize as usize;
          ifrom[i * 4 + 2] = -1isize as usize;
          ifrom[i * 4 + 3] = -1isize as usize;
        }
        _ => unreachable!(),
      }
    }
    let mut max = slice.num_ref_idx_l0_active_minus1;
    if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
      max *= 2;
      max += 1;
    }
    for (i, pmode) in pmode.iter().enumerate() {
      if (pmode & 1) != 0 && !slice.mb().mb_type.is_p_8x8ref0() {
        slice.mb_mut().ref_idx[0][i] = self.ref_idx(slice, i as isize, 0, max)?;
      } else {
        slice.mb_mut().ref_idx[0][i] = 0;
      }
    }
    max = slice.num_ref_idx_l1_active_minus1;
    if slice.mbaff_frame_flag && slice.mb().mb_field_decoding_flag {
      max *= 2;
      max += 1;
    }
    for (i, pmode) in pmode.iter().enumerate() {
      if (pmode & 2) != 0 {
        slice.mb_mut().ref_idx[1][i] = self.ref_idx(slice, i as isize, 1, max)?;
      } else {
        slice.mb_mut().ref_idx[1][i] = 0;
      }
    }
    for i in 0..16 {
      if ifrom[i] == -1isize as usize {
        if (pmode[i / 4] & 1) != 0 {
          slice.mb_mut().mvd[0][i][0] = self.mvd(slice, i as isize, 0, 0)?;
          slice.mb_mut().mvd[0][i][1] = self.mvd(slice, i as isize, 1, 0)?;
        } else {
          slice.mb_mut().mvd[0][i][0] = 0;
          slice.mb_mut().mvd[0][i][1] = 0;
        }
      } else {
        slice.mb_mut().mvd[0][i][0] = slice.mb().mvd[0][ifrom[i]][0];
        slice.mb_mut().mvd[0][i][1] = slice.mb().mvd[0][ifrom[i]][1];
      }
    }
    for i in 0..16 {
      if ifrom[i] == -1isize as usize {
        if (pmode[i / 4] & 2) != 0 {
          slice.mb_mut().mvd[1][i][0] = self.mvd(slice, i as isize, 0, 1)?;
          slice.mb_mut().mvd[1][i][1] = self.mvd(slice, i as isize, 1, 1)?;
        } else {
          slice.mb_mut().mvd[1][i][0] = 0;
          slice.mb_mut().mvd[1][i][1] = 0;
        }
      } else {
        slice.mb_mut().mvd[1][i][0] = slice.mb().mvd[1][ifrom[i]][0];
        slice.mb_mut().mvd[1][i][1] = slice.mb().mvd[1][ifrom[i]][1];
      }
    }
    slice.mb_mut().intra_chroma_pred_mode = 0;
    Ok(())
  }

  pub fn intra_chroma_pred_mode(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let cond_term_flag_a = (slice.mb_nb(MbPosition::A, 0)?.intra_chroma_pred_mode != 0) as isize;
    let cond_term_flag_b = (slice.mb_nb(MbPosition::B, 0)?.intra_chroma_pred_mode != 0) as isize;
    let ctx_idx = [
      CTXIDX_INTRA_CHROMA_PRED_MODE + cond_term_flag_a + cond_term_flag_b,
      CTXIDX_INTRA_CHROMA_PRED_MODE + 3,
    ];
    self.tu(slice, &ctx_idx, 2, 3)
  }

  pub fn rem_intra_pred_mode(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    let bit = [
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
      self.decision(slice, CTXIDX_REM_INTRA_PRED_MODE)?,
    ];
    Ok(bit[0] | bit[1] << 1 | bit[2] << 2)
  }

  pub fn prev_intra_pred_mode_flag(&mut self, slice: &mut Slice) -> CabacResult<u8> {
    self.decision(slice, CTXIDX_PREV_INTRA_PRED_MODE_FLAG)
  }
}
