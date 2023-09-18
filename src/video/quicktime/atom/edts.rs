use super::*;
use crate::ascii::LogDisplay;
use crate::log;

#[derive(Debug, Default)]
pub struct EdtsAtom {
  pub elst: EncodedAtom<ElstAtom>,
}

impl AtomDecoder for EdtsAtom {
  const NAME: [u8; 4] = *b"edts";
  fn decode_unchecked(atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut edts = Self::default();
    for atom in atom.atoms(decoder) {
      match atom {
        Ok(atom) => match &*atom.name {
          b"elst" => edts.elst = EncodedAtom::Encoded(atom),
          _ => log!(warn@"#[edts] Unused atom {atom:#?}"),
        },
        Err(e) => log!(err@"#[edts] {e}"),
      }
    }

    Ok(edts)
  }
}

#[derive(Debug, Default)]
pub struct ElstAtom {
  pub version: u8,
  pub flags: [u8; 3],
  pub number_of_entries: u32,
  pub edit_list_table: Box<[ElstItem]>,
}

impl AtomDecoder for ElstAtom {
  const NAME: [u8; 4] = *b"elst";
  fn decode_unchecked(mut atom: Atom, decoder: &mut Decoder) -> AtomResult<Self> {
    let mut data = atom.read_data(decoder)?;
    Ok(Self {
      version: data.version(),
      flags: data.flags(),
      number_of_entries: data.next_into()?,
      edit_list_table: data
        .chunks(12)
        .map(|data| ElstItem::from_bytes(AtomData::new(data, atom.offset)))
        .collect::<AtomResult<_>>()?,
    })
  }
}

#[derive(Debug, Default)]
pub struct ElstItem {
  pub track_duration: u32,
  pub media_time: i32,
  pub media_rate: f32,
}

impl ElstItem {
  pub fn from_bytes(mut data: AtomData) -> AtomResult<Self> {
    Ok(Self {
      track_duration: data.next_into()?,
      media_time: data.next_into()?,
      media_rate: data.fixed_point_16()?,
    })
  }
}
