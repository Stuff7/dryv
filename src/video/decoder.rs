use super::atom::*;
use super::cabac::CabacError;
use super::sample::*;
use super::slice::dpb::DecodedPictureBuffer;
use super::slice::*;
use crate::byte::{BitStream, Str};
use crate::log;
use crate::video::frame::Frame;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
  #[error("Decoder IO Error\n{0}")]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Atom(#[from] AtomError),
  #[error("Decoder does not support brand {0:?}")]
  Unsupported(Str<4>),
  #[error(transparent)]
  Sample(#[from] SampleError),
  #[error("Missing decoder config")]
  MissingConfig,
  #[error(transparent)]
  Cabac(#[from] CabacError),
}

pub type DecoderResult<T = ()> = Result<T, DecoderError>;

#[derive(Debug)]
pub enum DecoderBrand {
  QuickTime,
  Isom,
  None,
}

impl DecoderBrand {
  pub fn is_isom(&self) -> bool {
    matches!(self, DecoderBrand::Isom)
  }
}

impl TryFrom<Str<4>> for DecoderBrand {
  type Error = DecoderError;
  fn try_from(brand: Str<4>) -> Result<Self, Self::Error> {
    match &*brand {
      b"qt  " => Ok(Self::QuickTime),
      b"isom" => Ok(Self::Isom),
      _ => Err(DecoderError::Unsupported(brand)),
    }
  }
}

#[derive(Debug)]
pub struct Decoder {
  pub file: File,
  pub size: u64,
  pub brand: DecoderBrand,
}

impl Decoder {
  pub fn open<P: AsRef<Path>>(path: P) -> DecoderResult<Self> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    Ok(Self {
      size,
      file,
      brand: DecoderBrand::None,
    })
  }

  pub fn decode_root(&mut self) -> DecoderResult<RootAtom> {
    let root = RootAtom::new(&mut self.file, self.size)?;
    self.brand = DecoderBrand::try_from(root.ftyp.major_brand).or_else(|e| {
      root
        .ftyp
        .compatible_brands
        .iter()
        .find_map(|b| DecoderBrand::try_from(*b).ok())
        .ok_or(e)
    })?;
    Ok(root)
  }

  pub fn decode_sample(&mut self, stbl: &mut StblAtom) -> DecoderResult {
    let samples = SampleIter::new(self, stbl)?.take(1).enumerate();
    let Some(CodecData::Avc1(avc1)) = stbl
      .stsd
      .decode(self)?
      .sample_description_table
      .get_mut(0)
      .map(|d| &mut d.data) else {
        return Err(DecoderError::MissingConfig)
      };
    let nal_length_size = avc1.avcc.nal_length_size_minus_one as usize + 1;
    let mut dpb = DecodedPictureBuffer::new();

    for (i, sample) in samples {
      log!(File@"{:-^100}", format!("SAMPLE #{} ({} bytes)", i + 1, sample.len()));
      for nal in sample.units(nal_length_size) {
        let nal = nal?;
        let msg = format!(
          "[{:?} idc={}] ({} bytes) => ",
          nal.unit_type, nal.idc, nal.size
        );
        match nal.unit_type {
          NALUnitType::Sei => {
            let mut bit_data = BitStream::new(nal.data);
            let sei_msg = SeiMessage::decode(nal.size, &mut bit_data);
            if let SeiPayload::UserDataUnregistered {
              uuid_iso_iec_11578,
              data,
            } = sei_msg.payload
            {
              log!(File@"{msg}SEI: (\"{:016x}\", \"{}\")", uuid_iso_iec_11578, String::from_utf8_lossy(&data));
            } else {
              log!(File@"{msg}{sei_msg:?}");
            }
          }
          NALUnitType::NonIDRPicture | NALUnitType::IDRPicture => {
            let mut slice = Slice::new(nal.data, &nal, &mut avc1.avcc.sps, &mut avc1.avcc.pps);
            let mut frame = Frame::new(&slice);
            slice.data(&mut dpb, &mut frame)?;
            log!(File@"{msg}{:#?}", slice);
            use std::io::Write;
            let name = format!("temp/slice/{i}");
            let mut f = std::fs::File::create(name).expect("SLICE CREATION");
            f.write_all(
              format!(
                "{:#?}\n{:#?}\n{:#?}\n{:#?}",
                dpb,
                nal,
                slice,
                &slice.macroblocks[..10]
              )
              .as_bytes(),
            )
            .expect("SLICE SAVING");
            if i == 0 {
              frame.write_to_yuv_file("temp/yuv_frame")?;
            }
          }
          _ => log!(File@"{msg} [UNUSED]"),
        }
      }
    }
    Ok(())
  }

  pub fn decode_udta_meta<'a>(
    &mut self,
    root: &'a mut RootAtom,
  ) -> DecoderResult<Vec<&'a mut MetaAtom>> {
    log!(File@"{:-^100}", "meta");
    let decoder = self;
    root
      .moov
      .udta
      .as_mut()
      .map(|udta| {
        let udta = udta.decode(decoder)?;
        udta
          .metas
          .iter_mut()
          .map(|meta| {
            log!(File@"MOOV.UDTA.META TAGS {:#?}", meta.tags());
            Ok(meta)
          })
          .collect::<DecoderResult<Vec<_>>>()
      })
      .unwrap_or_else(|| Ok(Vec::new()))
  }

  pub fn decode_stbl<'a>(&mut self, trak: &'a mut TrakAtom) -> DecoderResult<&'a mut StblAtom> {
    let mdia = trak.mdia.decode(self)?;
    log!(File@"ROOT.TRAK.MDIA.HDLR {:#?}", mdia.hdlr.decode(self)?);
    let minf = mdia.minf.decode(self)?;
    let stbl = minf.stbl.decode(self)?;
    log!(File@"STBL Size: {}", stbl.atom.size);
    stbl.stsd.decode(self)?;
    for stsd in &mut *stbl.stsd.decode(self)?.sample_description_table {
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSD {:#?}", stsd);
    }
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STTS {} {:#?}",
      stbl.stts.number_of_entries,
      stbl.stts.time_to_sample_table(self)?.take(4).collect::<Vec<_>>()
    );
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STCO {} {:#?}",
      stbl.stco.number_of_entries,
      stbl.stco.chunk_offset_table(self)?.take(4).collect::<Vec<_>>()
    );
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSZ {} {} {:#?}",
      stbl.stsz.atom.size,
      stbl.stsz.number_of_entries,
      stbl.stsz.sample_size_table(self)?.take(4).collect::<Vec<_>>()
    );
    {
      let stsc = stbl.stsc.decode(self)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSC {} {:#?}",
        stsc.number_of_entries,
        stsc.sample_to_chunk_table(self)?.take(4).collect::<Vec<_>>(),
      );
    }
    if let Some(stss) = &mut stbl.stss {
      let stss = stss;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSS {} {:#?}",
        stss.number_of_entries,
        stss.sync_sample_table(self)?.take(4).collect::<Vec<_>>()
      );
    }
    Ok(stbl)
  }
}

impl Read for Decoder {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.file.read(buf)
  }
}

impl Seek for Decoder {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    self.file.seek(pos)
  }
}
