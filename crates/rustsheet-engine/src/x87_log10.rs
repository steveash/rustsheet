//! Excel's own `log10` and `ln`, x87 `FLDLG2; FYL2X` and `FLDLN2; FYL2X` evaluations (T-2004,
//! T-2123; docs/excel-semantics/numbers.md "LOG with one argument is LOG10" and "LN is FLDLN2 then
//! FYL2X"). FYL2X forms `y * log2(x)` with `y` the 64-bit constant the load gives (RN64(log10 2)
//! or RN64(ln 2)), rounds it to a 64-bit significand, and the store rounds that to a double: two
//! roundings, so neither is always correctly rounded. LOG10 matches 92 of 92 cells of
//! probes/log10-kernel (excel-graph), where the correctly rounded log10 matches 78 and glibc's 56;
//! LN matches 2,887,521 of the 2,887,551 T-1941 `kmyst-gam-*` LN cells (the misses are FYL2X
//! near-ties, see [`excel_ln`]). LOG10, one-argument LOG and MULTINOMIAL's `log10(k)` terms use [`excel_log10`];
//! the worksheet LN uses [`excel_ln`].

use crate::arithmetic::significand_and_exponent;
use crate::x87_exp::{
    dd_add, dd_div, dd_mul_in_range, dd_rounded_twice, two_sum, LN_2_HI, LN_2_LO,
};

/// `RN64(log10 2) / ln 2` as a double-double (hi + lo), from 80-digit decimal arithmetic:
/// `y * log2(x)` is `ln(x)` times this. RN64(log10 2) is `0x9A209A84FBCFF799 * 2^-65` (log10 2 =
/// `0x9A209A84FBCFF798_8F89`... rounds up).
const LOG10_2_64_OVER_LN_2_HI: f64 = f64::from_bits(0x3FDB_CB7B_1526_E50E); // 0x1.bcb7b1526e50ep-2
const LOG10_2_64_OVER_LN_2_LO: f64 = f64::from_bits(0x3C69_5D79_BCB5_84BC); // 0x1.95d79bcb584bcp-57

/// `RN64(ln 2) / ln 2` as a double-double, from 400-bit mpmath: `y * log2(x)` is `ln(x)` times
/// this. FLDLN2 loads ln 2 rounded up in the last bit of a 64-bit significand,
/// `0xB17217F7D1CF79AC * 2^-64`, so the factor is 1 + 0x1.384291b5ee917p-66 (about 1 + 1.65e-20).
const LN_2_64_OVER_LN_2_HI: f64 = 1.0;
const LN_2_64_OVER_LN_2_LO: f64 = f64::from_bits(0x3BD3_8429_1B5E_E917); // 0x1.384291b5ee917p-66

/// The coefficients 1/(2k+1) of the atanh series as double-doubles, k = 0..23 (from 60-digit
/// decimal arithmetic). [`ln_reduced`] sums the first [`ATANH_TERMS`]; all 23 cover |s| <= 3 - 2√2
/// < 0.1716 (the next term is under 0.1716^46 < 2^-117 of the first), the reduction the tests'
/// reference kernel uses.
#[rustfmt::skip]
const ATANH_COEFFICIENTS: [(f64, f64); 23] = [
    (f64::from_bits(0x3FF0_0000_0000_0000), f64::from_bits(0x0000_0000_0000_0000)), // 1/1
    (f64::from_bits(0x3FD5_5555_5555_5555), f64::from_bits(0x3C75_5555_5555_5555)), // 1/3
    (f64::from_bits(0x3FC9_9999_9999_999A), f64::from_bits(0xBC69_9999_9999_999A)), // 1/5
    (f64::from_bits(0x3FC2_4924_9249_2492), f64::from_bits(0x3C62_4924_9249_2492)), // 1/7
    (f64::from_bits(0x3FBC_71C7_1C71_C71C), f64::from_bits(0x3C5C_71C7_1C71_C71C)), // 1/9
    (f64::from_bits(0x3FB7_45D1_745D_1746), f64::from_bits(0xBC47_45D1_745D_1746)), // 1/11
    (f64::from_bits(0x3FB3_B13B_13B1_3B14), f64::from_bits(0xBC53_B13B_13B1_3B14)), // 1/13
    (f64::from_bits(0x3FB1_1111_1111_1111), f64::from_bits(0x3C31_1111_1111_1111)), // 1/15
    (f64::from_bits(0x3FAE_1E1E_1E1E_1E1E), f64::from_bits(0x3C2E_1E1E_1E1E_1E1E)), // 1/17
    (f64::from_bits(0x3FAA_F286_BCA1_AF28), f64::from_bits(0x3C4A_F286_BCA1_AF28)), // 1/19
    (f64::from_bits(0x3FA8_6186_1861_8618), f64::from_bits(0x3C48_6186_1861_8618)), // 1/21
    (f64::from_bits(0x3FA6_42C8_590B_2164), f64::from_bits(0x3C36_42C8_590B_2164)), // 1/23
    (f64::from_bits(0x3FA4_7AE1_47AE_147B), f64::from_bits(0xBC2E_B851_EB85_1EB8)), // 1/25
    (f64::from_bits(0x3FA2_F684_BDA1_2F68), f64::from_bits(0x3C42_F684_BDA1_2F68)), // 1/27
    (f64::from_bits(0x3FA1_A7B9_611A_7B96), f64::from_bits(0x3C21_A7B9_611A_7B96)), // 1/29
    (f64::from_bits(0x3FA0_8421_0842_1084), f64::from_bits(0x3C30_8421_0842_1084)), // 1/31
    (f64::from_bits(0x3F9F_07C1_F07C_1F08), f64::from_bits(0xBC2F_07C1_F07C_1F08)), // 1/33
    (f64::from_bits(0x3F9D_41D4_1D41_D41D), f64::from_bits(0x3C30_7507_5075_0750)), // 1/35
    (f64::from_bits(0x3F9B_ACF9_14C1_BAD0), f64::from_bits(0xBC3B_ACF9_14C1_BAD0)), // 1/37
    (f64::from_bits(0x3F9A_41A4_1A41_A41A), f64::from_bits(0x3C30_6906_9069_0690)), // 1/39
    (f64::from_bits(0x3F98_F9C1_8F9C_18FA), f64::from_bits(0xBC2F_3831_F383_1F38)), // 1/41
    (f64::from_bits(0x3F97_D05F_417D_05F4), f64::from_bits(0x3C17_D05F_417D_05F4)), // 1/43
    (f64::from_bits(0x3F96_C16C_16C1_6C17), f64::from_bits(0xBC2F_49F4_9F49_F49F)), // 1/45
];

/// The atanh terms [`ln_reduced`] sums: |s| <= 2^-8.5 there, so the first omitted term is under
/// (2^-17)^8 = 2^-136 of the first.
const ATANH_TERMS: usize = 8;

/// The atanh terms from this one on are summed in plain double: s^8 <= 2^-68, so their rounding
/// error is under 2^-120 of the sum.
const ATANH_DOUBLE_TERMS_FROM: usize = 4;

/// The first table point, 91/128 (the nearest i/128 to √2/2 = 90.51/128).
const LN_TABLE_FIRST: f64 = 91.0;

/// ln(i/128) as double-doubles for i = 91..=181, the points i/128 nearest to an f in [√2/2, √2]
/// (from 80-digit decimal arithmetic; ln(128/128) is exactly 0, so x near 1 keeps its relative
/// accuracy).
#[rustfmt::skip]
const LN_TABLE: [(f64, f64); 91] = [
    (f64::from_bits(0xBFD5_D5BD_DF59_5F30), f64::from_bits(0x3C76_5411_48CB_B8A2)), // ln(91/128)
    (f64::from_bits(0xBFD5_22AE_0738_A3D8), f64::from_bits(0x3C68_F7E9_B38A_6979)), // ln(92/128)
    (f64::from_bits(0xBFD4_718D_C271_C41B), f64::from_bits(0xBC38_FB4C_14C5_6EEF)), // ln(93/128)
    (f64::from_bits(0xBFD3_C252_7733_3184), f64::from_bits(0x3C72_AD27_E50A_8EC6)), // ln(94/128)
    (f64::from_bits(0xBFD3_14F1_E1D3_5CE4), f64::from_bits(0x3C73_D699_09E5_C3DC)), // ln(95/128)
    (f64::from_bits(0xBFD2_6962_1134_DB92), f64::from_bits(0xBC7E_0EFA_DD9D_B02B)), // ln(96/128)
    (f64::from_bits(0xBFD1_BF99_635A_6B95), f64::from_bits(0x3C61_2AEB_8424_9223)), // ln(97/128)
    (f64::from_bits(0xBFD1_178E_8227_E47C), f64::from_bits(0x3C60_E63A_5F01_C691)), // ln(98/128)
    (f64::from_bits(0xBFD0_7138_604D_5862), f64::from_bits(0xBC7C_DB16_ED4E_9138)), // ln(99/128)
    (f64::from_bits(0xBFCF_991C_6CB3_B379), f64::from_bits(0xBC6F_6650_66F9_80A2)), // ln(100/128)
    (f64::from_bits(0xBFCE_530E_FFE7_1012), f64::from_bits(0xBC42_2760_41F4_3042)), // ln(101/128)
    (f64::from_bits(0xBFCD_1037_F265_5E7B), f64::from_bits(0xBC66_0629_2424_71A2)), // ln(102/128)
    (f64::from_bits(0xBFCB_D087_383B_D8AD), f64::from_bits(0xBC3D_D355_F6A5_16D7)), // ln(103/128)
    (f64::from_bits(0xBFCA_93ED_3C8A_D9E3), f64::from_bits(0xBC6B_CAFA_9DE9_7203)), // ln(104/128)
    (f64::from_bits(0xBFC9_5A5A_DCF7_017F), f64::from_bits(0xBC51_42C5_07FB_7A3D)), // ln(105/128)
    (f64::from_bits(0xBFC8_23C1_6551_A3C2), f64::from_bits(0x3C61_232C_E70B_E781)), // ln(106/128)
    (f64::from_bits(0xBFC6_F012_8B75_6ABC), f64::from_bits(0x3C68_DE59_C21E_166C)), // ln(107/128)
    (f64::from_bits(0xBFC5_BF40_6B54_3DB2), f64::from_bits(0x3C21_F5B4_4C0D_F7E7)), // ln(108/128)
    (f64::from_bits(0xBFC4_913D_8333_B561), f64::from_bits(0x3C50_D560_4930_F135)), // ln(109/128)
    (f64::from_bits(0xBFC3_65FC_B015_9016), f64::from_bits(0xBC57_D411_A5B9_44AD)), // ln(110/128)
    (f64::from_bits(0xBFC2_3D71_2A49_C202), f64::from_bits(0x3C66_E381_6105_1D69)), // ln(111/128)
    (f64::from_bits(0xBFC1_178E_8227_E47C), f64::from_bits(0x3C50_E63A_5F01_C691)), // ln(112/128)
    (f64::from_bits(0xBFBF_E891_39DB_D566), f64::from_bits(0x3C5A_C9F4_215F_9393)), // ln(113/128)
    (f64::from_bits(0xBFBD_A727_6384_46A2), f64::from_bits(0xBC54_01FA_7173_3019)), // ln(114/128)
    (f64::from_bits(0xBFBB_6AC8_8DAD_5B1C), f64::from_bits(0x3C40_057E_ED1C_A59F)), // ln(115/128)
    (f64::from_bits(0xBFB9_335E_5D59_4989), f64::from_bits(0x3C54_78A8_5704_CCB7)), // ln(116/128)
    (f64::from_bits(0xBFB7_00D3_0AEA_C0E1), f64::from_bits(0x3C27_2566_212C_DD05)), // ln(117/128)
    (f64::from_bits(0xBFB4_D311_5D20_7EAC), f64::from_bits(0xBC57_69F4_2C78_42CC)), // ln(118/128)
    (f64::from_bits(0xBFB2_AA04_A447_17A5), f64::from_bits(0x3C5D_15D3_8D2F_A3F7)), // ln(119/128)
    (f64::from_bits(0xBFB0_8598_B59E_3A07), f64::from_bits(0x3C5D_D700_9902_BF32)), // ln(120/128)
    (f64::from_bits(0xBFAC_CB73_CDDD_B2CC), f64::from_bits(0x3C4E_48FB_0500_EFD4)), // ln(121/128)
    (f64::from_bits(0xBFA8_94AA_149F_B343), f64::from_bits(0xBC3A_8BE9_7660_A23D)), // ln(122/128)
    (f64::from_bits(0xBFA4_66AE_D42D_E3EA), f64::from_bits(0x3C4C_DD6F_7F4A_137E)), // ln(123/128)
    (f64::from_bits(0xBFA0_415D_89E7_4444), f64::from_bits(0xBC4C_05CF_1D75_3622)), // ln(124/128)
    (f64::from_bits(0xBF98_4925_28C8_CABF), f64::from_bits(0x3C3D_192D_0619_FA67)), // ln(125/128)
    (f64::from_bits(0xBF90_2056_5893_5847), f64::from_bits(0xBC32_7C8E_8416_E71F)), // ln(126/128)
    (f64::from_bits(0xBF80_1015_7588_DE71), f64::from_bits(0xBC14_6662_D417_CED0)), // ln(127/128)
    (f64::from_bits(0x0000_0000_0000_0000), f64::from_bits(0x0000_0000_0000_0000)), // ln(128/128)
    (f64::from_bits(0x3F7F_E02A_6B10_6789), f64::from_bits(0xBBCE_44B7_E371_1EBF)), // ln(129/128)
    (f64::from_bits(0x3F8F_C0A8_B0FC_03E4), f64::from_bits(0xBC18_3092_C596_42A1)), // ln(130/128)
    (f64::from_bits(0x3F97_B91B_07D5_B11B), f64::from_bits(0xBC35_B602_ACE3_A510)), // ln(131/128)
    (f64::from_bits(0x3F9F_829B_0E78_3300), f64::from_bits(0x3C33_3E3F_04F1_EF23)), // ln(132/128)
    (f64::from_bits(0x3FA3_9E87_B9FE_BD60), f64::from_bits(0xBC45_BFA9_37F5_51BB)), // ln(133/128)
    (f64::from_bits(0x3FA7_7458_F632_DCFC), f64::from_bits(0x3C41_8D3C_A87B_9296)), // ln(134/128)
    (f64::from_bits(0x3FAB_42DD_7119_71BF), f64::from_bits(0xBC3E_B975_9C13_0499)), // ln(135/128)
    (f64::from_bits(0x3FAF_0A30_C011_62A6), f64::from_bits(0x3C48_5F32_5C5B_BACD)), // ln(136/128)
    (f64::from_bits(0x3FB1_6536_EEA3_7AE1), f64::from_bits(0xBC37_9DA3_E8C2_2CDA)), // ln(137/128)
    (f64::from_bits(0x3FB3_41D7_961B_D1D1), f64::from_bits(0xBC5B_599F_227B_ECBB)), // ln(138/128)
    (f64::from_bits(0x3FB5_1B07_3F06_183F), f64::from_bits(0x3C5A_49E3_9A1A_8BE4)), // ln(139/128)
    (f64::from_bits(0x3FB6_F0D2_8AE5_6B4C), f64::from_bits(0xBC59_06D9_9184_B992)), // ln(140/128)
    (f64::from_bits(0x3FB8_C345_D631_9B21), f64::from_bits(0xBC24_A697_AB34_24A9)), // ln(141/128)
    (f64::from_bits(0x3FBA_926D_3A4A_D563), f64::from_bits(0x3C59_42F4_8AA7_0EA9)), // ln(142/128)
    (f64::from_bits(0x3FBC_5E54_8F5B_C743), f64::from_bits(0x3C35_D617_EF81_61B1)), // ln(143/128)
    (f64::from_bits(0x3FBE_2707_6E2A_F2E6), f64::from_bits(0xBC36_1578_001E_0162)), // ln(144/128)
    (f64::from_bits(0x3FBF_EC91_31DB_EABB), f64::from_bits(0xBC55_746B_9981_B36C)), // ln(145/128)
    (f64::from_bits(0x3FC0_D77E_7CD0_8E59), f64::from_bits(0x3C69_A5DC_5E90_30AC)), // ln(146/128)
    (f64::from_bits(0x3FC1_B72A_D52F_67A0), f64::from_bits(0x3C54_8302_3472_CD74)), // ln(147/128)
    (f64::from_bits(0x3FC2_9552_F81F_F523), f64::from_bits(0x3C63_0177_1C40_7DBF)), // ln(148/128)
    (f64::from_bits(0x3FC3_71FC_201E_8F74), f64::from_bits(0x3C5D_E6CB_62AF_18A0)), // ln(149/128)
    (f64::from_bits(0x3FC4_4D2B_6CCB_7D1E), f64::from_bits(0x3C69_F4F6_543E_1F88)), // ln(150/128)
    (f64::from_bits(0x3FC5_26E5_E3A1_B438), f64::from_bits(0xBC67_46FF_8A47_0D3A)), // ln(151/128)
    (f64::from_bits(0x3FC5_FF30_70A7_93D4), f64::from_bits(0xBC5B_C60E_FAFC_6F6E)), // ln(152/128)
    (f64::from_bits(0x3FC6_D60F_E719_D21D), f64::from_bits(0xBC6C_AAE2_68EC_D179)), // ln(153/128)
    (f64::from_bits(0x3FC7_AB89_0210_D909), f64::from_bits(0x3C4B_E36B_2D6A_0608)), // ln(154/128)
    (f64::from_bits(0x3FC8_7FA0_6520_C911), f64::from_bits(0xBC6B_F7FD_BFA0_8D9A)), // ln(155/128)
    (f64::from_bits(0x3FC9_525A_9CF4_56B4), f64::from_bits(0x3C6D_904C_1D4E_2E26)), // ln(156/128)
    (f64::from_bits(0x3FCA_23BC_1FE2_B563), f64::from_bits(0x3C49_3711_B07A_998C)), // ln(157/128)
    (f64::from_bits(0x3FCA_F3C9_4E80_BFF3), f64::from_bits(0xBC53_98CF_F364_1985)), // ln(158/128)
    (f64::from_bits(0x3FCB_C286_742D_8CD6), f64::from_bits(0x3C54_FCE7_4487_0F55)), // ln(159/128)
    (f64::from_bits(0x3FCC_8FF7_C79A_9A22), f64::from_bits(0xBC64_F689_F843_4012)), // ln(160/128)
    (f64::from_bits(0x3FCD_5C21_6B4F_BB91), f64::from_bits(0x3C66_E443_597E_4D40)), // ln(161/128)
    (f64::from_bits(0x3FCE_2707_6E2A_F2E6), f64::from_bits(0xBC46_1578_001E_0162)), // ln(162/128)
    (f64::from_bits(0x3FCE_F0AD_CBDC_5936), f64::from_bits(0x3C64_8637_950D_C20D)), // ln(163/128)
    (f64::from_bits(0x3FCF_B918_6D5E_3E2B), f64::from_bits(0xBC6C_AAAE_64F2_1ACB)), // ln(164/128)
    (f64::from_bits(0x3FD0_4025_94B4_D041), f64::from_bits(0xBC62_8EC2_17A5_022D)), // ln(165/128)
    (f64::from_bits(0x3FD0_A324_E273_90E3), f64::from_bits(0x3C77_DCFD_E806_1C03)), // ln(166/128)
    (f64::from_bits(0x3FD1_058B_F9AE_4AD5), f64::from_bits(0x3C58_9FA0_AB4C_B31D)), // ln(167/128)
    (f64::from_bits(0x3FD1_675C_ABAB_A60E), f64::from_bits(0x3C2C_E63E_AB88_3717)), // ln(168/128)
    (f64::from_bits(0x3FD1_C898_C169_99FB), f64::from_bits(0xBC30_E5C6_2AFF_1C44)), // ln(169/128)
    (f64::from_bits(0x3FD2_2941_FBCF_7966), f64::from_bits(0xBC77_6F5E_B096_28AF)), // ln(170/128)
    (f64::from_bits(0x3FD2_895A_13DE_86A3), f64::from_bits(0x3C77_AD24_C13F_040E)), // ln(171/128)
    (f64::from_bits(0x3FD2_E8E2_BAE1_1D31), f64::from_bits(0xBC78_F4CD_B95E_BDF9)), // ln(172/128)
    (f64::from_bits(0x3FD3_47DD_9A98_7D55), f64::from_bits(0xBC64_DD4C_5809_19F8)), // ln(173/128)
    (f64::from_bits(0x3FD3_A64C_5569_45EA), f64::from_bits(0xBC6C_6865_1945_F97C)), // ln(174/128)
    (f64::from_bits(0x3FD4_0430_8686_A7E4), f64::from_bits(0xBC70_BCFB_6082_CE6D)), // ln(175/128)
    (f64::from_bits(0x3FD4_618B_C21C_5EC2), f64::from_bits(0x3C7F_42DE_CDEC_CF1D)), // ln(176/128)
    (f64::from_bits(0x3FD4_BE5F_9577_78A1), f64::from_bits(0xBC62_59B3_5B04_813D)), // ln(177/128)
    (f64::from_bits(0x3FD5_1AAD_872D_F82D), f64::from_bits(0x3C43_927A_C19F_55E3)), // ln(178/128)
    (f64::from_bits(0x3FD5_7677_1745_5A6C), f64::from_bits(0x3C75_26AD_B283_660C)), // ln(179/128)
    (f64::from_bits(0x3FD5_D1BD_BF58_09CA), f64::from_bits(0x3C74_2363_83DC_7FE1)), // ln(180/128)
    (f64::from_bits(0x3FD6_2C82_F2B9_C795), f64::from_bits(0x3C67_B7AF_9153_00E5)), // ln(181/128)
];

/// Excel's LOG10(x) for a positive finite `x` (module docs); anything else is NaN.
pub(crate) fn excel_log10(x: f64) -> f64 {
    fyl2x(x, (LOG10_2_64_OVER_LN_2_HI, LOG10_2_64_OVER_LN_2_LO))
}

/// Excel's LN(x) for a positive finite `x`: RN53(RN64(RN64(ln 2) * log2 x)) (module docs);
/// anything else is NaN. The research model (`docs/research/t1941/lnmodels.py`, on branch
/// `agent/research/T-1941-crack-gammaln-gamma-ln` until it merges) matches 2,887,521 of
/// 2,887,551 Excel cells; each of the 30 misses (one ulp, e.g. x = 0.612354100000274) is an FYL2X
/// near-tie, where Excel's own result varies between runs (probes/ln-fyl2x-kernel), and this
/// follows the model.
pub(crate) fn excel_ln(x: f64) -> f64 {
    fyl2x(x, (LN_2_64_OVER_LN_2_HI, LN_2_64_OVER_LN_2_LO))
}

/// `y * log2(x)` stored as FYL2X does, for `y_over_ln_2` = y / ln 2 as a double-double: ln x in
/// double-double (about 2^-104 relative), times the factor, rounded to a 64-bit significand and
/// then to a double. A positive finite `x` only; anything else is NaN.
fn fyl2x(x: f64, y_over_ln_2: (f64, f64)) -> f64 {
    if !(x > 0.0 && x.is_finite()) {
        return f64::NAN;
    }
    if x == 1.0 {
        return 0.0;
    }
    let log = dd_mul_in_range(ln_double_double(x), y_over_ln_2);
    // FYL2X's 64-bit rounding, then the store's rounding to a double (|log| < 745, never tiny:
    // the smallest, at x = 1 ± 2^-53, is about 4.8e-17 for log10 and 1.1e-16 for ln).
    dd_rounded_twice(log)
}

/// ln of a positive double-double `hi + lo` (|lo| <= ulp(hi)/2): [`ln_double_double`] of the head
/// plus lo/hi, the first term of ln(1 + lo/hi), whose square is below 2^-106.
pub(crate) fn ln_double_double_pair((hi, lo): (f64, f64)) -> (f64, f64) {
    dd_add(ln_double_double(hi), (lo / hi, 0.0))
}

/// ln x in double-double (about 2^-104 relative) for a positive finite `x`: ln x = e ln 2 + ln f
/// with x = f 2^e, f in [√2/2, √2]. [`fyl2x`]'s logarithm before its roundings, also F.TEST's
/// accurate ln B (`f_test_tail.rs`, T-2681) and, through [`ln_double_double_pair`], the accurate
/// half-integer ln B and F tails (`bratio/ln_beta.rs`, `incomplete_beta.rs`, T-2219).
pub(crate) fn ln_double_double(x: f64) -> (f64, f64) {
    // x = f * 2^e with f in [√2/2, √2], f a double (an exact scaling of x's significand).
    let (significand, exponent) = significand_and_exponent(x);
    let mut f = significand as f64 * f64::EPSILON; // in [1, 2)
    let mut e = exponent + 52;
    if f > std::f64::consts::SQRT_2 {
        f *= 0.5;
        e += 1;
    }
    // ln x = e ln 2 + ln f; |e| <= 1075 is exact in a double.
    dd_add(
        dd_mul_in_range((f64::from(e), 0.0), (LN_2_HI, LN_2_LO)),
        ln_reduced(f),
    )
}

/// ln f in double-double (about 2^-104 relative) for f in [√2/2, √2]: ln c + 2 atanh(s) with
/// c = i/128 the table point nearest f and s = (f - c)/(f + c), |s| <= (1/256)/(2 · 0.707) < 2^-8.5,
/// so [`ATANH_TERMS`] terms do what 23 do over the untabulated |s| < 0.1716 (T-2016).
fn ln_reduced(f: f64) -> (f64, f64) {
    let point = (f * 128.0).round();
    let c = point / 128.0;
    // point is 91..=181 for f in [√2/2, √2]; the clamp only keeps the index in bounds.
    let index = ((point - LN_TABLE_FIRST).max(0.0) as usize).min(LN_TABLE.len() - 1);
    let ln_c = LN_TABLE[index];
    // f - c is exact (Sterbenz: c/2 <= f <= 2c), f + c is a double-double.
    let s = dd_div((f - c, 0.0), two_sum(f, c));
    let s2 = dd_mul_in_range(s, s);
    // atanh(s)/s = sum of s^(2k)/(2k+1), by Horner from the last term: the small tail in double,
    // the leading terms in double-double.
    let mut tail = 0.0;
    for &(coefficient, _) in ATANH_COEFFICIENTS[ATANH_DOUBLE_TERMS_FROM..ATANH_TERMS]
        .iter()
        .rev()
    {
        tail = coefficient + s2.0 * tail;
    }
    let mut series = (tail, 0.0);
    for &coefficient in ATANH_COEFFICIENTS[..ATANH_DOUBLE_TERMS_FROM].iter().rev() {
        series = dd_add(coefficient, dd_mul_in_range(s2, series));
    }
    let atanh = dd_mul_in_range(s, series);
    dd_add(ln_c, (2.0 * atanh.0, 2.0 * atanh.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x87_exp::{dd_div_small, dd_mul};

    /// The kernel before T-2016, as a reference: ln f = 2 atanh((f - 1)/(f + 1)) by all 23 terms,
    /// with no table.
    fn fyl2x_untabulated(x: f64, y_over_ln_2: (f64, f64)) -> f64 {
        let (significand, exponent) = significand_and_exponent(x);
        let mut f = significand as f64 * f64::EPSILON;
        let mut e = exponent + 52;
        if f > std::f64::consts::SQRT_2 {
            f *= 0.5;
            e += 1;
        }
        let s = dd_div((f - 1.0, 0.0), two_sum(f, 1.0));
        let s2 = dd_mul(s, s);
        let mut series = (0.0, 0.0);
        for &coefficient in ATANH_COEFFICIENTS.iter().rev() {
            series = dd_add(coefficient, dd_mul(s2, series));
        }
        let ln_f = dd_mul(s, series);
        let ln_x = dd_add(
            dd_mul((f64::from(e), 0.0), (LN_2_HI, LN_2_LO)),
            (2.0 * ln_f.0, 2.0 * ln_f.1),
        );
        let log = dd_mul(ln_x, y_over_ln_2);
        dd_rounded_twice(log)
    }

    /// The table-driven kernel returns the untabulated kernel's bits for LOG10 and LN: on random
    /// doubles over the whole positive range (subnormals included), on every double within 2^12
    /// ulp of 1, of the table points and of the reduction's cut at √2/2 and √2, and on the
    /// probes/log10-kernel edge inputs.
    #[test]
    fn the_table_returns_the_untabulated_bits() {
        let mut inputs = vec![
            1.0 + f64::EPSILON,
            1.0 - f64::EPSILON / 2.0,
            1.0 + 1e-10,
            1.0 - 1e-10,
            5e-324,
            2.2e-308,
            f64::MIN_POSITIVE,
            f64::MAX,
        ];
        let mut centres = vec![
            1.0,
            std::f64::consts::SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ];
        centres.extend((91..=181).map(|i| f64::from(i) / 128.0));
        centres.extend((91..=181).map(|i| (f64::from(i) - 0.5) / 128.0));
        for centre in centres {
            for step in -4096_i64..=4096 {
                inputs.push(f64::from_bits(centre.to_bits().wrapping_add_signed(step)));
            }
        }
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        for _ in 0..400_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            inputs.push(f64::from_bits(state & 0x7FEF_FFFF_FFFF_FFFF));
        }
        for x in inputs.into_iter().filter(|x| *x > 0.0 && x.is_finite()) {
            for (name, y) in [
                ("LOG10", (LOG10_2_64_OVER_LN_2_HI, LOG10_2_64_OVER_LN_2_LO)),
                ("LN", (LN_2_64_OVER_LN_2_HI, LN_2_64_OVER_LN_2_LO)),
            ] {
                assert_eq!(
                    fyl2x(x, y).to_bits(),
                    fyl2x_untabulated(x, y).to_bits(),
                    "{name}({x:e})"
                );
            }
        }
    }

    /// The table is ln(i/128) to double-double precision: the untabulated series agrees to
    /// 2^-100 relative (the table itself is from 80-digit decimal arithmetic).
    #[test]
    fn the_table_is_ln_of_its_points() {
        for (offset, &(hi, lo)) in LN_TABLE.iter().enumerate() {
            let c = (LN_TABLE_FIRST + offset as f64) / 128.0;
            let (f, e) = if c > std::f64::consts::SQRT_2 {
                (c * 0.5, 1.0)
            } else {
                (c, 0.0)
            };
            let s = dd_div((f - 1.0, 0.0), two_sum(f, 1.0));
            let s2 = dd_mul(s, s);
            let mut series = (0.0, 0.0);
            for &coefficient in ATANH_COEFFICIENTS.iter().rev() {
                series = dd_add(coefficient, dd_mul(s2, series));
            }
            let atanh = dd_mul(s, series);
            let want = dd_add(
                dd_mul((e, 0.0), (LN_2_HI, LN_2_LO)),
                (2.0 * atanh.0, 2.0 * atanh.1),
            );
            let difference = (hi - want.0) + (lo - want.1);
            assert!(difference.abs() <= hi.abs() * 2f64.powi(-100), "ln({c})");
        }
    }

    /// Powers of ten are exact, and x = 1 is 0.
    #[test]
    fn powers_of_ten_are_exact() {
        assert_eq!(excel_log10(1.0), 0.0);
        for k in -20..=20 {
            let x: f64 = format!("1e{k}").parse().unwrap_or(1.0);
            assert_eq!(excel_log10(x), f64::from(k), "1e{k}");
        }
    }

    /// probes/log10-kernel (excel-graph): glibc's log10 misses the first three, the correctly
    /// rounded log10 misses the next three (FYL2X's two roundings), and the desktop cell
    /// LOG10(1.3) (ironcalc `LOG_LOG10_LN.xlsx`) agrees.
    #[test]
    fn excel_cells() {
        assert_eq!(excel_log10(11.0), 1.041_392_685_158_225_1);
        assert_eq!(excel_log10(40.0), 1.602_059_991_327_962_3);
        assert_eq!(excel_log10(0.004), -2.397_940_008_672_037_5);
        assert_eq!(excel_log10(295_344_268.0), 8.470_328_546_549_39);
        assert_eq!(excel_log10(616.376_978_144_439_4), 2.789_846_409_326_800_7);
        assert_eq!(
            excel_log10(2.154_116_618_830_740_6e-261),
            -260.666_730_788_715_1
        );
        assert_eq!(excel_log10(1.3), 0.113_943_352_306_836_79);
    }

    /// The integers where glibc's log10 is one ulp off and MULTINOMIAL's deleted literal table
    /// held the correctly rounded value, which the model equals on 1..=170 (T-1190, T-1600).
    #[test]
    fn multinomial_terms_are_the_correctly_rounded_log10() {
        for (k, want) in [
            (11.0, 1.041_392_685_158_225_1),
            (40.0, 1.602_059_991_327_962_3),
            (43.0, 1.633_468_455_579_586_4),
            (53.0, 1.724_275_869_600_789),
            (85.0, 1.929_418_925_714_292_6),
            (90.0, 1.954_242_509_439_325),
            (113.0, 2.053_078_443_483_419_5),
            (119.0, 2.075_546_961_392_530_6),
            (121.0, 2.082_785_370_316_450_3),
            (160.0, 2.204_119_982_655_924_6),
        ] {
            assert_eq!(excel_log10(k), want, "log10({k})");
        }
    }

    /// The series coefficients are 1/(2k+1) to double-double precision.
    #[test]
    fn atanh_coefficients_are_the_reciprocals() {
        for (k, &(hi, lo)) in ATANH_COEFFICIENTS.iter().enumerate() {
            let n = f64::from(u32::try_from(2 * k + 1).unwrap_or(1));
            let (want_hi, want_lo) = dd_div_small((1.0, 0.0), n);
            assert_eq!(hi, want_hi, "1/{n}");
            assert!((lo - want_lo).abs() <= hi * 1e-31, "1/{n}");
        }
    }

    /// The committed sample of the T-1941 LN runs (`test/fixtures/ln_kernel.fixture`: 4,000 random distinct x
    /// plus all 30 points where Excel differs from the research model): the engine equals the
    /// Python model (`docs/research/t1941/lnmodels.py`) on every point, and Excel on all but those.
    #[test]
    fn ln_equals_the_research_model_on_the_fixture() {
        let rows =
            crate::test::kernel_rows::fixture_rows(include_str!("test/fixtures/ln_kernel.fixture"));
        assert_eq!(rows.len(), 4030);
        let mut excel_misses = 0;
        for row in &rows {
            let ln = excel_ln(row.x);
            assert_eq!(
                ln.to_bits(),
                row.model.to_bits(),
                "LN({:e}) vs the model",
                row.x
            );
            if ln.to_bits() != row.excel.to_bits() {
                excel_misses += 1;
            }
        }
        assert_eq!(excel_misses, 30);
    }

    /// LN(1) is 0, LN(e) and LN of powers of two are as Excel gives them, and the near-tie the
    /// research found first (x = 0.612354100000274) follows the model, one ulp from Excel.
    #[test]
    fn ln_cells() {
        assert_eq!(excel_ln(1.0), 0.0);
        assert_eq!(excel_ln(std::f64::consts::E), 1.0);
        assert_eq!(excel_ln(2.0), std::f64::consts::LN_2);
        assert_eq!(excel_ln(0.612_354_100_000_274), -0.490_444_569_019_538_53);
        for x in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(excel_ln(x).is_nan(), "{x}");
        }
        assert!(excel_ln(f64::MIN_POSITIVE / 4.0).is_finite());
    }

    /// The full T-1941 LN runs (`kmyst-gam-*`, every cell whose formula is `=LN(A<row>)`), when
    /// `RUSTSHEET_T1941_ROWS` points at the extracted rows (`crate::test::kernel_rows`): Excel's
    /// bits on 2,887,521 of 2,887,551 cells, the 30 misses being the model's own (numbers.md).
    #[test]
    #[ignore = "needs the T-1941 research data (RUSTSHEET_T1941_ROWS)"]
    fn ln_matches_the_t1941_runs() {
        let Some(rows) = crate::test::kernel_rows::full_rows("ln") else {
            return;
        };
        let exact = crate::test::kernel_rows::exact_count(&rows, excel_ln);
        assert_eq!((exact, rows.len()), (2_887_521, 2_887_551));
    }

    #[test]
    fn non_positive_and_non_finite_are_nan() {
        for x in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(excel_log10(x).is_nan(), "{x}");
        }
    }
}
