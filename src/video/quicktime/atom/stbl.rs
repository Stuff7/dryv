use std::fs::File;
use std::marker::PhantomData;

use super::*;
use crate::ascii::LogDisplay;
use crate::byte::FromSlice;
use crate::log;

#[derive(Debug)]
pub struct StblAtom {
  pub atom: Atom,
  pub stsd: EncodedAtom<StsdAtom>,
  pub stts: SttsAtom,
  pub ctts: Option<CttsAtom>,
  pub stsc: EncodedAtom<StscAtom>,
  pub stss: Option<StssAtom>,
  pub stsz: StszAtom,
  pub stco: StcoAtom,
  pub sgpd: Option<SgpdAtom>,
  pub sbgp: Option<SbgpAtom>,
}

impl AtomDecoder for StblAtom {
  const NAME: [u8; 4] = *b"stbl";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut stsd = EncodedAtom::Required;
    let mut stts = None;
    let mut ctts = None;
    let mut stsc = EncodedAtom::Required;
    let mut stss = None;
    let mut stsz = None;
    let mut stco = None;
    let mut sgpd = None;
    let mut sbgp = None;
    let mut atoms = atom.atoms(decoder);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"stsd" => stsd = EncodedAtom::Encoded(atom),
          b"stts" => stts = Some(SttsAtom::decode_unchecked(atom, atoms.reader)?),
          b"ctts" => ctts = Some(CttsAtom::decode_unchecked(atom, atoms.reader)?),
          b"stsc" => stsc = EncodedAtom::Encoded(atom),
          b"stss" => stss = Some(StssAtom::decode_unchecked(atom, atoms.reader)?),
          b"stsz" => stsz = Some(StszAtom::decode_unchecked(atom, atoms.reader)?),
          b"stco" | b"co64" => stco = Some(StcoAtom::decode_unchecked(atom, atoms.reader)?),
          b"sgpd" => sgpd = Some(SgpdAtom::decode_unchecked(atom, atoms.reader)?),
          b"sbgp" => sbgp = Some(SbgpAtom::decode_unchecked(atom, atoms.reader)?),
          _ => log!(warn@"#[stbl] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[stbl] {e}"),
      }
    }

    Ok(Self {
      atom,
      stsd,
      stts: stts.ok_or(AtomError::Required(SttsAtom::NAME))?,
      ctts,
      stsc,
      stss,
      stsz: stsz.ok_or(AtomError::Required(StszAtom::NAME))?,
      stco: stco.ok_or(AtomError::Required(StcoAtom::NAME))?,
      sgpd,
      sbgp,
    })
  }
}

#[derive(Debug)]
pub struct SttsAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
}

impl SttsAtom {
  pub fn time_to_sample_table(&self, decoder: &mut Decoder) -> AtomResult<SampleTable<SttsItem>> {
    Ok(SampleTable::new(
      decoder.file.try_clone()?,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      8,
    ))
  }
}

impl AtomDecoder for SttsAtom {
  const NAME: [u8; 4] = *b"stts";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<8, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct SttsItem {
  pub sample_count: u32,
  pub sample_duration: u32,
}

impl FromSlice for SttsItem {
  fn from_slice(slice: &[u8]) -> Self {
    let sample_count =
      u32::from_be_bytes((&slice[..4]).try_into().expect("Stts sample_count missing"));
    let sample_duration = u32::from_be_bytes(
      (&slice[4..8])
        .try_into()
        .expect("Stts sample_duration missing"),
    );

    Self {
      sample_count,
      sample_duration,
    }
  }
}

#[derive(Debug)]
pub struct StssAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
}

impl StssAtom {
  pub fn sync_sample_table(&self, decoder: &mut Decoder) -> AtomResult<SampleTable> {
    Ok(SampleTable::new(
      decoder.file.try_clone()?,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      4,
    ))
  }
}

impl AtomDecoder for StssAtom {
  const NAME: [u8; 4] = *b"stss";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<8, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct CttsAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub entry_count: u32,
}

impl AtomDecoder for CttsAtom {
  const NAME: [u8; 4] = *b"ctts";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<8, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      entry_count: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct StscAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
}

impl StscAtom {
  pub fn sample_to_chunk_table(&self, decoder: &mut Decoder) -> AtomResult<SampleTable<StscItem>> {
    Ok(SampleTable::new(
      decoder.file.try_clone()?,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      12,
    ))
  }
}

impl AtomDecoder for StscAtom {
  const NAME: [u8; 4] = *b"stsc";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<8, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct StscItem {
  pub first_chunk: u32,
  pub samples_per_chunk: u32,
  pub sample_description_id: u32,
}

impl FromSlice for StscItem {
  fn from_slice(slice: &[u8]) -> Self {
    let first_chunk =
      u32::from_be_bytes((&slice[..4]).try_into().expect("Stsc first_chunk missing"));
    let samples_per_chunk = u32::from_be_bytes(
      (&slice[4..8])
        .try_into()
        .expect("Stsc samples_per_chunk missing"),
    );
    let sample_description_id = u32::from_be_bytes(
      (&slice[8..12])
        .try_into()
        .expect("Stsc sample_description_id missing"),
    );

    Self {
      first_chunk,
      samples_per_chunk,
      sample_description_id,
    }
  }
}

#[derive(Debug)]
pub struct StszAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub sample_size: u32,
  pub number_of_entries: u32,
}

impl StszAtom {
  pub fn sample_size_table(&self, decoder: &mut Decoder) -> AtomResult<SampleTable> {
    Ok(SampleTable::new(
      decoder.file.try_clone()?,
      self.atom.offset + 12,
      self.atom.offset + self.atom.size as u64,
      4,
    ))
  }
}

impl AtomDecoder for StszAtom {
  const NAME: [u8; 4] = *b"stsz";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<12, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      sample_size: data.next_into()?,
      number_of_entries: data.next_into()?,
    })
  }
}

impl StcoAtom {
  pub fn chunk_offset_table(&self, decoder: &mut Decoder) -> AtomResult<SampleTable> {
    let atom = &self.atom;
    Ok(SampleTable::new(
      decoder.file.try_clone()?,
      atom.offset + 8,
      atom.offset + atom.size as u64,
      if *atom.name == *b"stco" { 4 } else { 8 },
    ))
  }
}

#[derive(Debug, Default)]
pub struct StcoAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
}

impl AtomDecoder for StcoAtom {
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<8, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct SgpdAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub grouping_table: u32,
  pub default_length: u32,
  pub entry_count: u32,
  pub payload_data: Box<[u16]>,
}

impl AtomDecoder for SgpdAtom {
  const NAME: [u8; 4] = *b"sgpd";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      grouping_table: data.next_into()?,
      default_length: data.next_into()?,
      entry_count: data.next_into()?,
      payload_data: data
        .chunks(2)
        .map(|b| {
          b.try_into()
            .map(u16::from_be_bytes)
            .map_err(AtomError::from)
        })
        .collect::<AtomResult<_>>()?,
    })
  }
}

#[derive(Debug)]
pub struct SbgpAtom {
  pub atom: Atom,
  pub version: u8,
  pub flags: [u8; 3],
  pub grouping_type: Str<4>,
  pub entry_count: u32,
  pub sample_count: u32,
  pub group_description_index: u32,
}

impl AtomDecoder for SbgpAtom {
  const NAME: [u8; 4] = *b"sbgp";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data_exact::<20, _>(decoder)?;
    Ok(Self {
      atom,
      version: data.version(),
      flags: data.flags(),
      grouping_type: data.next_into()?,
      entry_count: data.next_into()?,
      sample_count: data.next_into()?,
      group_description_index: data.next_into()?,
    })
  }
}

#[derive(Debug)]
pub struct SampleTable<T: FromSlice = u64> {
  pub reader: File,
  pub buffer: Vec<u8>,
  pub buffer_size: usize,
  pub offset: usize,
  pub start: u64,
  pub end: u64,
  pub chunk_size: usize,
  pub phantom: PhantomData<T>,
}

impl<T: FromSlice> SampleTable<T> {
  const MAX_SIZE: usize = 24_000;
  pub fn new(reader: File, start: u64, end: u64, chunk_size: usize) -> Self {
    Self {
      reader,
      buffer: vec![0; std::cmp::min((end - start) as usize, Self::MAX_SIZE)],
      buffer_size: 0,
      offset: 0,
      start,
      end,
      chunk_size,
      phantom: PhantomData,
    }
  }
}

impl<T: FromSlice> Iterator for SampleTable<T> {
  type Item = T;
  fn next(&mut self) -> Option<Self::Item> {
    (self.start < self.end)
      .then(|| -> AtomResult<T> {
        if self.offset >= self.buffer_size {
          self.buffer_size = std::cmp::min((self.end - self.start) as usize, Self::MAX_SIZE);
          self.reader.seek(SeekFrom::Start(self.start))?;
          self
            .reader
            .read_exact(&mut self.buffer[..self.buffer_size])?;
          self.offset = 0;
        }

        self.start += self.chunk_size as u64;
        let start = self.offset;
        self.offset += self.chunk_size;
        Ok(T::from_slice(&self.buffer[start..self.offset]))
      })
      .and_then(|n| n.ok())
  }

  fn nth(&mut self, n: usize) -> Option<Self::Item> {
    self.start += (n * self.chunk_size) as u64;
    self.next()
  }
}
