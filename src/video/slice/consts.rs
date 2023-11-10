use super::macroblock::Macroblock;

pub const MB_UNAVAILABLE_INTRA: Macroblock = Macroblock::empty_unavailable(1);
pub const MB_UNAVAILABLE_INTER: Macroblock = Macroblock::empty_unavailable(0);

/* I */
/// Macroblock type for an I-frame with no sub-macroblock partitioning (NxN).
pub const MB_TYPE_I_NXN: u8 = 0;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where all sub-macroblocks
/// use intra prediction mode 0.
pub const MB_TYPE_I_16X16_0_0_0: u8 = 1;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 1, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_0_0: u8 = 2;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_0_0: u8 = 3;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 3, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_0_0: u8 = 4;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 0, and sub-macroblock 1 uses mode 1.
pub const MB_TYPE_I_16X16_0_1_0: u8 = 5;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 1, sub-macroblock 1 uses mode 1, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_1_0: u8 = 6;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, sub-macroblock 1 uses mode 1, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_1_0: u8 = 7;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 3, sub-macroblock 1 uses mode 1, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_1_0: u8 = 8;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 0, sub-macroblock 1 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_0_2_0: u8 = 9;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 1, sub-macroblock 1 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_2_0: u8 = 10;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, sub-macroblock 1 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_2_0: u8 = 11;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 3, sub-macroblock 1 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_2_0: u8 = 12;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 0, sub-macroblock 1 uses mode 1, and sub-macroblock 2 uses mode 1.
pub const MB_TYPE_I_16X16_0_0_1: u8 = 13;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 1, sub-macroblock 1 uses mode 1, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_0_1: u8 = 14;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, sub-macroblock 1 uses mode 1, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_0_1: u8 = 15;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 3, sub-macroblock 1 uses mode 1, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_0_1: u8 = 16;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 0, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_0_1_1: u8 = 17;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 1, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_1_1: u8 = 18;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_1_1: u8 = 19;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 3, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 1,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_1_1: u8 = 20;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 0, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 2,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_0_2_1: u8 = 21;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 1 uses
/// intra prediction mode 2, and sub-macroblock 2 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_1_2_1: u8 = 22;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 0 uses
/// intra prediction mode 2, sub-macroblock 1 uses mode 2, and sub-macroblock 2 uses mode 2,
/// and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_2_2_1: u8 = 23;

/// Macroblock type for an I-frame with 16x16 macroblock partitioning, where sub-macroblock 3 uses
/// intra prediction mode 2, and sub-macroblock 2 uses mode 2, and all other sub-macroblocks use mode 0.
pub const MB_TYPE_I_16X16_3_2_1: u8 = 24;

/// Macroblock type for an I-frame using Pulse Code Modulation (PCM) mode.
pub const MB_TYPE_I_PCM: u8 = 25;

/* SI */
/// Represents the macroblock type for a Switching Intra (SI) frame.
/// An SI frame is used to switch between different video streams in a multi-view video sequence.
pub const MB_TYPE_SI: u8 = 26;

/* P */
/// Represents the macroblock type for a P-frame with 16x16 macroblock partitioning in reference list L0.
/// In this mode, motion and prediction information is derived solely from reference list L0.
pub const MB_TYPE_P_L0_16X16: u8 = 27;

/// Represents the macroblock type for a P-frame with 16x8 macroblock partitioning in both reference lists L0 and L0.
/// This mode allows for 16x8 motion and prediction partitioning using reference list L0.
pub const MB_TYPE_P_L0_L0_16X8: u8 = 28;

/// Represents the macroblock type for a P-frame with 8x16 macroblock partitioning in both reference lists L0 and L0.
/// This mode allows for 8x16 motion and prediction partitioning using reference list L0.
pub const MB_TYPE_P_L0_L0_8X16: u8 = 29;

/// Represents the macroblock type for a P-frame with 8x8 macroblock partitioning.
/// In this mode, each macroblock is partitioned into 8x8 blocks for motion and prediction.
pub const MB_TYPE_P_8X8: u8 = 30;

/// Represents the macroblock type for a P-frame with 8x8 macroblock partitioning in reference list L0.
/// In this mode, each macroblock is partitioned into 8x8 blocks for motion and prediction using reference list L0.
pub const MB_TYPE_P_8X8REF0: u8 = 31;

/// Represents the macroblock type for a skipped P-frame.
/// In this mode, there is no motion information or residual data, indicating a skipped macroblock.
pub const MB_TYPE_P_SKIP: u8 = 32;

/* B */
/// Represents the macroblock type for a B-frame with direct 16x16 motion prediction.
/// In this mode, motion information is directly derived from reference frames for 16x16 blocks.
pub const MB_TYPE_B_DIRECT_16X16: u8 = 33;

/// Represents the macroblock type for a B-frame with 16x16 macroblock partitioning in reference list L0.
/// This mode uses reference list L0 for motion and prediction with 16x16 block partitioning.
pub const MB_TYPE_B_L0_16X16: u8 = 34;

/// Represents the macroblock type for a B-frame with 16x16 macroblock partitioning in reference list L1.
/// This mode uses reference list L1 for motion and prediction with 16x16 block partitioning.
pub const MB_TYPE_B_L1_16X16: u8 = 35;

/// Represents the macroblock type for a B-frame with 16x16 macroblock partitioning in both reference lists (bi-directional).
/// In this mode, motion and prediction information is derived from both reference lists for 16x16 blocks.
pub const MB_TYPE_B_BI_16X16: u8 = 36;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L0 and L0.
/// This mode uses reference lists L0 and L0 for motion and prediction with 16x8 block partitioning.
pub const MB_TYPE_B_L0_L0_16X8: u8 = 37;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L1 and L1.
/// This mode uses reference lists L1 and L1 for motion and prediction with 8x16 block partitioning.
pub const MB_TYPE_B_L0_L0_8X16: u8 = 38;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L1 and L1.
/// This mode uses reference lists L1 and L1 for motion and prediction with 16x8 block partitioning.
pub const MB_TYPE_B_L1_L1_16X8: u8 = 39;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L1 and L1.
/// This mode uses reference lists L1 and L1 for motion and prediction with 8x16 block partitioning.
pub const MB_TYPE_B_L1_L1_8X16: u8 = 40;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L0 and L1.
/// This mode uses reference lists L0 and L1 for motion and prediction with 16x8 block partitioning.
pub const MB_TYPE_B_L0_L1_16X8: u8 = 41;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L0 and L1.
/// This mode uses reference lists L0 and L1 for motion and prediction with 8x16 block partitioning.
pub const MB_TYPE_B_L0_L1_8X16: u8 = 42;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L1 and L0.
/// This mode uses reference lists L1 and L0 for motion and prediction with 16x8 block partitioning.
pub const MB_TYPE_B_L1_L0_16X8: u8 = 43;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L1 and L0.
/// This mode uses reference lists L1 and L0 for motion and prediction with 8x16 block partitioning.
pub const MB_TYPE_B_L1_L0_8X16: u8 = 44;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L0 and L0
/// and bi-directional mode.
/// This mode uses reference lists L0 and L0 for motion and prediction with 16x8 block partitioning in bi-directional mode.
pub const MB_TYPE_B_L0_BI_16X8: u8 = 45;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L0 and L0
/// and bi-directional mode.
/// This mode uses reference lists L0 and L0 for motion and prediction with 8x16 block partitioning in bi-directional mode.
pub const MB_TYPE_B_L0_BI_8X16: u8 = 46;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L1 and L1
/// and bi-directional mode.
/// This mode uses reference lists L1 and L1 for motion and prediction with 16x8 block partitioning in bi-directional mode.
pub const MB_TYPE_B_L1_BI_16X8: u8 = 47;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L1 and L1
/// and bi-directional mode.
/// This mode uses reference lists L1 and L1 for motion and prediction with 8x16 block partitioning in bi-directional mode.
pub const MB_TYPE_B_L1_BI_8X16: u8 = 48;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L0 and L1
/// and bi-directional mode.
/// This mode uses reference lists L0 and L1 for motion and prediction with 16x8 block partitioning in bi-directional mode.
pub const MB_TYPE_B_BI_L0_16X8: u8 = 49;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L0 and L1
/// and bi-directional mode.
/// This mode uses reference lists L0 and L1 for motion and prediction with 8x16 block partitioning in bi-directional mode.
pub const MB_TYPE_B_BI_L0_8X16: u8 = 50;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in reference lists L1 and L0
/// and bi-directional mode.
/// This mode uses reference lists L1 and L0 for motion and prediction with 16x8 block partitioning in bi-directional mode.
pub const MB_TYPE_B_BI_L1_16X8: u8 = 51;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in reference lists L1 and L0
/// and bi-directional mode.
/// This mode uses reference lists L1 and L0 for motion and prediction with 8x16 block partitioning in bi-directional mode.
pub const MB_TYPE_B_BI_L1_8X16: u8 = 52;

/// Represents the macroblock type for a B-frame with 16x8 macroblock partitioning in bi-directional mode for both reference lists.
/// This mode uses bi-directional motion and prediction information with 16x8 block partitioning.
pub const MB_TYPE_B_BI_BI_16X8: u8 = 53;

/// Represents the macroblock type for a B-frame with 8x16 macroblock partitioning in bi-directional mode for both reference lists.
/// This mode uses bi-directional motion and prediction information with 8x16 block partitioning.
pub const MB_TYPE_B_BI_BI_8X16: u8 = 54;

/// Represents the macroblock type for an 8x8 block in B-frames.
/// In this mode, each macroblock is partitioned into 8x8 blocks for motion and prediction.
pub const MB_TYPE_B_8X8: u8 = 55;

/// Represents the macroblock type for a skipped B-frame.
/// In this mode, there is no motion information or residual data, indicating a skipped macroblock in B-frames.
pub const MB_TYPE_B_SKIP: u8 = 56;

/// Represents an unavailable macroblock type.
/// This value is used to indicate that a macroblock type is not available or unused.
pub const MB_TYPE_UNAVAILABLE: u8 = 57;

/* P */
/// Represents the sub-macroblock type for P-frames with 8x8 macroblock partitioning in reference list L0.
/// In this mode, each macroblock is partitioned into 8x8 blocks using reference list L0.
pub const SUB_MB_TYPE_P_L0_8X8: u8 = 0;

/// Represents the sub-macroblock type for P-frames with 8x4 macroblock partitioning in reference list L0.
/// This mode allows for 8x4 motion and prediction partitioning using reference list L0.
pub const SUB_MB_TYPE_P_L0_8X4: u8 = 1;

/// Represents the sub-macroblock type for P-frames with 4x8 macroblock partitioning in reference list L0.
/// This mode allows for 4x8 motion and prediction partitioning using reference list L0.
pub const SUB_MB_TYPE_P_L0_4X8: u8 = 2;

/// Represents the sub-macroblock type for P-frames with 4x4 macroblock partitioning in reference list L0.
/// In this mode, each macroblock is partitioned into 4x4 blocks using reference list L0.
pub const SUB_MB_TYPE_P_L0_4X4: u8 = 3;

/// Represents the sub-macroblock type for B-frames with direct 8x8 motion prediction.
/// In this mode, motion information is directly derived from reference frames for 8x8 blocks.
pub const SUB_MB_TYPE_B_DIRECT_8X8: u8 = 4;

/* B */
/// Represents the sub-macroblock type for B-frames with 8x8 macroblock partitioning in reference list L0.
/// This mode uses reference list L0 for motion and prediction with 8x8 block partitioning.
pub const SUB_MB_TYPE_B_L0_8X8: u8 = 5;

/// Represents the sub-macroblock type for B-frames with 8x8 macroblock partitioning in reference list L1.
/// This mode uses reference list L1 for motion and prediction with 8x8 block partitioning.
pub const SUB_MB_TYPE_B_L1_8X8: u8 = 6;

/// Represents the sub-macroblock type for B-frames with 8x8 macroblock partitioning in both reference lists (bi-directional).
/// In this mode, motion and prediction information is derived from both reference lists for 8x8 blocks.
pub const SUB_MB_TYPE_B_BI_8X8: u8 = 7;

/// Represents the sub-macroblock type for B-frames with 8x4 macroblock partitioning in reference list L0.
/// This mode uses reference list L0 for motion and prediction with 8x4 block partitioning.
pub const SUB_MB_TYPE_B_L0_8X4: u8 = 8;

/// Represents the sub-macroblock type for B-frames with 4x8 macroblock partitioning in reference list L0.
/// This mode uses reference list L0 for motion and prediction with 4x8 block partitioning.
pub const SUB_MB_TYPE_B_L0_4X8: u8 = 9;

/// Represents the sub-macroblock type for B-frames with 8x4 macroblock partitioning in reference list L1.
/// This mode uses reference list L1 for motion and prediction with 8x4 block partitioning.
pub const SUB_MB_TYPE_B_L1_8X4: u8 = 10;

/// Represents the sub-macroblock type for B-frames with 4x8 macroblock partitioning in reference list L1.
/// This mode uses reference list L1 for motion and prediction with 4x8 block partitioning.
pub const SUB_MB_TYPE_B_L1_4X8: u8 = 11;

/// Represents the sub-macroblock type for B-frames with 8x4 macroblock partitioning in both reference lists (bi-directional).
/// This mode uses both reference lists for motion and prediction with 8x4 block partitioning.
pub const SUB_MB_TYPE_B_BI_8X4: u8 = 12;

/// Represents the sub-macroblock type for B-frames with 4x8 macroblock partitioning in both reference lists (bi-directional).
/// This mode uses both reference lists for motion and prediction with 4x8 block partitioning.
pub const SUB_MB_TYPE_B_BI_4X8: u8 = 13;

/// Represents the sub-macroblock type for B-frames with 4x4 macroblock partitioning in reference list L0.
/// In this mode, each macroblock is partitioned into 4x4 blocks using reference list L0.
pub const SUB_MB_TYPE_B_L0_4X4: u8 = 14;

/// Represents the sub-macroblock type for B-frames with 4x4 macroblock partitioning in reference list L1.
/// In this mode, each macroblock is partitioned into 4x4 blocks using reference list L1.
pub const SUB_MB_TYPE_B_L1_4X4: u8 = 15;

/// Represents the sub-macroblock type for B-frames with 4x4 macroblock partitioning in both reference lists (bi-directional).
/// This mode uses both reference lists for motion and prediction with 4x4 block partitioning.
pub const SUB_MB_TYPE_B_BI_4X4: u8 = 16;

pub const fn is_intra_16x16_mb_type(mb_type: u8) -> bool {
  mb_type >= MB_TYPE_I_16X16_0_0_0 && mb_type <= MB_TYPE_I_16X16_3_2_1
}

pub const fn name_mb_type(mb_type: u8) -> &'static str {
  match mb_type {
    MB_TYPE_I_NXN => "I_NXN",
    MB_TYPE_I_16X16_0_0_0 => "I_16X16_0_0_0",
    MB_TYPE_I_16X16_1_0_0 => "I_16X16_1_0_0",
    MB_TYPE_I_16X16_2_0_0 => "I_16X16_2_0_0",
    MB_TYPE_I_16X16_3_0_0 => "I_16X16_3_0_0",
    MB_TYPE_I_16X16_0_1_0 => "I_16X16_0_1_0",
    MB_TYPE_I_16X16_1_1_0 => "I_16X16_1_1_0",
    MB_TYPE_I_16X16_2_1_0 => "I_16X16_2_1_0",
    MB_TYPE_I_16X16_3_1_0 => "I_16X16_3_1_0",
    MB_TYPE_I_16X16_0_2_0 => "I_16X16_0_2_0",
    MB_TYPE_I_16X16_1_2_0 => "I_16X16_1_2_0",
    MB_TYPE_I_16X16_2_2_0 => "I_16X16_2_2_0",
    MB_TYPE_I_16X16_3_2_0 => "I_16X16_3_2_0",
    MB_TYPE_I_16X16_0_0_1 => "I_16X16_0_0_1",
    MB_TYPE_I_16X16_1_0_1 => "I_16X16_1_0_1",
    MB_TYPE_I_16X16_2_0_1 => "I_16X16_2_0_1",
    MB_TYPE_I_16X16_3_0_1 => "I_16X16_3_0_1",
    MB_TYPE_I_16X16_0_1_1 => "I_16X16_0_1_1",
    MB_TYPE_I_16X16_1_1_1 => "I_16X16_1_1_1",
    MB_TYPE_I_16X16_2_1_1 => "I_16X16_2_1_1",
    MB_TYPE_I_16X16_3_1_1 => "I_16X16_3_1_1",
    MB_TYPE_I_16X16_0_2_1 => "I_16X16_0_2_1",
    MB_TYPE_I_16X16_1_2_1 => "I_16X16_1_2_1",
    MB_TYPE_I_16X16_2_2_1 => "I_16X16_2_2_1",
    MB_TYPE_I_16X16_3_2_1 => "I_16X16_3_2_1",
    MB_TYPE_I_PCM => "I_PCM",
    MB_TYPE_SI => "SI",
    MB_TYPE_P_L0_16X16 => "P_L0_16X16",
    MB_TYPE_P_L0_L0_16X8 => "P_L0_L0_16X8",
    MB_TYPE_P_L0_L0_8X16 => "P_L0_L0_8X16",
    MB_TYPE_P_8X8 => "P_8X8",
    MB_TYPE_P_8X8REF0 => "P_8X8REF0",
    MB_TYPE_P_SKIP => "P_SKIP",
    MB_TYPE_B_DIRECT_16X16 => "B_DIRECT_16X16",
    MB_TYPE_B_L0_16X16 => "B_L0_16X16",
    MB_TYPE_B_L1_16X16 => "B_L1_16X16",
    MB_TYPE_B_BI_16X16 => "B_BI_16X16",
    MB_TYPE_B_L0_L0_16X8 => "B_L0_L0_16X8",
    MB_TYPE_B_L0_L0_8X16 => "B_L0_L0_8X16",
    MB_TYPE_B_L1_L1_16X8 => "B_L1_L1_16X8",
    MB_TYPE_B_L1_L1_8X16 => "B_L1_L1_8X16",
    MB_TYPE_B_L0_L1_16X8 => "B_L0_L1_16X8",
    MB_TYPE_B_L0_L1_8X16 => "B_L0_L1_8X16",
    MB_TYPE_B_L1_L0_16X8 => "B_L1_L0_16X8",
    MB_TYPE_B_L1_L0_8X16 => "B_L1_L0_8X16",
    MB_TYPE_B_L0_BI_16X8 => "B_L0_BI_16X8",
    MB_TYPE_B_L0_BI_8X16 => "B_L0_BI_8X16",
    MB_TYPE_B_L1_BI_16X8 => "B_L1_BI_16X8",
    MB_TYPE_B_L1_BI_8X16 => "B_L1_BI_8X16",
    MB_TYPE_B_BI_L0_16X8 => "B_BI_L0_16X8",
    MB_TYPE_B_BI_L0_8X16 => "B_BI_L0_8X16",
    MB_TYPE_B_BI_L1_16X8 => "B_BI_L1_16X8",
    MB_TYPE_B_BI_L1_8X16 => "B_BI_L1_8X16",
    MB_TYPE_B_BI_BI_16X8 => "B_BI_BI_16X8",
    MB_TYPE_B_BI_BI_8X16 => "B_BI_BI_8X16",
    MB_TYPE_B_8X8 => "B_8X8",
    MB_TYPE_B_SKIP => "B_SKIP",
    MB_TYPE_UNAVAILABLE => "UNAVAILABLE",
    _ => "INVALID MB TYPE",
  }
}

pub const fn name_sub_mb_type(sub_mb_type: u8) -> &'static str {
  match sub_mb_type {
    SUB_MB_TYPE_P_L0_8X8 => "P_L0_8X8",
    SUB_MB_TYPE_P_L0_8X4 => "P_L0_8X4",
    SUB_MB_TYPE_P_L0_4X8 => "P_L0_4X8",
    SUB_MB_TYPE_P_L0_4X4 => "P_L0_4X4",
    SUB_MB_TYPE_B_DIRECT_8X8 => "B_DIRECT_8X8",
    SUB_MB_TYPE_B_L0_8X8 => "B_L0_8X8",
    SUB_MB_TYPE_B_L1_8X8 => "B_L1_8X8",
    SUB_MB_TYPE_B_BI_8X8 => "B_BI_8X8",
    SUB_MB_TYPE_B_L0_8X4 => "B_L0_8X4",
    SUB_MB_TYPE_B_L0_4X8 => "B_L0_4X8",
    SUB_MB_TYPE_B_L1_8X4 => "B_L1_8X4",
    SUB_MB_TYPE_B_L1_4X8 => "B_L1_4X8",
    SUB_MB_TYPE_B_BI_8X4 => "B_BI_8X4",
    SUB_MB_TYPE_B_BI_4X8 => "B_BI_4X8",
    SUB_MB_TYPE_B_L0_4X4 => "B_L0_4X4",
    SUB_MB_TYPE_B_L1_4X4 => "B_L1_4X4",
    SUB_MB_TYPE_B_BI_4X4 => "B_BI_4X4",
    _ => "INVALID SUB MB TYPE",
  }
}
