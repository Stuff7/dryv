pub mod atom;
pub mod sample;

use crate::byte::{BitData, Str};
use crate::log;
use atom::*;
use sample::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::num::TryFromIntError;
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

  pub fn decode_moov_meta<'a>(
    &mut self,
    root: &'a mut RootAtom,
  ) -> DecoderResult<Option<&'a mut MetaAtom>> {
    log!(File@"{:-^100}", "meta");
    root
      .moov
      .meta
      .as_mut()
      .map(|meta| {
        log!(File@"MOOV.META TAGS {:#?}", meta.tags());
        log!(File@"MOOV.META {:#?}", meta);
        Ok(meta)
      })
      .transpose()
  }

  pub fn decode_sample(&mut self, stbl: &mut StblAtom) -> DecoderResult {
    for (i, sample) in SampleIter::new(self, stbl)?.take(52).enumerate() {
      println!("SAMPLE #{} ({} bytes)", i + 1, sample.len());
      if let Some(CodecData::Avc1(avc1)) = stbl
        .stsd
        .decode(self)?
        .sample_description_table
        .get(0)
        .map(|d| &d.data)
      {
        let nal_length_size = avc1.avcc.nal_length_size_minus_one as usize + 1;
        for nal in NALUnitIter::new(&sample, nal_length_size) {
          print!("NAL [{:?}] ({} bytes) => ", nal.unit_type, nal.size);
          match nal.unit_type {
            NALUnitType::Sei => {
              let mut bit_data = BitData::new(nal.data);
              let sei_msg = SeiMessage::decode(nal.size, &mut bit_data);
              if let SeiPayload::UserDataUnregistered {
                uuid_iso_iec_11578,
                data,
              } = sei_msg.payload
              {
                log!(File@"SEI.UUID: \"{:016x}\"\nSEI.DATA: \"{}\"", uuid_iso_iec_11578, String::from_utf8_lossy(&data));
              } else {
                println!("{sei_msg:?}");
              }
            }
            NALUnitType::IDRPicture => {
              use std::io::Write;
              let name = format!("temp/idr-{i}.h264");
              let mut img = std::fs::File::create(name).expect("IDR CREATION");
              img.write_all(nal.data).expect("IDR SAVING");
            }
            _ => println!("Unused"),
          }
        }
      }
      println!();
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
      stbl.stts.time_to_sample_table(self)?.take(10).collect::<Vec<_>>()
    );
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STCO {} {:#?}",
      stbl.stco.number_of_entries,
      stbl.stco.chunk_offset_table(self)?.take(10).collect::<Vec<_>>()
    );
    log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSZ {} {} {:#?}",
      stbl.stsz.atom.size,
      stbl.stsz.number_of_entries,
      stbl.stsz.sample_size_table(self)?.take(10).collect::<Vec<_>>()
    );
    {
      let stsc = stbl.stsc.decode(self)?;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSC {} {:#?}",
        stsc.number_of_entries,
        stsc.sample_to_chunk_table(self)?.take(10).collect::<Vec<_>>(),
      );
    }
    if let Some(stss) = &mut stbl.stss {
      let stss = stss;
      log!(File@"ROOT.TRAK.MDIA.MINF.STBL.STSS {} {:#?}",
        stss.number_of_entries,
        stss.sync_sample_table(self)?.take(10).collect::<Vec<_>>()
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
