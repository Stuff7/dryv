pub mod consts;
pub mod header;
pub mod macroblock;

use self::macroblock::MacroblockError;
use consts::*;
use macroblock::MbPosition;

use super::{
  cabac::{CabacContext, CabacError, CabacResult},
  sample::NALUnitType,
};
use crate::{
  byte::BitStream,
  video::atom::{PictureParameterSet, SequenceParameterSet},
  video::sample::NALUnit,
};
use header::*;
use macroblock::Macroblock;
use std::ops::Deref;

#[derive(Debug)]
pub struct Slice<'a> {
  pub header: SliceHeader,
  pub sps: &'a SequenceParameterSet,
  pub pps: &'a PictureParameterSet,
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
  pub macroblocks: &'a mut [Macroblock],
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
      pic_size_in_mbs: pic_width_in_mbs * pic_height_in_mbs,
      sliceqpy: 26 + pps.pic_init_qp_minus26 + header.slice_qp_delta,
      mbaff_frame_flag: sps.mb_adaptive_frame_field_flag && !header.field_pic_flag,
      last_mb_in_slice: 0,
      header,
      sps,
      pps,
      stream,
      prev_mb_addr: 0,
      curr_mb_addr: 0,
      sgmap: &[],
      macroblocks: &mut [],
    }
  }

  pub fn mb(&self) -> &Macroblock {
    &self.macroblocks[self.curr_mb_addr]
  }

  pub fn mb_mut(&mut self) -> &mut Macroblock {
    &mut self.macroblocks[self.curr_mb_addr]
  }

  pub fn data(&'a mut self) -> CabacResult {
    self.prev_mb_addr = -1isize as usize;
    self.curr_mb_addr = (self.first_mb_in_slice * (1 + self.mbaff_frame_flag as u16)) as usize;
    self.last_mb_in_slice = self.curr_mb_addr;

    let skip_type = if matches!(self.slice_type, SliceType::B) {
      MB_TYPE_B_SKIP
    } else {
      MB_TYPE_P_SKIP
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
          cabac.mb_skip_flag(self, &mut mb_skip_flag)?;
          self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = save;
        }
        if mb_skip_flag != 0 {
          self.infer_skip()?;
        } else {
          if self.mbaff_frame_flag {
            let first_addr = self.curr_mb_addr & !1;
            if self.curr_mb_addr == first_addr {
              let mut tmp = 0;
              cabac.mb_field_decoding_flag(self, &mut tmp)?;
              self.macroblocks[first_addr].mb_field_decoding_flag = tmp != 0;
            } else {
              if self.macroblocks[first_addr].mb_type == skip_type {
                let mut tmp = 0;
                cabac.mb_field_decoding_flag(self, &mut tmp)?;
                self.macroblocks[first_addr].mb_field_decoding_flag = tmp != 0
              }
              self.macroblocks[first_addr + 1].mb_field_decoding_flag =
                self.macroblocks[first_addr].mb_field_decoding_flag;
            }
          } else {
            self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = self.field_pic_flag;
          }
          todo!("h264_macroblock_layer(&self.macroblocks[self.curr_mb_addr])?;");
        }
        if !self.mbaff_frame_flag || (self.curr_mb_addr & 1) != 0 {
          let mut end_of_slice_flag = (self.last_mb_in_slice == self.curr_mb_addr) as u8;
          cabac.terminate(self, &mut end_of_slice_flag)?;
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
            if (self.macroblocks[first_addr].mb_type == skip_type) {
              self.macroblocks[first_addr].mb_field_decoding_flag = self.stream.bit_flag();
            }
            self.macroblocks[first_addr + 1].mb_field_decoding_flag =
              self.macroblocks[first_addr].mb_field_decoding_flag;
          }
        } else {
          self.macroblocks[self.curr_mb_addr].mb_field_decoding_flag = self.field_pic_flag;
        }
        todo!("h264_macroblock_layer(&self.macroblocks[self.curr_mb_addr])?;");
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
    return mbaddr;
  }

  pub fn inferred_mb_field_decoding_flag(&mut self) -> bool {
    if (self.mbaff_frame_flag) {
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
          mbp.offset(self.macroblocks, 1)?
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
              mbp.offset(self.macroblocks, 1)?
            }
          } else if (self.curr_mb_addr & 1) != 0 {
            mbt.offset(self.macroblocks, -1)?
          } else if mbp.mb_type != MB_TYPE_UNAVAILABLE {
            mbp.offset(self.macroblocks, 1)?
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
    return &self.macroblocks[mbaddr];
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
