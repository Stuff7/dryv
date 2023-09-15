use super::*;
use crate::ascii::LogDisplay;
use crate::byte::pascal_string;
use crate::log;
use crate::math::fixed_point_to_f32;

#[derive(Debug)]
pub struct StsdAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub sample_description_table: Vec<StsdItem>,
}

impl AtomDecoder for StsdAtom {
  const NAME: [u8; 4] = *b"stsd";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let data = atom.read_data(decoder)?;

    let (version, flags) = decode_version_flags(&data);
    let number_of_entries = u32::from_be_bytes((&data[4..8]).try_into()?);

    atom.offset += 8;
    let mut sample_description_table = Vec::new();
    let mut atoms = atom.atoms(decoder);
    while let Some(atom) = atoms.next() {
      match atom {
        Ok(mut atom) => {
          sample_description_table.push(StsdItem::new(&atom.read_data(atoms.reader)?, atom.name)?)
        }
        Err(e) => log!(err@"#[stsd] {e}"),
      }
    }

    Ok(Self {
      version,
      flags,
      number_of_entries,
      sample_description_table,
    })
  }
}

#[derive(Debug)]
pub struct StsdItem {
  pub data_format: Str<4>,
  pub dref_index: u16,
  pub data: StsdKind,
}

impl StsdItem {
  pub fn new(data: &[u8], data_format: Str<4>) -> AtomResult<Self> {
    // __reserved__ (6 bytes)
    let dref_index = u16::from_be_bytes((&data[6..8]).try_into()?);
    println!("{data_format} {}", data[8..].len());

    Ok(Self {
      data_format,
      dref_index,
      data: StsdKind::new(data_format, &data[8..])?,
    })
  }
}

#[derive(Debug)]
pub enum StsdKind {
  Vide(StsdVide),
  Soun(StsdSoun),
  Unknown(Str<4>),
}

impl StsdKind {
  fn new(hdlr: Str<4>, data: &[u8]) -> AtomResult<Self> {
    Ok(match &*hdlr {
      b"avc1" => Self::Vide(StsdVide::decode(data)?),
      b"mp4a" => Self::Soun(StsdSoun::decode(data)?),
      _ => Self::Unknown(hdlr),
    })
  }
}

#[derive(Debug)]
pub struct StsdVide {
  pub revision_level: u16,
  pub version: u16,
  pub vendor: u32,
  pub temporal_quality: u32,
  pub spatial_quality: u32,
  pub width: u16,
  pub height: u16,
  pub horizontal_resolution: f32,
  pub vertical_resolution: f32,
  pub data_size: u32,
  pub frame_count: u16,
  pub compressor_name: Box<str>,
  pub depth: u16,
  pub color_table_id: u16,
}

impl StsdVide {
  pub fn decode(data: &[u8]) -> AtomResult<Self> {
    let (compressor_name, length) = pascal_string(&data[34..]);
    Ok(Self {
      revision_level: u16::from_be_bytes((&data[..2]).try_into()?),
      version: u16::from_be_bytes((&data[2..4]).try_into()?),
      vendor: u32::from_be_bytes((&data[4..8]).try_into()?),
      temporal_quality: u32::from_be_bytes((&data[8..12]).try_into()?),
      spatial_quality: u32::from_be_bytes((&data[12..16]).try_into()?),
      width: u16::from_be_bytes((&data[16..18]).try_into()?),
      height: u16::from_be_bytes((&data[18..20]).try_into()?),
      horizontal_resolution: fixed_point_to_f32(
        i32::from_be_bytes((&data[20..24]).try_into()?) as f32,
        16,
      ),
      vertical_resolution: fixed_point_to_f32(
        i32::from_be_bytes((&data[24..28]).try_into()?) as f32,
        16,
      ),
      data_size: u32::from_be_bytes((&data[28..32]).try_into()?),
      frame_count: u16::from_be_bytes((&data[32..34]).try_into()?),
      compressor_name,
      depth: u16::from_be_bytes((&data[length..length + 2]).try_into()?),
      color_table_id: u16::from_be_bytes((&data[length + 2..length + 4]).try_into()?),
    })
  }
}

#[derive(Debug)]
pub struct StsdSoun {
  pub version: u16,
  pub revision_level: u16,
  pub vendor: u32,
  pub number_of_channels: u16,
  pub sample_size: u16,
  pub compression_id: u16,
  pub packet_size: u16,
  pub sample_rate: f32,
}

impl StsdSoun {
  pub fn decode(data: &[u8]) -> AtomResult<Self> {
    Ok(Self {
      version: u16::from_be_bytes((&data[..2]).try_into()?),
      revision_level: u16::from_be_bytes((&data[2..4]).try_into()?),
      vendor: u32::from_be_bytes((&data[4..8]).try_into()?),
      number_of_channels: u16::from_be_bytes((&data[8..10]).try_into()?),
      sample_size: u16::from_be_bytes((&data[10..12]).try_into()?),
      compression_id: u16::from_be_bytes((&data[12..14]).try_into()?),
      packet_size: u16::from_be_bytes((&data[14..16]).try_into()?),
      sample_rate: fixed_point_to_f32(i32::from_be_bytes((&data[16..20]).try_into()?) as f32, 16),
    })
  }
}
