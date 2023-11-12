pub mod consts;
pub mod dpb;
pub mod header;
pub mod macroblock;

use self::dpb::DecodedPictureBuffer;

use super::{
  atom::SliceGroup,
  cabac::{CabacContext, CabacError, CabacResult},
  frame::Frame,
  sample::NALUnitType,
};
use crate::{
  byte::BitStream,
  video::atom::{PictureParameterSet, SequenceParameterSet},
  video::sample::NALUnit,
};
use consts::*;
use header::*;
use macroblock::Macroblock;
use macroblock::{BlockSize, MacroblockError, MbMode, MbPosition};
use std::ops::Deref;

/// Represents a slice in an H.264 video frame.
pub struct Slice<'a> {
  /// The header information for the slice, including slice type and other metadata.
  pub header: SliceHeader,

  /// Reference to the Sequence Parameter Set (SPS) associated with the video stream.
  /// SPS contains essential information about the video sequence, such as frame dimensions and color space.
  pub sps: &'a SequenceParameterSet,

  /// Reference to the Picture Parameter Set (PPS) associated with the video stream.
  /// PPS contains parameters related to individual pictures or frames, such as reference frame settings.
  pub pps: &'a PictureParameterSet,

  /// The type of Network Abstraction Layer (NAL) unit to which the slice belongs.
  /// NAL units provide a structured representation of video data within the H.264 bitstream.
  pub nal_unit_type: NALUnitType,

  pub nal_idc: u8,

  /// Bitstream representing the encoded data of the slice.
  /// The slice's encoded data is stored in a `BitStream` for efficient parsing and processing.
  pub stream: BitStream<'a>,

  /// The Context-Adaptive Binary Arithmetic Coding (CABAC) initialization mode.
  /// CABAC is a video coding method that adapts probability models for binary decisions.
  pub cabac_init_mode: usize,

  /// Width of the picture in macroblocks (MBs).
  /// The picture width is measured in the number of macroblocks it contains.
  pub pic_width_in_mbs: u16,

  /// Height of the picture in macroblocks (MBs).
  /// The picture height is measured in the number of macroblocks it contains.
  pub pic_height_in_mbs: u16,

  /// Total size of the picture in macroblocks (MBs).
  /// The picture size represents the number of macroblocks within the picture.
  pub pic_size_in_mbs: u16,

  pub pic_width_in_samples_l: u16,

  pub pic_height_in_samples_l: u16,

  pub pic_width_in_samples_c: u16,

  pub pic_height_in_samples_c: u16,

  /// Quantization parameter for the slice.
  /// The quantization parameter affects the trade-off between compression and image quality.
  pub sliceqpy: isize,

  /// Flag indicating the presence of macroblock adaptive frame/field (MBaff) mode.
  /// In video, "interlaced" refers to a display method where each frame is divided into two fields,
  /// with odd and even lines displayed alternately. MBaff mode is used in interlaced video.
  /// When `mbaff_frame_flag` is true, it indicates that the frame is interlaced, and macroblocks
  /// are processed differently to handle this interlaced structure.
  pub mbaff_frame_flag: bool,

  pub qp_bd_offset_y: isize,

  pub qpy_prev: isize,

  pub qsy: isize,

  /// Index of the last macroblock in the slice.
  /// It helps identify the endpoint of macroblock processing within the slice.
  pub last_mb_in_slice: isize,

  /// Address of the previous macroblock within the picture.
  /// This helps establish the spatial relationship between macroblocks in the picture.
  pub prev_mb_addr: isize,

  /// Address of the current macroblock within the picture.
  /// This helps establish the spatial relationship between macroblocks in the picture.
  pub curr_mb_addr: isize,

  /// Slice Group Map (SGMap) for macroblock grouping.
  /// SGMap defines the grouping of macroblocks for various purposes, such as parallel processing.
  pub sgmap: Box<[u8]>,

  /// Collection of macroblocks contained within the slice.
  /// Macroblocks represent the fundamental coding units within a video frame.
  pub macroblocks: Box<[Macroblock]>,
}

impl<'a> Slice<'a> {
  pub fn new(
    data: &'a [u8],
    nal: &NALUnit,
    sps: &'a mut SequenceParameterSet,
    pps: &'a mut PictureParameterSet,
  ) -> Self {
    let pic_width_in_mbs;
    let mut pic_height_in_mbs;
    let pic_size_in_mbs;
    let sliceqpy;
    let mut stream = BitStream::new(data);
    let header = SliceHeader::new(&mut stream, nal, sps, pps);
    Self {
      cabac_init_mode: header.cabac_init_idc.map(|idc| idc + 1).unwrap_or(0) as usize,
      pic_width_in_mbs: {
        pic_width_in_mbs = sps.pic_width_in_mbs_minus1 + 1;
        pic_width_in_mbs
      },
      pic_height_in_mbs: {
        pic_height_in_mbs = sps.pic_height_in_map_units_minus1 + 1;
        if !sps.frame_mbs_only_flag {
          pic_height_in_mbs *= 2;
        }
        if header.field_pic_flag {
          pic_height_in_mbs /= 2;
        }
        pic_height_in_mbs
      },
      pic_size_in_mbs: {
        pic_size_in_mbs = pic_width_in_mbs * pic_height_in_mbs;
        pic_size_in_mbs
      },
      pic_width_in_samples_l: pic_width_in_mbs * 16,
      pic_height_in_samples_l: pic_height_in_mbs * 16,
      pic_width_in_samples_c: pic_width_in_mbs * header.mb_width_c as u16,
      pic_height_in_samples_c: pic_height_in_mbs * header.mb_height_c as u16,
      sliceqpy: {
        sliceqpy = 26 + pps.pic_init_qp_minus26 as isize + header.slice_qp_delta as isize;
        sliceqpy
      },
      mbaff_frame_flag: sps.mb_adaptive_frame_field_flag && !header.field_pic_flag,
      qp_bd_offset_y: 6 * sps.bit_depth_luma_minus8 as isize,
      qpy_prev: sliceqpy,
      qsy: 26
        + pps.pic_init_qs_minus26 as isize
        + header.slice_qs_delta.unwrap_or_default() as isize,
      last_mb_in_slice: 0,
      sgmap: SliceGroup::init_sgmap(
        header.slice_group_change_cycle.unwrap_or_default(),
        pic_width_in_mbs,
        sps,
        pps,
      ),
      header,
      sps,
      pps,
      nal_unit_type: nal.unit_type,
      nal_idc: nal.idc,
      stream,
      prev_mb_addr: 0,
      curr_mb_addr: 0,
      macroblocks: (0..pic_size_in_mbs).map(|_| Macroblock::empty()).collect(),
    }
  }

  pub fn mb(&self) -> &Macroblock {
    &self.macroblocks[self.curr_mb_addr as usize]
  }

  pub fn mb_mut(&mut self) -> &mut Macroblock {
    &mut self.macroblocks[self.curr_mb_addr as usize]
  }

  pub fn data(&mut self, dpb: &mut DecodedPictureBuffer, frame: &mut Frame) -> CabacResult {
    self.prev_mb_addr = -1isize;
    self.curr_mb_addr = (self.first_mb_in_slice * (1 + self.mbaff_frame_flag as u16)) as isize;
    self.last_mb_in_slice = self.curr_mb_addr;

    let skip_type = match self.slice_type {
      SliceType::B => MB_TYPE_B_SKIP,
      _ => MB_TYPE_P_SKIP,
    };
    if self.pps.entropy_coding_mode_flag {
      let mut cabac = CabacContext::new(self)?;
      dpb.decode_pic_order_cnt_type(self);
      if self.slice_type.is_predictive() || self.slice_type.is_bidirectional() {
        dpb.reference_picture_lists_construction(self);
      }
      loop {
        let mut mb_skip_flag = 0u8;
        if !self.slice_type.is_intra() {
          let save = self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag;
          let ival = if self.mbaff_frame_flag
            && (self.curr_mb_addr & 1) != 0
            && *self.macroblocks[self.curr_mb_addr as usize - 1].mb_type != skip_type
          {
            self.macroblocks[self.curr_mb_addr as usize - 1].mb_field_decoding_flag
          } else {
            self.inferred_mb_field_decoding_flag()
          };
          self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag = ival;
          mb_skip_flag = cabac.mb_skip_flag(self)?;
          self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag = save;
        }
        if mb_skip_flag != 0 {
          self.infer_skip()?;
        } else {
          if self.mbaff_frame_flag {
            let first_addr = self.curr_mb_addr & !1;
            if self.curr_mb_addr == first_addr {
              self.macroblocks[first_addr as usize].mb_field_decoding_flag =
                cabac.mb_field_decoding_flag(self)? != 0;
            } else {
              if *self.macroblocks[first_addr as usize].mb_type == skip_type {
                self.macroblocks[first_addr as usize].mb_field_decoding_flag =
                  cabac.mb_field_decoding_flag(self)? != 0
              }
              self.macroblocks[first_addr as usize + 1].mb_field_decoding_flag =
                self.macroblocks[first_addr as usize].mb_field_decoding_flag;
            }
          } else {
            self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag =
              self.field_pic_flag;
          }
          cabac.macroblock_layer(self, frame)?;
        }
        if !self.mbaff_frame_flag || (self.curr_mb_addr & 1) != 0 {
          let end_of_slice_flag = cabac.terminate(self)?;
          if end_of_slice_flag != 0 {
            self.last_mb_in_slice = self.curr_mb_addr;
            self.stream.is_byte_aligned(0);
            dpb.push(self);
            return Ok(());
          }
        }
        self.prev_mb_addr = self.curr_mb_addr;
        self.last_mb_in_slice = self.curr_mb_addr;
        self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
        if self.curr_mb_addr >= self.pic_size_in_mbs as isize {
          return Err(CabacError::from(MacroblockError::MacroblockBounds(
            self.curr_mb_addr,
            self.pic_size_in_mbs as usize,
          )));
        }
      }
    } else {
      loop {
        let mut mb_skip_run: i16;
        if !self.slice_type.is_intra() {
          mb_skip_run = self.stream.exponential_golomb();
          while mb_skip_run != 0 {
            mb_skip_run -= 1;
            if self.curr_mb_addr >= self.pic_size_in_mbs as isize {
              return Err(CabacError::from(MacroblockError::MacroblockBounds(
                self.curr_mb_addr,
                self.pic_size_in_mbs as usize,
              )));
            }
            self.last_mb_in_slice = self.curr_mb_addr;
            self.macroblocks[self.curr_mb_addr as usize].set_mb_type(skip_type);
            self.infer_skip()?;
            self.prev_mb_addr = self.curr_mb_addr;
            self.last_mb_in_slice = self.curr_mb_addr;
            self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
          }
          if !self.stream.has_bits() {
            break;
          }
        }
        if self.curr_mb_addr >= self.pic_size_in_mbs as isize {
          return Err(CabacError::from(MacroblockError::MacroblockBounds(
            self.curr_mb_addr,
            self.pic_size_in_mbs as usize,
          )));
        }
        if self.mbaff_frame_flag {
          let first_addr = self.curr_mb_addr & !1;
          if self.curr_mb_addr == first_addr {
            self.macroblocks[first_addr as usize].mb_field_decoding_flag = self.stream.bit_flag();
          } else {
            if *self.macroblocks[first_addr as usize].mb_type == skip_type {
              self.macroblocks[first_addr as usize].mb_field_decoding_flag = self.stream.bit_flag();
            }
            self.macroblocks[first_addr as usize + 1].mb_field_decoding_flag =
              self.macroblocks[first_addr as usize].mb_field_decoding_flag;
          }
        } else {
          self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag = self.field_pic_flag;
        }
        todo!("implement macroblock layer for cavlc");
        // self.last_mb_in_slice = self.curr_mb_addr;
        // if !self.stream.has_bits() {
        //   break;
        // }
        // self.prev_mb_addr = self.curr_mb_addr;
        // self.last_mb_in_slice = self.curr_mb_addr;
        // self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
        // if self.curr_mb_addr >= self.pic_size_in_mbs as isize {
        //   return Err(CabacError::from(MacroblockError::MacroblockBounds(
        //     self.curr_mb_addr,
        //     self.pic_size_in_mbs as usize,
        //   )));
        // }
      }
    }
    Ok(())
  }

  pub fn next_mb_addr(&self, mut mbaddr: isize) -> isize {
    let sg = self.mb_slice_group(mbaddr);
    mbaddr += 1;
    while mbaddr < self.pic_size_in_mbs as isize && self.mb_slice_group(mbaddr) != sg {
      mbaddr += 1;
    }
    mbaddr
  }

  pub fn inferred_mb_field_decoding_flag(&mut self) -> bool {
    if self.mbaff_frame_flag {
      let mb_a = self.mb_nb_p(MbPosition::A, 0);
      let mb_b = self.mb_nb_p(MbPosition::B, 0);
      if mb_a.mb_type.is_available() {
        mb_a.mb_field_decoding_flag
      } else if mb_b.mb_type.is_available() {
        mb_b.mb_field_decoding_flag
      } else {
        false
      }
    } else {
      self.field_pic_flag
    }
  }

  pub fn infer_skip(&mut self) -> CabacResult {
    let skip_type = if self.slice_type.is_bidirectional() {
      MB_TYPE_B_SKIP
    } else {
      MB_TYPE_P_SKIP
    };
    if self.mbaff_frame_flag {
      if (self.curr_mb_addr & 1) != 0 {
        if self.macroblocks[self.curr_mb_addr as usize & !1]
          .mb_type
          .is_skip()
        {
          let val = self.inferred_mb_field_decoding_flag();
          self.macroblocks[self.curr_mb_addr as usize - 1].mb_field_decoding_flag = val;
        }
        self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag =
          self.macroblocks[self.curr_mb_addr as usize - 1].mb_field_decoding_flag;
      }
    } else {
      self.macroblocks[self.curr_mb_addr as usize].mb_field_decoding_flag = self.field_pic_flag;
    }
    {
      let mb = &mut self.macroblocks[self.curr_mb_addr as usize];
      mb.set_mb_type(skip_type);
      mb.mb_qp_delta = 0;
      mb.transform_size_8x8_flag = 0;
      mb.coded_block_pattern = 0;
      mb.intra_chroma_pred_mode = 0;
    }
    self.infer_intra(0);
    self.infer_intra(1);
    let mb = &mut self.macroblocks[self.curr_mb_addr as usize];
    for i in 0..17 {
      mb.coded_block_flag[0][i] = 0;
      mb.coded_block_flag[1][i] = 0;
      mb.coded_block_flag[2][i] = 0;
    }
    for i in 0..16 {
      mb.total_coeff[0][i] = 0;
      mb.total_coeff[1][i] = 0;
      mb.total_coeff[2][i] = 0;
    }
    Ok(())
  }

  pub fn infer_intra(&mut self, which: usize) {
    let mb = &mut self.macroblocks[self.curr_mb_addr as usize];
    for i in 0..4 {
      mb.ref_idx[which][i] = 0;
    }
    for i in 0..16 {
      mb.mvd[which][i][0] = 0;
      mb.mvd[which][i][1] = 0;
    }
  }

  pub fn inter_filter<'b>(&self, mb: &'b Macroblock, inter: u8) -> &'b Macroblock {
    if inter == 0
      && self.pps.constrained_intra_pred_flag
      && matches!(self.nal_unit_type, NALUnitType::DataPartitionA)
      && self.mb().mb_type.is_inter()
    {
      Macroblock::unavailable(1)
    } else {
      mb
    }
  }

  pub fn mb_nb(&self, position: MbPosition, inter: u8) -> CabacResult<&Macroblock> {
    let mbp = self.mb_nb_p(position, inter);
    let mbt = &self.macroblocks[self.curr_mb_addr as usize];
    Ok(match position {
      MbPosition::This => mbp,
      MbPosition::A => {
        if self.mbaff_frame_flag
          && mbp.mb_type.is_available()
          && (self.curr_mb_addr & 1) != 0
          && mbp.mb_field_decoding_flag == mbt.mb_field_decoding_flag
        {
          mbp.offset(&self.macroblocks, 1)?
        } else {
          mbp
        }
      }
      MbPosition::B => {
        if self.mbaff_frame_flag {
          if mbt.mb_field_decoding_flag {
            if mbp.mb_type.is_unavailable()
              || (self.curr_mb_addr & 1 == 0 && mbp.mb_field_decoding_flag)
            {
              mbp
            } else {
              mbp.offset(&self.macroblocks, 1)?
            }
          } else if (self.curr_mb_addr & 1) != 0 {
            mbt.offset(&self.macroblocks, -1)?
          } else if mbp.mb_type.is_available() {
            mbp.offset(&self.macroblocks, 1)?
          } else {
            mbp
          }
        } else {
          mbp
        }
      }
      _ => panic!("Slice::mb_nb received bad position {position:?}"),
    })
  }

  pub fn mb_nb_b(
    &self,
    position: MbPosition,
    mut block_size: BlockSize,
    inter: u8,
    idx: isize,
    pidx: &mut isize,
  ) -> CabacResult<&Macroblock> {
    let mb_p = self.mb_nb_p(position, inter);
    let mb_o = self.mb_nb(position, inter)?;
    let mb_t = self.mb();
    if self.chroma_array_type == 1 && block_size.is_chroma() {
      block_size = BlockSize::B8x8;
    }
    let mode = MbMode::new(mb_t, mb_p);
    let par = self.curr_mb_addr & 1;
    match position {
      MbPosition::A => match block_size {
        BlockSize::B4x4 => {
          if (idx & 1) != 0 {
            *pidx = idx - 1;
            Ok(mb_t)
          } else if (idx & 4) != 0 {
            *pidx = idx - 3;
            Ok(mb_t)
          } else {
            match mode {
              MbMode::Same => {
                *pidx = idx + 5;
                Ok(mb_o)
              }
              MbMode::FrameFromField => {
                *pidx = (idx >> 2 & 2) + par * 8 + 5;
                Ok(mb_o)
              }
              MbMode::FieldFromFrame => {
                *pidx = (idx << 2 & 8) + 5;
                Ok(mb_p.offset(&self.macroblocks, idx >> 3)?)
              }
            }
          }
        }
        BlockSize::Chroma => {
          if (idx & 1) != 0 {
            *pidx = idx - 1;
            Ok(mb_t)
          } else {
            match mode {
              MbMode::Same => {
                *pidx = idx + 1;
                Ok(mb_o)
              }
              MbMode::FrameFromField => {
                *pidx = (idx >> 1 & 2) + par * 4 + 1;
                Ok(mb_o)
              }
              MbMode::FieldFromFrame => {
                *pidx = (idx << 1 & 4) + 1;
                Ok(mb_p.offset(&self.macroblocks, idx >> 2)?)
              }
            }
          }
        }
        BlockSize::B8x8 => {
          if (idx & 1) != 0 {
            *pidx = idx - 1;
            Ok(mb_t)
          } else {
            match mode {
              MbMode::Same => {
                *pidx = idx + 1;
                Ok(mb_o)
              }
              MbMode::FrameFromField => {
                *pidx = par * 2 + 1;
                Ok(mb_o)
              }
              MbMode::FieldFromFrame => {
                *pidx = 1;
                Ok(mb_p.offset(&self.macroblocks, idx >> 1)?)
              }
            }
          }
        }
      },
      MbPosition::B => match block_size {
        BlockSize::B4x4 => {
          if (idx & 2) != 0 {
            *pidx = idx - 2;
            Ok(mb_t)
          } else if (idx & 8) != 0 {
            *pidx = idx - 6;
            Ok(mb_t)
          } else {
            *pidx = idx + 10;
            Ok(mb_o)
          }
        }
        BlockSize::Chroma => {
          if (idx & 6) != 0 {
            *pidx = idx - 2;
            Ok(mb_t)
          } else {
            *pidx = idx + 6;
            Ok(mb_o)
          }
        }
        BlockSize::B8x8 => {
          if (idx & 2) != 0 {
            *pidx = idx - 2;
            Ok(mb_t)
          } else {
            *pidx = idx + 2;
            Ok(mb_o)
          }
        }
      },
      position => panic!("Invalid position passed to mb_nb_b {position:?}"),
    }
  }

  /// 6.4.9  Derivation process for neighbouring macroblock addresses and their availability
  /// 6.4.10 Derivation process for neighbouring macroblock addresses and their availability in MBAFF frames
  pub fn mb_nb_p(&self, position: MbPosition, inter: u8) -> &Macroblock {
    let mut mbaddr = self.curr_mb_addr;
    if self.mbaff_frame_flag {
      mbaddr /= 2;
    }
    let pic_width_in_mbs = self.pic_width_in_mbs as isize;
    match position {
      MbPosition::This => return &self.macroblocks[self.curr_mb_addr as usize],
      MbPosition::A => {
        if (mbaddr % pic_width_in_mbs) == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr -= 1;
      }
      MbPosition::B => {
        mbaddr -= pic_width_in_mbs;
      }
      MbPosition::C => {
        if ((mbaddr + 1) % pic_width_in_mbs) == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr -= pic_width_in_mbs - 1;
      }
      MbPosition::D => {
        if (mbaddr % pic_width_in_mbs) == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr -= pic_width_in_mbs + 1;
      }
    }
    if self.mbaff_frame_flag {
      mbaddr *= 2;
    }
    if !self.mb_available(mbaddr) {
      return Macroblock::unavailable(inter);
    }
    &self.macroblocks[mbaddr as usize]
  }

  pub fn mb_available(&self, mbaddr: isize) -> bool {
    if mbaddr < (self.first_mb_in_slice * (self.mbaff_frame_flag as u16 + 1)) as isize
      || mbaddr > self.curr_mb_addr
    {
      return false;
    }
    self.mb_slice_group(mbaddr) == self.mb_slice_group(self.curr_mb_addr)
  }

  /// 8.2.2.8 Specification for conversion of map unit to slice group map to macroblock to slice group map
  pub fn mb_slice_group(&self, mbaddr: isize) -> u8 {
    if mbaddr >= self.pic_size_in_mbs as isize || self.pps.slice_group.is_none() {
      return 0;
    }
    let mbaddr = mbaddr as usize;
    if self.sps.frame_mbs_only_flag || self.field_pic_flag {
      self.sgmap[mbaddr]
    } else if !self.mbaff_frame_flag {
      self.sgmap[mbaddr / 2]
    } else {
      let pic_width_in_mbs = self.pic_width_in_mbs as usize;
      let x = mbaddr % pic_width_in_mbs;
      let y = mbaddr / pic_width_in_mbs;
      self.sgmap[y / 2 * pic_width_in_mbs + x]
    }
  }
}

impl<'a> Deref for Slice<'a> {
  type Target = SliceHeader;
  fn deref(&self) -> &Self::Target {
    &self.header
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
