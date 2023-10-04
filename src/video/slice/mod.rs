pub mod consts;
pub mod header;
pub mod macroblock;

use super::{
  cabac::{CabacContext, CabacError, CabacResult},
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

pub struct Slice<'a> {
  pub header: SliceHeader,
  pub sps: &'a SequenceParameterSet,
  pub pps: &'a PictureParameterSet,
  pub nal_unit_type: NALUnitType,
  pub stream: BitStream<'a>,
  pub cabac_init_mode: usize,
  pub chroma_array_type: u16,
  pub pic_width_in_mbs: u16,
  pub pic_height_in_mbs: u16,
  pub pic_size_in_mbs: u16,
  pub sliceqpy: i16,
  pub mbaff_frame_flag: bool,
  pub last_mb_in_slice: usize,
  /// Previous macroblock address
  pub prev_mb_addr: usize,
  /// Current macroblock address
  pub curr_mb_addr: usize,
  /// Macroblocks
  pub sgmap: &'a [u8],
  pub macroblocks: Box<[Macroblock]>,
}

impl<'a> Slice<'a> {
  pub fn new(
    data: &'a [u8],
    nal: &NALUnit,
    sps: &'a SequenceParameterSet,
    pps: &'a PictureParameterSet,
  ) -> Self {
    let pic_width_in_mbs;
    let mut pic_height_in_mbs;
    let pic_size_in_mbs;
    let mut stream = BitStream::new(data);
    let header = SliceHeader::new(&mut stream, nal, sps, pps);
    Self {
      cabac_init_mode: header.cabac_init_idc.map(|idc| idc + 1).unwrap_or(0) as usize,
      chroma_array_type: match nal.unit_type {
        NALUnitType::AuxiliaryCodedPicture => 0,
        _ => {
          if sps.separate_color_plane_flag {
            0
          } else {
            sps.chroma_format_idc
          }
        }
      },
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
      sliceqpy: 26 + pps.pic_init_qp_minus26 + header.slice_qp_delta,
      mbaff_frame_flag: sps.mb_adaptive_frame_field_flag && !header.field_pic_flag,
      last_mb_in_slice: 0,
      header,
      sps,
      pps,
      nal_unit_type: nal.unit_type,
      stream,
      prev_mb_addr: 0,
      curr_mb_addr: 0,
      sgmap: &[],
      macroblocks: (0..pic_size_in_mbs).map(|_| Macroblock::empty(0)).collect(),
    }
  }

  pub fn mb(&self) -> &Macroblock {
    &self.macroblocks[self.curr_mb_addr]
  }

  pub fn mb_mut(&mut self) -> &mut Macroblock {
    &mut self.macroblocks[self.curr_mb_addr]
  }

  pub fn data(&mut self) -> CabacResult {
    self.prev_mb_addr = -1isize as usize;
    self.curr_mb_addr = (self.first_mb_in_slice * (1 + self.mbaff_frame_flag as u16)) as usize;
    self.last_mb_in_slice = self.curr_mb_addr;

    let skip_type = match self.slice_type {
      SliceType::B => MB_TYPE_B_SKIP,
      _ => MB_TYPE_P_SKIP,
    };
    if self.pps.entropy_coding_mode_flag {
      let mut cabac = CabacContext::new(self)?;
      loop {
        let mut mb_skip_flag = 0u8;
        if !self.slice_type.is_intra() {
          let save = self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag;
          let ival = if self.mbaff_frame_flag
            && (self.curr_mb_addr & 1) != 0
            && self.macroblocks[self.curr_mb_addr - 1].mb_type != skip_type
          {
            self.macroblocks[self.curr_mb_addr - 1].mb_field_decoding_flag
          } else {
            self.inferred_mb_field_decoding_flag()
          };
          self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = ival;
          mb_skip_flag = cabac.mb_skip_flag(self)?;
          self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = save;
        }
        if mb_skip_flag != 0 {
          self.infer_skip()?;
        } else {
          if self.mbaff_frame_flag {
            let first_addr = self.curr_mb_addr & !1;
            if self.curr_mb_addr == first_addr {
              self.macroblocks[first_addr].mb_field_decoding_flag =
                cabac.mb_field_decoding_flag(self)? != 0;
            } else {
              if self.macroblocks[first_addr].mb_type == skip_type {
                self.macroblocks[first_addr].mb_field_decoding_flag =
                  cabac.mb_field_decoding_flag(self)? != 0
              }
              self.macroblocks[first_addr + 1].mb_field_decoding_flag =
                self.macroblocks[first_addr].mb_field_decoding_flag;
            }
          } else {
            self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = self.field_pic_flag;
          }
          cabac.macroblock_layer(self)?;
        }
        if !self.mbaff_frame_flag || (self.curr_mb_addr & 1) != 0 {
          let end_of_slice_flag = cabac.terminate(self)?;
          if end_of_slice_flag != 0 {
            self.last_mb_in_slice = self.curr_mb_addr;
            self.stream.skip_trailing_bits();
            return Ok(());
          }
        }
        self.prev_mb_addr = self.curr_mb_addr;
        self.last_mb_in_slice = self.curr_mb_addr;
        self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
        if self.curr_mb_addr >= self.pic_size_in_mbs as usize {
          return Err(CabacError::from(MacroblockError::MacroblockBounds(
            self.curr_mb_addr as isize,
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
            if self.curr_mb_addr >= self.pic_size_in_mbs as usize {
              return Err(CabacError::from(MacroblockError::MacroblockBounds(
                self.curr_mb_addr as isize,
                self.pic_size_in_mbs as usize,
              )));
            }
            self.last_mb_in_slice = self.curr_mb_addr;
            self.macroblocks[self.curr_mb_addr].mb_type = skip_type;
            self.infer_skip()?;
            self.prev_mb_addr = self.curr_mb_addr;
            self.last_mb_in_slice = self.curr_mb_addr;
            self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
          }
          if !self.stream.has_bits() {
            break;
          }
        }
        if self.curr_mb_addr >= self.pic_size_in_mbs as usize {
          return Err(CabacError::from(MacroblockError::MacroblockBounds(
            self.curr_mb_addr as isize,
            self.pic_size_in_mbs as usize,
          )));
        }
        if self.mbaff_frame_flag {
          let first_addr = self.curr_mb_addr & !1;
          if self.curr_mb_addr == first_addr {
            self.macroblocks[first_addr].mb_field_decoding_flag = self.stream.bit_flag();
          } else {
            if self.macroblocks[first_addr].mb_type == skip_type {
              self.macroblocks[first_addr].mb_field_decoding_flag = self.stream.bit_flag();
            }
            self.macroblocks[first_addr + 1].mb_field_decoding_flag =
              self.macroblocks[first_addr].mb_field_decoding_flag;
          }
        } else {
          self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = self.field_pic_flag;
        }
        todo!("implement macroblock layer for cavlc");
        self.last_mb_in_slice = self.curr_mb_addr;
        if !self.stream.has_bits() {
          break;
        }
        self.prev_mb_addr = self.curr_mb_addr;
        self.last_mb_in_slice = self.curr_mb_addr;
        self.curr_mb_addr = self.next_mb_addr(self.curr_mb_addr);
        if self.curr_mb_addr >= self.pic_size_in_mbs as usize {
          return Err(CabacError::from(MacroblockError::MacroblockBounds(
            self.curr_mb_addr as isize,
            self.pic_size_in_mbs as usize,
          )));
        }
      }
    }
    Ok(())
  }

  pub fn next_mb_addr(&self, mut mbaddr: usize) -> usize {
    let sg = self.mb_slice_group(mbaddr);
    mbaddr += 1;
    while mbaddr < self.pic_size_in_mbs as usize && self.mb_slice_group(mbaddr) != sg {
      mbaddr += 1;
    }
    mbaddr
  }

  pub fn inferred_mb_field_decoding_flag(&mut self) -> bool {
    if self.mbaff_frame_flag {
      let mb_a = self.mb_nb_p(MbPosition::A, 0);
      let mb_b = self.mb_nb_p(MbPosition::B, 0);
      if mb_a.mb_type != MB_TYPE_UNAVAILABLE {
        mb_a.mb_field_decoding_flag
      } else if mb_b.mb_type != MB_TYPE_UNAVAILABLE {
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
        if is_skip_mb_type(self.macroblocks[self.curr_mb_addr & !1].mb_type) {
          let val = self.inferred_mb_field_decoding_flag();
          self.macroblocks[self.curr_mb_addr - 1].mb_field_decoding_flag = val;
        }
        self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag =
          self.macroblocks[self.curr_mb_addr - 1].mb_field_decoding_flag;
      }
    } else {
      self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = self.field_pic_flag;
    }
    {
      let mb = &mut self.macroblocks[self.curr_mb_addr];
      mb.mb_type = skip_type;
      mb.mb_qp_delta = 0;
      mb.transform_size_8x8_flag = 0;
      mb.coded_block_pattern = 0;
      mb.intra_chroma_pred_mode = 0;
    }
    self.infer_intra(0);
    self.infer_intra(1);
    let mb = &mut self.macroblocks[self.curr_mb_addr];
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
    let mb = &mut self.macroblocks[self.curr_mb_addr];
    for i in 0..4 {
      mb.ref_idx[which][i] = 0;
    }
    for i in 0..16 {
      mb.mvd[which][i][0] = 0;
      mb.mvd[which][i][1] = 0;
    }
  }

  pub fn inter_filter(&self, inter: u8) -> &Macroblock {
    if inter == 0
      && self.pps.constrained_intra_pred_flag
      && matches!(self.nal_unit_type, NALUnitType::DataPartitionA)
      && is_inter_mb_type(self.mb().mb_type)
    {
      Macroblock::unavailable(1)
    } else {
      self.mb()
    }
  }

  pub fn mb_nb(&self, position: MbPosition, inter: u8) -> CabacResult<&Macroblock> {
    let mbp = self.mb_nb_p(position, inter);
    let mbt = &self.macroblocks[self.curr_mb_addr];
    Ok(match position {
      MbPosition::This => mbp,
      MbPosition::A => {
        if self.mbaff_frame_flag
          && mbp.mb_type != MB_TYPE_UNAVAILABLE
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
            if mbp.mb_type == MB_TYPE_UNAVAILABLE
              || (self.curr_mb_addr & 1 == 0 && mbp.mb_field_decoding_flag)
            {
              mbp
            } else {
              mbp.offset(&self.macroblocks, 1)?
            }
          } else if (self.curr_mb_addr & 1) != 0 {
            mbt.offset(&self.macroblocks, -1)?
          } else if mbp.mb_type != MB_TYPE_UNAVAILABLE {
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
    idx: usize,
    pidx: &mut usize,
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
                Ok(mb_p.offset(&self.macroblocks, idx as isize >> 3)?)
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
                Ok(mb_p.offset(&self.macroblocks, idx as isize >> 2)?)
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
                Ok(mb_p.offset(&self.macroblocks, idx as isize >> 1)?)
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

  pub fn mb_nb_p(&self, position: MbPosition, inter: u8) -> &Macroblock {
    let mut mbaddr = self.curr_mb_addr;
    if self.mbaff_frame_flag {
      mbaddr /= 2;
    }
    let pic_width_in_mbs = self.pic_width_in_mbs as usize;
    match position {
      MbPosition::This => return &self.macroblocks[self.curr_mb_addr],
      MbPosition::A => {
        if mbaddr % pic_width_in_mbs == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr = mbaddr.wrapping_sub(1);
      }
      MbPosition::B => {
        mbaddr = mbaddr.wrapping_sub(pic_width_in_mbs);
      }
      MbPosition::C => {
        if ((mbaddr + 1) % pic_width_in_mbs) == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr = mbaddr.wrapping_sub(pic_width_in_mbs - 1);
      }
      MbPosition::D => {
        if (mbaddr % pic_width_in_mbs) == 0 {
          return Macroblock::unavailable(inter);
        }
        mbaddr = mbaddr.wrapping_sub(pic_width_in_mbs + 1);
      }
    }
    if self.mbaff_frame_flag {
      mbaddr *= 2;
    }
    if !self.mb_available(mbaddr) {
      return Macroblock::unavailable(inter);
    }
    &self.macroblocks[mbaddr]
  }

  pub fn mb_available(&self, mbaddr: usize) -> bool {
    if mbaddr < (self.first_mb_in_slice * (self.mbaff_frame_flag as u16 + 1)) as usize
      || mbaddr > self.curr_mb_addr
    {
      return false;
    }
    self.mb_slice_group(mbaddr) == self.mb_slice_group(self.curr_mb_addr)
  }

  pub fn mb_slice_group(&self, mbaddr: usize) -> u8 {
    if mbaddr >= self.pic_size_in_mbs as usize || self.pps.slice_group.is_none() {
      return 0;
    }
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
      .field("mbaff_frame_flag", &self.mbaff_frame_flag)
      .field("last_mb_in_slice", &self.last_mb_in_slice)
      .field("prev_mb_addr", &self.prev_mb_addr)
      .field("curr_mb_addr", &self.curr_mb_addr)
      .field("sgmap", &self.sgmap)
      .field("macroblocks length", &self.macroblocks.len())
      .finish()
  }
}
