use std::marker::PhantomData;

use super::*;
use crate::ascii::LogDisplay;
use crate::byte::FromSlice;
use crate::log;

#[derive(Debug, Default)]
pub struct StblAtom {
  pub stsd: EncodedAtom<StsdAtom>,
  pub stts: EncodedAtom<SttsAtom>,
  pub stss: Option<EncodedAtom<StssAtom>>,
  pub ctts: Option<EncodedAtom<CttsAtom>>,
  pub stsc: EncodedAtom<StscAtom>,
  pub stsz: EncodedAtom<StszAtom>,
  pub stco: StcoAtom,
  pub sgpd: Option<EncodedAtom<SgpdAtom>>,
  pub sbgp: Option<EncodedAtom<SbgpAtom>>,
}

impl AtomDecoder for StblAtom {
  const NAME: [u8; 4] = *b"stbl";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut stbl = Self::default();
    let mut atoms = atom.atoms(decoder);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(atom) => match &*atom.name {
          b"stsd" => stbl.stsd = EncodedAtom::Encoded(atom),
          b"stts" => stbl.stts = EncodedAtom::Encoded(atom),
          b"stss" => stbl.stss = Some(EncodedAtom::Encoded(atom)),
          b"ctts" => stbl.ctts = Some(EncodedAtom::Encoded(atom)),
          b"stsc" => stbl.stsc = EncodedAtom::Encoded(atom),
          b"stsz" => stbl.stsz = EncodedAtom::Encoded(atom),
          b"stco" | b"co64" => stbl.stco = StcoAtom::decode_unchecked(atom, atoms.reader)?,
          b"sgpd" => stbl.sgpd = Some(EncodedAtom::Encoded(atom)),
          b"sbgp" => stbl.sbgp = Some(EncodedAtom::Encoded(atom)),
          _ => log!(warn@"#[stbl] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[stbl] {e}"),
      }
    }

    Ok(stbl)
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
  pub fn time_to_sample_table<'a>(&self, decoder: &'a mut Decoder) -> SampleTable<'a, SttsItem> {
    SampleTable::new(
      decoder,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      8,
    )
  }
}

impl AtomDecoder for SttsAtom {
  const NAME: [u8; 4] = *b"stts";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
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
  pub fn sync_sample_table<'a>(&self, decoder: &'a mut Decoder) -> SampleTable<'a> {
    SampleTable::new(
      decoder,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      4,
    )
  }
}

impl AtomDecoder for StssAtom {
  const NAME: [u8; 4] = *b"stss";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data: [u8; 8] = atom.read_data_exact(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
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
    let data: [u8; 8] = atom.read_data_exact(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let entry_count = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      entry_count,
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
  pub fn sample_to_chunk_table<'a>(&self, decoder: &'a mut Decoder) -> SampleTable<'a, StscItem> {
    SampleTable::new(
      decoder,
      self.atom.offset + 8,
      self.atom.offset + self.atom.size as u64,
      12,
    )
  }
}

impl AtomDecoder for StscAtom {
  const NAME: [u8; 4] = *b"stsc";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data: [u8; 8] = atom.read_data_exact(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries,
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
  pub fn sample_size_table<'a>(&self, decoder: &'a mut Decoder) -> SampleTable<'a> {
    SampleTable::new(
      decoder,
      self.atom.offset + 12,
      self.atom.offset + self.atom.size as u64,
      4,
    )
  }
}

impl AtomDecoder for StszAtom {
  const NAME: [u8; 4] = *b"stsz";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data: [u8; 12] = atom.read_data_exact(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let sample_size = u32::from_be_bytes((&data[4..8]).try_into()?);
    let number_of_entries = u32::from_be_bytes((&data[8..12]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      sample_size,
      number_of_entries,
    })
  }
}

impl StcoAtom {
  pub fn chunk_offset_table<'a>(&self, decoder: &'a mut Decoder) -> SampleTable<'a> {
    let atom = &self.atom;
    SampleTable::new(
      decoder,
      atom.offset + 8,
      atom.offset + atom.size as u64,
      if *atom.name == *b"stco" { 4 } else { 8 },
    )
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
    let data: [u8; 8] = atom.read_data_exact(decoder)?;
    let (version, flags) = decode_version_flags(&data);

    Ok(Self {
      atom,
      version,
      flags,
      number_of_entries: u32::from_be_bytes((&data[4..8]).try_into()?),
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
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let grouping_table = u32::from_be_bytes((&data[4..8]).try_into()?);
    let default_length = u32::from_be_bytes((&data[8..12]).try_into()?);
    let entry_count = u32::from_be_bytes((&data[12..16]).try_into()?);

    let payload_data = data[16..]
      .chunks(2)
      .map(|b| {
        b.try_into()
          .map(u16::from_be_bytes)
          .map_err(AtomError::from)
      })
      .collect::<AtomResult<_>>()?;

    Ok(Self {
      atom,
      version,
      flags,
      grouping_table,
      default_length,
      entry_count,
      payload_data,
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
    let data: [u8; 20] = atom.read_data_exact(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let grouping_type = Str::try_from(&data[4..8])?;
    let entry_count = u32::from_be_bytes((&data[8..12]).try_into()?);
    let sample_count = u32::from_be_bytes((&data[12..16]).try_into()?);

    let group_description_index = u32::from_be_bytes((&data[16..20]).try_into()?);

    Ok(Self {
      atom,
      version,
      flags,
      grouping_type,
      entry_count,
      sample_count,
      group_description_index,
    })
  }
}

#[derive(Debug)]
pub struct SampleTable<'a, T: FromSlice = u64> {
  pub reader: &'a mut Decoder,
  pub buffer: Vec<u8>,
  pub buffer_size: usize,
  pub offset: usize,
  pub start: u64,
  pub end: u64,
  pub chunk_size: usize,
  pub phantom: PhantomData<T>,
}

impl<'a, T: FromSlice> SampleTable<'a, T> {
  const MAX_SIZE: usize = 24 * 1_000;
  pub fn new(reader: &'a mut Decoder, start: u64, end: u64, chunk_size: usize) -> Self {
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

impl<'a, T: FromSlice> Iterator for SampleTable<'a, T> {
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
    self.start += n as u64;
    self.next()
  }
}
