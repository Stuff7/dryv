use super::*;

impl<'a> std::fmt::Display for Slice<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("- SLICE HEADER -\n\n")?;

    f.write_str(&format!("nal_idc:  {}\n", self.nal_idc))?;
    f.write_str(&format!("slice_type: {:?}\n", self.slice_type))?;
    f.write_str(&format!("first_mb_in_slice: {}\n", self.first_mb_in_slice))?;
    // f.write_str(&format!("pic_parameter_set_id: {}\n", self.pic_parameter_set_id))?;
    f.write_str(&format!("colour_plane_id: {}\n", self.color_plane_id.unwrap_or_default()))?;
    f.write_str(&format!("frame_num: {}\n", self.frame_num))?;
    f.write_str(&format!("field_pic_flag: {}\n", self.field_pic_flag as u8))?;
    f.write_str(&format!("bottom_field_flag: {}\n", self.bottom_field_flag as u8))?;
    f.write_str(&format!("idr_pic_id: {}\n", self.idr_pic_id.unwrap_or_default()))?;
    f.write_str(&format!("pic_order_cnt_lsb: {}\n", self.pic_order_cnt_lsb.unwrap_or_default()))?;
    f.write_str(&format!(
      "delta_pic_order_cnt_bottom: {}\n",
      self.delta_pic_order_cnt_bottom.unwrap_or_default()
    ))?;
    // f.write_str(&format!("delta_pic_order_cnt: {}\n", self.delta_pic_order_cnt.unwrap_or_default()))?;
    f.write_str(&format!("redundant_pic_cnt: {}\n", self.redundant_pic_cnt.unwrap_or_default()))?;
    f.write_str(&format!("direct_spatial_mv_pred_flag: {}\n", self.direct_spatial_mv_pred_flag as u8))?;
    f.write_str(&format!(
      "num_ref_idx_active_override_flag: {}\n",
      self.num_ref_idx_active_override_flag as u8
    ))?;
    f.write_str(&format!("num_ref_idx_l0_active_minus1: {}\n", self.num_ref_idx_l0_active_minus1))?;
    f.write_str(&format!("num_ref_idx_l1_active_minus1: {}\n", self.num_ref_idx_l1_active_minus1))?;
    f.write_str(&format!("cabac_init_idc: {}\n", self.cabac_init_idc.unwrap_or_default()))?;
    f.write_str(&format!("slice_qp_delta: {}\n", self.slice_qp_delta))?;
    f.write_str(&format!("sp_for_switch_flag: {}\n", self.sp_for_switch_flag as u8))?;
    f.write_str(&format!("slice_qs_delta: {}\n", self.slice_qs_delta.unwrap_or_default()))?;

    if let Some(dfc) = &self.deblocking_filter_control {
      f.write_str(&format!("disable_deblocking_filter_idc: {}\n", dfc.disable_deblocking_filter_idc))?;
      if let Some(dfcs) = &dfc.slice {
        f.write_str(&format!("slice_alpha_c0_offset_div2: {}\n", dfcs.alpha_c0_offset_div2))?;
        f.write_str(&format!("slice_beta_offset_div2: {}\n", dfcs.beta_offset_div2))?;
      }
    }

    f.write_str(&format!(
      "slice_group_change_cycle: {}\n",
      self.slice_group_change_cycle.unwrap_or_default()
    ))?;
    f.write_str(&format!("QSY: {}\n", self.qsy))?;

    if let Some(drpm) = &self.dec_ref_pic_marking {
      f.write_str(&format!("no_output_of_prior_pics_flag: {}\n", drpm.no_output_of_prior_pics_flag as u8))?;
      f.write_str(&format!("long_term_reference_flag: {}\n", drpm.long_term_reference_flag as u8))?;
      f.write_str(&format!(
        "adaptive_ref_pic_marking_mode_flag: {}\n",
        drpm.adaptive_ref_pic_marking_mode_flag as u8
      ))?;
      // f.write_str(&format!("dec_ref_pic_markings_size: {}\n", drpm.dec_ref_pic_markings_size))?;
      // f.write_str(&format!("ref_pic_list_modification_flag_l0: {}\n", drpm.ref_pic_list_modification_flag as u8_l0))?;
      // f.write_str(&format!("ref_pic_list_modification_flag_l1: {}\n", drpm.ref_pic_list_modification_flag as u8_l1))?;
      // f.write_str(&format!("ref_pic_list_modification_count_l0: {}\n", drpm.ref_pic_list_modification_count_l0))?;
      // f.write_str(&format!("ref_pic_list_modification_count_l1: {}\n", drpm.ref_pic_list_modification_count_l1))?;
    }

    f.write_str(&format!(
      "luma_log2_weight_denom: {}\n",
      self.pred_weight_table.as_ref().map(|pwt| pwt.luma_log2_weight_denom).unwrap_or_default()
    ))?;
    f.write_str(&format!(
      "chroma_log2_weight_denom: {}\n",
      self
        .pred_weight_table
        .as_ref()
        .map(|pwt| pwt.chroma_log2_weight_denom)
        .unwrap_or_default()
    ))?;
    // f.write_str(&format!("luma_weight_l0_flag: {}\n", pwt.luma_weight_l0_flag as u8))?;
    // f.write_str(&format!("chroma_weight_l0_flag: {}\n", pwt.chroma_weight_l0_flag as u8))?;
    // f.write_str(&format!("luma_weight_l1_flag: {}\n", pwt.luma_weight_l1_flag as u8))?;
    // f.write_str(&format!("chroma_weight_l1_flag: {}\n", pwt.chroma_weight_l1_flag as u8))?;

    f.write_str(&format!("MbaffFrameFlag: {}\n", self.mbaff_frame_flag as u8))?;
    // f.write_str(&format!("SliceGroupChangeRate: {}\n", self.slice_group_change_rate))?;
    f.write_str(&format!("SliceQPY: {}\n", self.sliceqpy))?;
    f.write_str(&format!("QPY_prev: {}\n", self.qpy_prev))?;
    f.write_str(&format!("FilterOffsetA: {}\n", self.filter_offset_a))?;
    f.write_str(&format!("FilterOffsetB: {}\n", self.filter_offset_b))?;
    f.write_str(&format!("PicHeightInMbs: {}\n", self.pic_height_in_mbs))?;
    f.write_str(&format!("PicSizeInMbs: {}\n", self.pic_size_in_mbs))?;
    f.write_str(&format!("MaxPicNum: {}\n", self.max_pic_num))?;
    f.write_str(&format!("CurrPicNum: {}\n", self.curr_pic_num))?;
    // f.write_str(&format!("mapUnitToSliceGroupMap: {}\n", self.map_unit_to_slice_group_map))?;
    // f.write_str(&format!("MbToSliceGroupMap: {}\n", self.mb_to_slice_group_map))?;
    // f.write_str(&format!("chroma_qp_index_offset: {}\n", self.chroma_qp_index_offset))?;
    // f.write_str(&format!("second_chroma_qp_index_offset: {}\n", self.second_chroma_qp_index_offset))?;
    f.write_str(&format!("QpBdOffsetC: {}\n", self.qp_bd_offset_c))?;

    f.write_str("\n- SLICE DATA -\n\n")?;

    f.write_str(&format!("CurrMbAddr: {}\n", self.curr_mb_addr))?;
    f.write_str(&format!("mbX: {}\n", self.mb_x))?;
    f.write_str(&format!("mbY: {}\n", self.mb_y))?;
    f.write_str(&format!("PicWidthInSamplesL: {}\n", self.pic_width_in_samples_l))?;
    f.write_str(&format!("PicHeightInSamplesL: {}\n", self.pic_height_in_samples_l))?;
    f.write_str(&format!("PicWidthInSamplesC: {}\n", self.pic_width_in_samples_c))?;
    f.write_str(&format!("PicHeightInSamplesC: {}\n", self.pic_height_in_samples_c))?;
    f.write_str(&format!("PicSizeInMbs: {}\n", self.pic_size_in_mbs))?;
    f.write_str(&format!("sliceNumber: {}\n", self.num))?;
    // f.write_str(&format!("mbCount: {}\n", self.mbCount))?;
    // f.write_str(&format!("sliceCount: {}\n", self.sliceCount))?;
    // f.write_str(&format!("PicOrderCntMsb: {}\n", self.PicOrderCntMsb))?;
    // f.write_str(&format!("PicOrderCntLsb: {}\n", self.pic_order_cnt_lsb.unwrap_or_default()))?;
    // f.write_str(&format!("TopFieldOrderCnt: {}\n", self.TopFieldOrderCnt))?;
    // f.write_str(&format!("BottomFieldOrderCnt: {}\n", self.BottomFieldOrderCnt))?;
    // f.write_str(&format!("FrameNumOffset: {}\n", self.FrameNumOffset))?;
    // f.write_str(&format!("PicOrderCnt: {}\n", self.PicOrderCnt))?;

    f.write_str("\n- SPS -\n\n")?;

    f.write_str(&format!("profile_idc: {}\n", self.sps.profile_idc))?;
    f.write_str(&format!("constraint_set0_flag: {}\n", self.sps.constraint_set0_flag as u8))?;
    f.write_str(&format!("constraint_set1_flag: {}\n", self.sps.constraint_set1_flag as u8))?;
    f.write_str(&format!("constraint_set2_flag: {}\n", self.sps.constraint_set2_flag as u8))?;
    f.write_str(&format!("constraint_set3_flag: {}\n", self.sps.constraint_set3_flag as u8))?;
    f.write_str(&format!("constraint_set4_flag: {}\n", self.sps.constraint_set4_flag as u8))?;
    f.write_str(&format!("constraint_set5_flag: {}\n", self.sps.constraint_set5_flag as u8))?;
    f.write_str(&format!("reserved_zero_2bits: {}\n", self.sps.reserved_zero_2bits as u8))?;
    f.write_str(&format!("level_idc: {}\n", self.sps.level_idc))?;
    f.write_str(&format!("seq_parameter_set_id: {}\n", self.sps.id))?;
    f.write_str(&format!("chroma_format_idc: {}\n", self.sps.chroma_format_idc))?;
    f.write_str(&format!("separate_colour_plane_flag: {}\n", self.sps.separate_color_plane_flag as u8))?;
    f.write_str(&format!("bit_depth_luma_minus8: {}\n", self.sps.bit_depth_luma_minus8))?;
    f.write_str(&format!("bit_depth_chroma_minus8: {}\n", self.sps.bit_depth_chroma_minus8))?;
    f.write_str(&format!(
      "qpprime_y_zero_transform_bypass_flag: {}\n",
      self.sps.qpprime_y_zero_transform_bypass_flag as u8
    ))?;
    f.write_str(&format!(
      "seq_scaling_matrix_present_flag: {}\n",
      self.sps.seq_scaling_matrix.is_some() as u8
    ))?;
    // seq_scaling_list_present_flag[12]: 0
    f.write_str(&format!("log2_max_frame_num_minus4: {}\n", self.sps.log2_max_frame_num_minus4))?;
    // f.write_str(&format!("MaxFrameNum: {}\n", self.sps.MaxFrameNum))?;
    f.write_str(&format!("pic_order_cnt_type: {}\n", self.sps.pic_order_cnt_type))?;
    f.write_str(&format!(
      "log2_max_pic_order_cnt_lsb_minus4: {}\n",
      self.sps.log2_max_pic_order_cnt_lsb_minus4.unwrap_or_default()
    ))?;
    // f.write_str(&format!("MaxPicOrderCntLsb: {}\n", self.sps.MaxPicOrderCntLsb))?;

    if let Some(poc1) = &self.sps.pic_order_cnt_type_one {
      f.write_str(&format!(
        "delta_pic_order_always_zero_flag: {}\n",
        poc1.delta_pic_order_always_zero_flag as u8
      ))?;
      f.write_str(&format!("offset_for_non_ref_pic: {}\n", poc1.offset_for_non_ref_pic))?;
      f.write_str(&format!("offset_for_top_to_bottom_field: {}\n", poc1.offset_for_top_to_bottom_field))?;
      f.write_str(&format!(
        "num_ref_frames_in_pic_order_cnt_cycle: {}\n",
        poc1.num_ref_frames_in_pic_order_cnt_cycle
      ))?;
      // offset_for_ref_frame[H264_MAX_OFFSET_REF_FRAME_COUNT]: 0
      f.write_str(&format!(
        "ExpectedDeltaPerPicOrderCntCycle: {}\n",
        poc1.expected_delta_per_pic_order_cnt_cycle
      ))?;
      // f.write_str(&format!("max_num_ref_frames: {}\n", poc1.max_num_ref_frames))?;
      // f.write_str(&format!(
      //   "gaps_in_frame_num_value_allowed_flag: {}\n",
      //   poc1.gaps_in_frame_num_value_allowed_flag as u8
      // ))?;
    }

    f.write_str(&format!("pic_width_in_mbs_minus1: {}\n", self.sps.pic_width_in_mbs_minus1))?;
    f.write_str(&format!("pic_height_in_map_units_minus1: {}\n", self.sps.pic_height_in_map_units_minus1))?;
    f.write_str(&format!("frame_mbs_only_flag: {}\n", self.sps.frame_mbs_only_flag as u8))?;
    f.write_str(&format!(
      "mb_adaptive_frame_field_flag: {}\n",
      self.sps.mb_adaptive_frame_field_flag as u8
    ))?;
    f.write_str(&format!("direct_8x8_inference_flag: {}\n", self.sps.direct_8x8_inference_flag as u8))?;

    if let Some(fc) = &self.sps.frame_cropping {
      f.write_str(&format!("frame_crop_left_offset: {}\n", fc.left))?;
      f.write_str(&format!("frame_crop_right_offset: {}\n", fc.right))?;
      f.write_str(&format!("frame_crop_top_offset: {}\n", fc.top))?;
      f.write_str(&format!("frame_crop_bottom_offset: {}\n", fc.bottom))?;
    }

    // f.write_str(&format!("FrameHeightInMbs: {}\n", self.sps.FrameHeightInMbs))?;
    f.write_str(&format!("PicHeightInMapUnits: {}\n", self.sps.pic_height_in_map_units_minus1 + 1))?;
    // f.write_str(&format!("PicSizeInMapUnits: {}\n", self.sps.PicSizeInMapUnits))?;
    // f.write_str(&format!("ChromaArrayType: {}\n", self.sps.ChromaArrayType))?;
    f.write_str(&format!("SubWidthC: {}\n", self.sub_width_c))?;
    f.write_str(&format!("SubHeightC: {}\n", self.sub_height_c))?;
    f.write_str(&format!("BitDepthY: {}\n", self.bit_depth_y))?;
    f.write_str(&format!("QpBdOffsetY: {}\n", self.qp_bd_offset_y))?;
    f.write_str(&format!("BitDepthC: {}\n", self.bit_depth_c))?;
    f.write_str(&format!("QpBdOffsetC: {}\n", self.qp_bd_offset_c))?;
    f.write_str(&format!("MbWidthC: {}\n", self.mb_width_c))?;
    f.write_str(&format!("MbHeightC: {}\n", self.mb_height_c))?;

    if let Some(vui) = &self.sps.vui_parameters {
      f.write_str(&format!("aspect_ratio_idc: {}\n", vui.aspect_ratio_idc))?;

      if let Some(sar) = &vui.sample_aspect_ratio {
        f.write_str(&format!("sar_width: {}\n", sar.width))?;
        f.write_str(&format!("sar_height: {}\n", sar.height))?;
      }

      f.write_str(&format!("overscan_appropriate_flag: {}\n", vui.overscan_appropriate_flag as u8))?;

      if let Some(vst) = &vui.video_signal_type {
        f.write_str(&format!("video_format: {}\n", vst.video_format))?;
        f.write_str(&format!("video_full_range_flag: {}\n", vst.video_full_range_flag as u8))?;

        if let Some(color) = &vst.color_description {
          f.write_str(&format!("colour_primaries: {}\n", color.primaries))?;
          f.write_str(&format!("transfer_characteristics: {}\n", color.transfer_characteristics))?;
          f.write_str(&format!("matrix_coefficients: {}\n", color.matrix_coefficients))?;
        }
      }

      if let Some(chroma_loc_info) = &vui.chroma_loc_info {
        f.write_str(&format!("chroma_sample_loc_type_top_field: {}\n", chroma_loc_info.top_field))?;
        f.write_str(&format!("chroma_sample_loc_type_bottom_field: {}\n", chroma_loc_info.bottom_field))?;
      }

      if let Some(timing) = &vui.timing_info {
        f.write_str(&format!("num_units_in_tick: {}\n", timing.num_units_in_tick))?;
        f.write_str(&format!("time_scale: {}\n", timing.time_scale))?;
        f.write_str(&format!("fixed_frame_rate_flag: {}\n", timing.fixed_frame_rate_flag as u8))?;
      }

      f.write_str(&format!("low_delay_hrd_flag: {}\n", vui.low_delay_hrd_flag as u8))?;
    }

    Ok(())
  }
}

impl<'a> std::fmt::Debug for Slice<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Slice")
      .field("header", &self.header)
      .field("sps", &self.sps)
      .field("pps", &self.pps)
      .field("nal_unit_type", &self.nal_unit_type)
      .field("stream", &self.stream)
      .field("cabac_init_mode", &self.cabac_init_mode)
      .field("chroma_array_type", &self.chroma_array_type)
      .field("pic_width_in_mbs", &self.pic_width_in_mbs)
      .field("pic_height_in_mbs", &self.pic_height_in_mbs)
      .field("pic_size_in_mbs", &self.pic_size_in_mbs)
      .field("sliceqpy", &self.sliceqpy)
      .field("qpy_prev", &self.qpy_prev)
      .field("qsy", &self.qsy)
      .field("qp_bd_offset_y", &self.qp_bd_offset_y)
      .field("mbaff_frame_flag", &self.mbaff_frame_flag)
      .field("last_mb_in_slice", &self.last_mb_in_slice)
      .field("prev_mb_addr", &self.prev_mb_addr)
      .field("curr_mb_addr", &self.curr_mb_addr)
      .field("sgmap", &self.sgmap)
      .field("pic_width_in_samples_l", &self.pic_width_in_samples_l)
      .field("pic_height_in_samples_l", &self.pic_height_in_samples_l)
      .field("pic_width_in_samples_c", &self.pic_width_in_samples_c)
      .field("pic_height_in_samples_c", &self.pic_height_in_samples_c)
      .field("macroblocks length", &self.macroblocks.len())
      .finish()
  }
}
