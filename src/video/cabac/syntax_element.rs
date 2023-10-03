use crate::video::slice::consts::*;

#[derive(Debug, Clone, Copy)]
pub struct SEBit {
  pub bin_idx: u8,
  pub value: u8,
}

impl SEBit {
  pub const fn new<const S: usize>(elems: [(u8, u8); S]) -> [Self; S] {
    let mut se_bits = [Self {
      bin_idx: 0,
      value: 0,
    }; S];
    let mut i = 0;
    while i < elems.len() {
      (se_bits[i].bin_idx, se_bits[i].value) = elems[i];
      i += 1;
    }
    se_bits
  }
}

#[derive(Debug)]
pub struct SEValue {
  pub value: u8,
  pub sub_table: Option<&'static [SEValue]>,
  pub bits: &'static [SEBit],
}

impl SEValue {
  pub const fn new(
    value: u8,
    sub_table: Option<&'static [SEValue]>,
    bits: &'static [SEBit],
  ) -> Self {
    Self {
      value,
      sub_table,
      bits,
    }
  }
}

// Table 9-36
#[rustfmt::skip]
pub const MB_TYPE_I_TABLE: &[SEValue] = &[
  SEValue::new(MB_TYPE_I_NXN, None, &SEBit::new([(0,0)])),
  SEValue::new(MB_TYPE_I_16X16_0_0_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,0), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_0_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,0), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_0_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,0), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_0_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,0), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_0_1_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,0), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_1_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,0), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_1_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,0), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_1_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,0), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_0_2_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,1), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_2_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,1), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_2_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,1), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_2_0, None, &SEBit::new([(0,1), (1,0), (2,0), (3,1), (4,1), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_0_0_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,0), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_0_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,0), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_0_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,0), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_0_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,0), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_0_1_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,0), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_1_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,0), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_1_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,0), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_1_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,0), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_0_2_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,1), (5,0), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_1_2_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,1), (5,0), (6,1)])),
  SEValue::new(MB_TYPE_I_16X16_2_2_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,1), (5,1), (6,0)])),
  SEValue::new(MB_TYPE_I_16X16_3_2_1, None, &SEBit::new([(0,1), (1,0), (2,1), (3,1), (4,1), (5,1), (6,1)])),
  SEValue::new(MB_TYPE_I_PCM, None, &SEBit::new([(0,1), (1,1)])),
];

#[rustfmt::skip]
pub const MB_TYPE_SI_TABLE: &[SEValue] = &[
  SEValue::new(MB_TYPE_SI, None, &SEBit::new([(7,0)])),
  SEValue::new(0, Some(MB_TYPE_I_TABLE), &SEBit::new([(7,1)])),
];

#[rustfmt::skip]
pub const MB_TYPE_P_TABLE: &[SEValue] = &[
  SEValue::new(MB_TYPE_P_L0_16X16, None, &SEBit::new([(7,0), (8,0), (9,0) ])),
  SEValue::new(MB_TYPE_P_8X8, None, &SEBit::new([(7,0), (8,0), (9,1) ])),
  SEValue::new(MB_TYPE_P_L0_L0_8X16, None, &SEBit::new([(7,0), (8,1), (10,0) ])),
  SEValue::new(MB_TYPE_P_L0_L0_16X8, None, &SEBit::new([(7,0), (8,1), (10,1) ])),
  SEValue::new(0, Some(MB_TYPE_I_TABLE), &SEBit::new([(7,1)])),
];

#[rustfmt::skip]
pub const MB_TYPE_B_TABLE: &[SEValue] = &[
  SEValue::new(MB_TYPE_B_DIRECT_16X16, None, &SEBit::new([(7,0)])),
  SEValue::new(MB_TYPE_B_L0_16X16, None, &SEBit::new([(7,1), (8,0), (10,0)])),
  SEValue::new(MB_TYPE_B_L1_16X16, None, &SEBit::new([(7,1), (8,0), (10,1)])),
  SEValue::new(MB_TYPE_B_BI_16X16, None, &SEBit::new([(7,1), (8,1), (9,0), (10,0), (10,0), (10,0)])),
  SEValue::new(MB_TYPE_B_L0_L0_16X8, None, &SEBit::new([(7,1), (8,1), (9,0), (10,0), (10,0), (10,1)])),
  SEValue::new(MB_TYPE_B_L0_L0_8X16, None, &SEBit::new([(7,1), (8,1), (9,0), (10,0), (10,1), (10,0)])),
  SEValue::new(MB_TYPE_B_L1_L1_16X8, None, &SEBit::new([(7,1), (8,1), (9,0), (10,0), (10,1), (10,1)])),
  SEValue::new(MB_TYPE_B_L1_L1_8X16, None, &SEBit::new([(7,1), (8,1), (9,0), (10,1), (10,0), (10,0)])),
  SEValue::new(MB_TYPE_B_L0_L1_16X8, None, &SEBit::new([(7,1), (8,1), (9,0), (10,1), (10,0), (10,1)])),
  SEValue::new(MB_TYPE_B_L0_L1_8X16, None, &SEBit::new([(7,1), (8,1), (9,0), (10,1), (10,1), (10,0)])),
  SEValue::new(MB_TYPE_B_L1_L0_16X8, None, &SEBit::new([(7,1), (8,1), (9,0), (10,1), (10,1), (10,1)])),
  SEValue::new(MB_TYPE_B_L0_BI_16X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,0), (10,0), (10,0)])),
  SEValue::new(MB_TYPE_B_L0_BI_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,0), (10,0), (10,1)])),
  SEValue::new(MB_TYPE_B_L1_BI_16X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,0), (10,1), (10,0)])),
  SEValue::new(MB_TYPE_B_L1_BI_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,0), (10,1), (10,1)])),
  SEValue::new(MB_TYPE_B_BI_L0_16X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,1), (10,0), (10,0)])),
  SEValue::new(MB_TYPE_B_BI_L0_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,1), (10,0), (10,1)])),
  SEValue::new(MB_TYPE_B_BI_L1_16X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,1), (10,1), (10,0)])),
  SEValue::new(MB_TYPE_B_BI_L1_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,0), (10,1), (10,1), (10,1)])),
  SEValue::new(MB_TYPE_B_BI_BI_16X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,1), (10,0), (10,0), (10,0)])),
  SEValue::new(MB_TYPE_B_BI_BI_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,1), (10,0), (10,0), (10,1)])),
  SEValue::new(0, Some(MB_TYPE_I_TABLE), &SEBit::new([(7,1), (8,1), (9,1), (10,1), (10,0), (10,1)])),
  SEValue::new(MB_TYPE_B_L1_L0_8X16, None, &SEBit::new([(7,1), (8,1), (9,1), (10,1), (10,1), (10,0)])),
  SEValue::new(MB_TYPE_B_8X8, None, &SEBit::new([(7,1), (8,1), (9,1), (10,1), (10,1), (10,1)])),
];

#[rustfmt::skip]
pub const SUB_MB_TYPE_P_TABLE: &[SEValue] = &[
  SEValue::new(SUB_MB_TYPE_P_L0_8X4, None, &SEBit::new([(0,0), (1,0)])),
  SEValue::new(SUB_MB_TYPE_P_L0_4X4, None, &SEBit::new([(0,0), (1,1), (2,0)])),
  SEValue::new(SUB_MB_TYPE_P_L0_4X8, None, &SEBit::new([(0,0), (1,1), (2,1)])),
  SEValue::new(SUB_MB_TYPE_P_L0_8X8, None, &SEBit::new([(0,1)])),
];

#[rustfmt::skip]
pub const SUB_MB_TYPE_B_TABLE: &[SEValue] = &[
  SEValue::new(SUB_MB_TYPE_B_DIRECT_8X8, None, &SEBit::new([(0,0)])),
  SEValue::new(SUB_MB_TYPE_B_L0_8X8, None, &SEBit::new([(0,1), (1,0), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_L1_8X8, None, &SEBit::new([(0,1), (1,0), (3,1)])),
  SEValue::new(SUB_MB_TYPE_B_BI_8X8, None, &SEBit::new([(0,1), (1,1), (2,0), (3,0), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_L0_8X4, None, &SEBit::new([(0,1), (1,1), (2,0), (3,0), (3,1)])),
  SEValue::new(SUB_MB_TYPE_B_L0_4X8, None, &SEBit::new([(0,1), (1,1), (2,0), (3,1), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_L1_8X4, None, &SEBit::new([(0,1), (1,1), (2,0), (3,1), (3,1)])),
  SEValue::new(SUB_MB_TYPE_B_L1_4X8, None, &SEBit::new([(0,1), (1,1), (2,1), (3,0), (3,0), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_BI_8X4, None, &SEBit::new([(0,1), (1,1), (2,1), (3,0), (3,0), (3,1)])),
  SEValue::new(SUB_MB_TYPE_B_BI_4X8, None, &SEBit::new([(0,1), (1,1), (2,1), (3,0), (3,1), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_L0_4X4, None, &SEBit::new([(0,1), (1,1), (2,1), (3,0), (3,1), (3,1)])),
  SEValue::new(SUB_MB_TYPE_B_L1_4X4, None, &SEBit::new([(0,1), (1,1), (2,1), (3,1), (3,0)])),
  SEValue::new(SUB_MB_TYPE_B_BI_4X4, None, &SEBit::new([(0,1), (1,1), (2,1), (3,1), (3,1)])),
];
