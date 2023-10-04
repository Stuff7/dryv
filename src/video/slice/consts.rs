use super::macroblock::Macroblock;

pub const MB_UNAVAILABLE_INTRA: Macroblock = Macroblock::empty(1);
pub const MB_UNAVAILABLE_INTER: Macroblock = Macroblock::empty(0);

/* I */
pub const MB_TYPE_I_NXN: u8 = 0;
pub const MB_TYPE_I_16X16_0_0_0: u8 = 1;
pub const MB_TYPE_I_16X16_1_0_0: u8 = 2;
pub const MB_TYPE_I_16X16_2_0_0: u8 = 3;
pub const MB_TYPE_I_16X16_3_0_0: u8 = 4;
pub const MB_TYPE_I_16X16_0_1_0: u8 = 5;
pub const MB_TYPE_I_16X16_1_1_0: u8 = 6;
pub const MB_TYPE_I_16X16_2_1_0: u8 = 7;
pub const MB_TYPE_I_16X16_3_1_0: u8 = 8;
pub const MB_TYPE_I_16X16_0_2_0: u8 = 9;
pub const MB_TYPE_I_16X16_1_2_0: u8 = 10;
pub const MB_TYPE_I_16X16_2_2_0: u8 = 11;
pub const MB_TYPE_I_16X16_3_2_0: u8 = 12;
pub const MB_TYPE_I_16X16_0_0_1: u8 = 13;
pub const MB_TYPE_I_16X16_1_0_1: u8 = 14;
pub const MB_TYPE_I_16X16_2_0_1: u8 = 15;
pub const MB_TYPE_I_16X16_3_0_1: u8 = 16;
pub const MB_TYPE_I_16X16_0_1_1: u8 = 17;
pub const MB_TYPE_I_16X16_1_1_1: u8 = 18;
pub const MB_TYPE_I_16X16_2_1_1: u8 = 19;
pub const MB_TYPE_I_16X16_3_1_1: u8 = 20;
pub const MB_TYPE_I_16X16_0_2_1: u8 = 21;
pub const MB_TYPE_I_16X16_1_2_1: u8 = 22;
pub const MB_TYPE_I_16X16_2_2_1: u8 = 23;
pub const MB_TYPE_I_16X16_3_2_1: u8 = 24;
pub const MB_TYPE_I_PCM: u8 = 25;
/* SI */
pub const MB_TYPE_SI: u8 = 26;
/* P */
pub const MB_TYPE_P_L0_16X16: u8 = 27;
pub const MB_TYPE_P_L0_L0_16X8: u8 = 28;
pub const MB_TYPE_P_L0_L0_8X16: u8 = 29;
pub const MB_TYPE_P_8X8: u8 = 30;
pub const MB_TYPE_P_8X8REF0: u8 = 31;
pub const MB_TYPE_P_SKIP: u8 = 32;
/* B */
pub const MB_TYPE_B_DIRECT_16X16: u8 = 33;
pub const MB_TYPE_B_L0_16X16: u8 = 34;
pub const MB_TYPE_B_L1_16X16: u8 = 35;
pub const MB_TYPE_B_BI_16X16: u8 = 36;
pub const MB_TYPE_B_L0_L0_16X8: u8 = 37;
pub const MB_TYPE_B_L0_L0_8X16: u8 = 38;
pub const MB_TYPE_B_L1_L1_16X8: u8 = 39;
pub const MB_TYPE_B_L1_L1_8X16: u8 = 40;
pub const MB_TYPE_B_L0_L1_16X8: u8 = 41;
pub const MB_TYPE_B_L0_L1_8X16: u8 = 42;
pub const MB_TYPE_B_L1_L0_16X8: u8 = 43;
pub const MB_TYPE_B_L1_L0_8X16: u8 = 44;
pub const MB_TYPE_B_L0_BI_16X8: u8 = 45;
pub const MB_TYPE_B_L0_BI_8X16: u8 = 46;
pub const MB_TYPE_B_L1_BI_16X8: u8 = 47;
pub const MB_TYPE_B_L1_BI_8X16: u8 = 48;
pub const MB_TYPE_B_BI_L0_16X8: u8 = 49;
pub const MB_TYPE_B_BI_L0_8X16: u8 = 50;
pub const MB_TYPE_B_BI_L1_16X8: u8 = 51;
pub const MB_TYPE_B_BI_L1_8X16: u8 = 52;
pub const MB_TYPE_B_BI_BI_16X8: u8 = 53;
pub const MB_TYPE_B_BI_BI_8X16: u8 = 54;
pub const MB_TYPE_B_8X8: u8 = 55;
pub const MB_TYPE_B_SKIP: u8 = 56;
pub const MB_TYPE_UNAVAILABLE: u8 = 57;
/* P */
pub const SUB_MB_TYPE_P_L0_8X8: u8 = 0;
pub const SUB_MB_TYPE_P_L0_8X4: u8 = 1;
pub const SUB_MB_TYPE_P_L0_4X8: u8 = 2;
pub const SUB_MB_TYPE_P_L0_4X4: u8 = 3;
/* B */
pub const SUB_MB_TYPE_B_DIRECT_8X8: u8 = 4;
pub const SUB_MB_TYPE_B_L0_8X8: u8 = 5;
pub const SUB_MB_TYPE_B_L1_8X8: u8 = 6;
pub const SUB_MB_TYPE_B_BI_8X8: u8 = 7;
pub const SUB_MB_TYPE_B_L0_8X4: u8 = 8;
pub const SUB_MB_TYPE_B_L0_4X8: u8 = 9;
pub const SUB_MB_TYPE_B_L1_8X4: u8 = 10;
pub const SUB_MB_TYPE_B_L1_4X8: u8 = 11;
pub const SUB_MB_TYPE_B_BI_8X4: u8 = 12;
pub const SUB_MB_TYPE_B_BI_4X8: u8 = 13;
pub const SUB_MB_TYPE_B_L0_4X4: u8 = 14;
pub const SUB_MB_TYPE_B_L1_4X4: u8 = 15;
pub const SUB_MB_TYPE_B_BI_4X4: u8 = 16;
pub const SUB_MB_TYPE_B_END: u8 = 17;

pub const fn is_skip_mb_type(mb_type: u8) -> bool {
  matches!(mb_type, MB_TYPE_P_SKIP | MB_TYPE_B_SKIP)
}

pub const fn is_submb_mb_type(mb_type: u8) -> bool {
  matches!(mb_type, MB_TYPE_P_8X8 | MB_TYPE_P_8X8REF0 | MB_TYPE_B_8X8)
}

pub const fn is_intra_16x16_mb_type(mb_type: u8) -> bool {
  mb_type >= MB_TYPE_I_16X16_0_0_0 && mb_type <= MB_TYPE_I_16X16_3_2_1
}

pub const fn is_inter_mb_type(mb_type: u8) -> bool {
  mb_type >= MB_TYPE_P_L0_16X16
}
