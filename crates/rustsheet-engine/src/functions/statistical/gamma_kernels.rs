//! The coefficient doubles of Excel's open GAMMA / GAMMALN kernels (T-1941, T-2128): Excel's own
//! rational structures with the best coefficient sets the research found, kept together so T-2122
//! can swap in the true bits. None is bit-exact yet; each array says its measured rate. Evidence
//! and checkers: `docs/research/t1941/` on branch `agent/research/T-1941-crack-gammaln-gamma-ln`
//! (HANDOFF2.md §3, §12). Each value is the hex double the research file gives, transcribed by bit
//! pattern.

/// GAMMALN's (2,3] kernel K2, numerator: `fl(fl(x - 2) * fl(N/D))` with N and D raw-x Horner, the
/// leading coefficient first. Excel's structure with the best known coefficients, not bit-exact;
/// measured exact rate 0.950 on [2.02, 3] (1,862,260 distinct x) and 0.991 through the (3, 8)
/// reduction; see T-2122. `k2_ref.py` (W2 and reviewer 3): N's first three stages and
/// D's first two stay in x87 registers, every op from the hand-off product on is stored.
/// FITTED(T-1941, 524288 cells, GAMMALN x 2..3, 1024 dense clusters)
#[rustfmt::skip]
pub(super) const K2_N: [f64; 10] = [
    f64::from_bits(0x3CC5_7179_94EF_45D1), // 0x1.5717994ef45d1p-51
    f64::from_bits(0x3EF5_E5B7_3827_6E47), // 0x1.5e5b738276e47p-16
    f64::from_bits(0x3F60_EE2F_1189_1885), // 0x1.0ee2f11891885p-9
    f64::from_bits(0x3FA5_72BF_8684_1A77), // 0x1.572bf86841a77p-5
    f64::from_bits(0x3FD0_9FD1_AD1D_8FEC), // 0x1.09fd1ad1d8fecp-2
    f64::from_bits(0x3FDF_0A8C_0539_86D7), // 0x1.f0a8c053986d7p-2
    f64::from_bits(0xBF90_8305_B479_98B9), // -0x1.08305b47998b9p-6
    f64::from_bits(0xBFE0_AE34_EB3C_BAD6), // -0x1.0ae34eb3cbad6p-1
    f64::from_bits(0xBFCD_E852_AEE8_C153), // -0x1.de852aee8c153p-3
    f64::from_bits(0xBF92_2260_ED97_843D), // -0x1.22260ed97843dp-6
];

/// K2's denominator (see [`K2_N`]), the leading coefficient first.
/// FITTED(T-1941, 524288 cells, GAMMALN x 2..3, 1024 dense clusters)
#[rustfmt::skip]
pub(super) const K2_D: [f64; 9] = [
    f64::from_bits(0x3ED0_3CAD_F2CC_53B6), // 0x1.03cadf2cc53b6p-18
    f64::from_bits(0x3F45_2FC9_914B_DD79), // 0x1.52fc9914bdd79p-11
    f64::from_bits(0x3F95_3036_58EF_4484), // 0x1.5303658ef4484p-6
    f64::from_bits(0x3FCB_B43B_464D_3964), // 0x1.bb43b464d3964p-3
    f64::from_bits(0x3FEC_5151_77EB_1641), // 0x1.c515177eb1641p-1
    f64::from_bits(0x3FF7_B39F_C919_4438), // 0x1.7b39fc9194438p+0
    f64::from_bits(0x3FEE_DDF2_040D_2BC2), // 0x1.eddf2040d2bc2p-1
    f64::from_bits(0x3FC9_B252_B2FA_B347), // 0x1.9b252b2fab347p-3
    f64::from_bits(0x3F80_0000_0000_00EA), // 0x1.00000000000eap-7
];

/// Reviewer 3's "cand9" K2 numerator (HANDOFF2 §12.1), evaluated all-stored, used for
/// x - 2 below 0.01 (`gammaln.rs` `K2_CAND9_BELOW`, the measured optimum, statistics.md). Excel's structure with the best known coefficients near 2,
/// not bit-exact; measured exact rate 0.983 on (2, 2.02) (192,512 distinct x, with the cut: cand9 below 2.01, `K2_N` above); see T-2122. Only p1..p3 differ from [`K2_N`] (HANDOFF2's "p1..p5" is relative to `grid_a`).
/// FITTED(T-1941, 524288 cells, GAMMALN x 2..3, clusters and near-2 ladders)
#[rustfmt::skip]
pub(super) const K2_CAND9_N: [f64; 10] = [
    f64::from_bits(0x3CC5_7179_94EF_45D1), // 0x1.5717994ef45d1p-51
    f64::from_bits(0x3EF5_E5B7_3827_6791), // 0x1.5e5b738276791p-16
    f64::from_bits(0x3F60_EE2F_1189_18CD), // 0x1.0ee2f118918cdp-9
    f64::from_bits(0x3FA5_72BF_8684_1A71), // 0x1.572bf86841a71p-5
    f64::from_bits(0x3FD0_9FD1_AD1D_8FEC), // 0x1.09fd1ad1d8fecp-2
    f64::from_bits(0x3FDF_0A8C_0539_86D7), // 0x1.f0a8c053986d7p-2
    f64::from_bits(0xBF90_8305_B479_98B9), // -0x1.08305b47998b9p-6
    f64::from_bits(0xBFE0_AE34_EB3C_BAD6), // -0x1.0ae34eb3cbad6p-1
    f64::from_bits(0xBFCD_E852_AEE8_C153), // -0x1.de852aee8c153p-3
    f64::from_bits(0xBF92_2260_ED97_843D), // -0x1.22260ed97843dp-6
];

/// The cand9 denominator (see [`K2_CAND9_N`]); only q0..q2 differ from [`K2_D`].
/// FITTED(T-1941, 524288 cells, GAMMALN x 2..3, clusters and near-2 ladders)
#[rustfmt::skip]
pub(super) const K2_CAND9_D: [f64; 9] = [
    f64::from_bits(0x3ED0_3CAD_F2CC_5614), // 0x1.03cadf2cc5614p-18
    f64::from_bits(0x3F45_2FC9_914B_DD53), // 0x1.52fc9914bdd53p-11
    f64::from_bits(0x3F95_3036_58EF_4486), // 0x1.5303658ef4486p-6
    f64::from_bits(0x3FCB_B43B_464D_3964), // 0x1.bb43b464d3964p-3
    f64::from_bits(0x3FEC_5151_77EB_1641), // 0x1.c515177eb1641p-1
    f64::from_bits(0x3FF7_B39F_C919_4438), // 0x1.7b39fc9194438p+0
    f64::from_bits(0x3FEE_DDF2_040D_2BC2), // 0x1.eddf2040d2bc2p-1
    f64::from_bits(0x3FC9_B252_B2FA_B347), // 0x1.9b252b2fab347p-3
    f64::from_bits(0x3F80_0000_0000_00EA), // 0x1.00000000000eap-7
];

/// GAMMALN's [0.7, 1.5) kernel K3, numerator: `fl(fl(x - 1) * fl(N/D))`, degree (9,9), N and D
/// raw-x Horner with every op stored, the constant term first (`w5/p4/ss_fc.txt`). Excel's structure
/// with the best known coefficients, not bit-exact; measured exact rate 0.322 on [0.7, 1.5] (8,082,879 distinct x) and 0.869 through the
/// x < 0.7 recursion; see T-2122.
/// FITTED(T-1941, 8082879 cells, GAMMALN x 0.7..1.5)
#[rustfmt::skip]
pub(super) const K3_N: [f64; 10] = [
    f64::from_bits(0xBF45_9F97_CB25_92FF), // -0x1.59f97cb2592ffp-11
    f64::from_bits(0xBF99_2587_E5CC_6B9C), // -0x1.92587e5cc6b9cp-6
    f64::from_bits(0xBFC8_558F_6AE3_9A6B), // -0x1.8558f6ae39a6bp-3
    f64::from_bits(0xBFDD_3F13_7130_667F), // -0x1.d3f137130667fp-2
    f64::from_bits(0xBFD4_C00F_8658_4772), // -0x1.4c00f86584772p-2
    f64::from_bits(0x3FA5_9E7C_A693_1D79), // 0x1.59e7ca6931d79p-5
    f64::from_bits(0x3FB6_D0D4_91D3_170A), // 0x1.6d0d491d3170ap-4
    f64::from_bits(0x3F93_038D_8982_4B68), // 0x1.3038d89824b68p-6
    f64::from_bits(0x3F4C_3AA7_BF92_DB61), // 0x1.c3aa7bf92db61p-11
    f64::from_bits(0x3EC3_37AA_15BF_7E5B), // 0x1.337aa15bf7e5bp-19
];

/// K3's denominator (see [`K3_N`]), the constant term first; D0 = 2^-13.
/// FITTED(T-1941, 8082879 cells, GAMMALN x 0.7..1.5)
#[rustfmt::skip]
pub(super) const K3_D: [f64; 10] = [
    f64::from_bits(0x3F20_0000_0000_0000), // 0x1.0000000000000p-13
    f64::from_bits(0x3F7F_168C_3E77_CF05), // 0x1.f168c3e77cf05p-8
    f64::from_bits(0x3FB7_2EB2_73A4_EE6D), // 0x1.72eb273a4ee6dp-4
    f64::from_bits(0x3FD6_B0B4_A084_3B68), // 0x1.6b0b4a0843b68p-2
    f64::from_bits(0x3FE1_8E83_90E5_37A6), // 0x1.18e8390e537a6p-1
    f64::from_bits(0x3FD6_E6B1_D23C_C97E), // 0x1.6e6b1d23cc97ep-2
    f64::from_bits(0x3FB8_A4A6_51D2_C4F4), // 0x1.8a4a651d2c4f4p-4
    f64::from_bits(0x3F83_5398_ED10_925E), // 0x1.35398ed10925ep-7
    f64::from_bits(0x3F2E_BFE9_8D77_F93F), // 0x1.ebfe98d77f93fp-13
    f64::from_bits(0xBE04_60D4_2BDB_1BE4), // -0x1.460d42bdb1be4p-31
];

/// GAMMALN's (1.5, 2] kernel, numerator of R in `fl(fl(x - 1) * fl(fl(x - 2) * fl(N/D)))`, degree
/// (9,9), raw-x Horner with every op stored, the constant term first (W6's
/// `w6/data/xl_chain_d3ls.txt`; D0 is not a power of two and is not normalised away). Excel's
/// structure with the best known coefficients, not bit-exact; measured exact rate 0.428 on (1.5, 2] (7,548,389 distinct x); see T-2122.
/// FITTED(T-1941, 7548389 cells, GAMMALN x 1.5..2)
#[rustfmt::skip]
pub(super) const R15_N: [f64; 10] = [
    f64::from_bits(0x3F66_733C_BA7A_7048), // 0x1.6733cba7a7048p-9
    f64::from_bits(0x3F7A_B5D3_A0A3_7D0C), // 0x1.ab5d3a0a37d0cp-8
    f64::from_bits(0x4023_8EFC_A744_DFDC), // 0x1.38efca744dfdcp+3
    f64::from_bits(0x4057_7C13_87C0_B531), // 0x1.77c1387c0b531p+6
    f64::from_bits(0x406B_CDC3_4D5E_76AC), // 0x1.bcdc34d5e76acp+7
    f64::from_bits(0x4065_9F05_33CB_DEEF), // 0x1.59f0533cbdeefp+7
    f64::from_bits(0x4046_C8E7_686B_F00A), // 0x1.6c8e7686bf00ap+5
    f64::from_bits(0x400A_F422_11D4_7FA3), // 0x1.af42211d47fa3p+1
    f64::from_bits(0x3F96_A4E4_59CD_F6C2), // 0x1.6a4e459cdf6c2p-6
    f64::from_bits(0xBF28_12A8_3119_6BEF), // -0x1.812a831196befp-13
];

/// The (1.5, 2] denominator (see [`R15_N`]), the constant term first.
/// FITTED(T-1941, 7548389 cells, GAMMALN x 1.5..2)
#[rustfmt::skip]
pub(super) const R15_D: [f64; 10] = [
    f64::from_bits(0x3F5A_E74A_1AEB_96D0), // 0x1.ae74a1aeb96d0p-10
    f64::from_bits(0xBF95_8D8D_CF81_1B03), // -0x1.58d8dcf811b03p-6
    f64::from_bits(0x4013_CA5E_BDFC_80D6), // 0x1.3ca5ebdfc80d6p+2
    f64::from_bits(0x4054_BE7C_906F_F0A5), // 0x1.4be7c906ff0a5p+6
    f64::from_bits(0x4073_0AAE_5374_6110), // 0x1.30aae53746110p+8
    f64::from_bits(0x4076_EF43_3DC4_82A1), // 0x1.6ef433dc482a1p+8
    f64::from_bits(0x4064_687E_0ABE_D9F4), // 0x1.4687e0abed9f4p+7
    f64::from_bits(0x4039_AAEC_FC22_B8CD), // 0x1.9aaecfc22b8cdp+4
    f64::from_bits(0x3FF1_3D5D_4770_BBAA), // 0x1.13d5d4770bbaap+0
    f64::from_bits(0xBD44_2F49_D800_0000), // -0x1.42f49d8000000p-43
];

/// GAMMA's G on [1, 2), K1, numerator: G = `fl(N/D)`, degree (7,7), raw-y Horner with every op stored,
/// the constant term first (W4's `w4/k1r/plain1.txt`). Excel's structure with the best known
/// coefficients, not bit-exact; measured exact rate 0.784 on GAMMA [1, 10) (4,229,812 distinct x), 0.838
/// on (0, 1) and 0.876 on [-10, 0), all through G; see T-2122.
/// FITTED(T-1941, 4229812 cells, GAMMA x 1..10)
#[rustfmt::skip]
pub(super) const K1_N: [f64; 8] = [
    f64::from_bits(0x40EB_5834_ABBE_A656), // 0x1.b5834abbea656p+15
    f64::from_bits(0x40D5_1477_3450_CAEF), // 0x1.514773450caefp+14
    f64::from_bits(0x40C6_6479_5A0F_0ED9), // 0x1.664795a0f0ed9p+13
    f64::from_bits(0x40A3_6C63_6BFA_4529), // 0x1.36c636bfa4529p+11
    f64::from_bits(0x4085_FF5A_1F11_40C6), // 0x1.5ff5a1f1140c6p+9
    f64::from_bits(0x405B_983B_931D_B2B8), // 0x1.b983b931db2b8p+6
    f64::from_bits(0x402C_11D5_CF35_DD45), // 0x1.c11d5cf35dd45p+3
    f64::from_bits(0x4004_AA35_6FDC_D9D5), // 0x1.4aa356fdcd9d5p+1
];

/// K1's denominator (see [`K1_N`]), the constant term first; D0 = -1/16.
/// FITTED(T-1941, 4229812 cells, GAMMA x 1..10)
#[rustfmt::skip]
pub(super) const K1_D: [f64; 8] = [
    f64::from_bits(0xBFB0_0000_0000_0000), // -0x1.0000000000000p-4
    f64::from_bits(0x40EB_584A_E618_5B27), // 0x1.b584ae6185b27p+15
    f64::from_bits(0x40EA_5269_4226_B55E), // 0x1.a52694226b55ep+15
    f64::from_bits(0xC0C8_FD07_2611_5AA1), // -0x1.8fd0726115aa1p+13
    f64::from_bits(0xC0BD_079C_BB8D_7BAA), // -0x1.d079cbb8d7baap+12
    f64::from_bits(0x40A8_11C8_9B66_ABB2), // 0x1.811c89b66abb2p+11
    f64::from_bits(0xC079_F2C6_B220_F0DA), // -0x1.9f2c6b220f0dap+8
    f64::from_bits(0x4033_F2E0_3E3A_10E2), // 0x1.3f2e03e3a10e2p+4
];
