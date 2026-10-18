//! Excel's error function kernels, bit-exact on every T-1778 probe (docs/excel-semantics/
//! statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and
//! POISSON (T-1778)"; docs/research/t1778-erf-kernel-research-plan.md, research updates 4, 7, 8
//! and 10).
//!
//! Every operation is x87 code stored to a double: rounded to a 64-bit significand, then to 53
//! bits (`fl53(fl64(.))`, the `extended_*` helpers). Horner steps are
//! `s = fl53(fl64(fl53(fl64(s*x)) + c))`, from the top coefficient. For x >= 0:
//! - 0 <= x <= 0.5: ERF = x*P(z)/Q(z) with z = x*x, P a quartic and Q a quintic in z, both stored
//!   (`small_erf`); ERFC = fl(1 - ERF).
//! - 0.5 < x <= 26.55: ERFC = E*R, with E Excel's own EXP(-x*x) ([`excel_exp`]) and R = N/D, a
//!   rational in x: degrees (9,10) up to 4 and (8,9) above (`erfc_kernel`); ERF = fl(1 - ERFC).
//!   The split sits exactly above 0.5 (x = 0.5 + 2^-53 is the first point of the rational) and
//!   x = 4 still takes the (9,10) piece.
//! - Beyond, and wherever E*R is below `DBL_MIN`, ERFC is 0.
//!
//! Negative x: ERF(-x) = -ERF(x) and ERFC(-x) = fl(2 - ERFC(x)).

use crate::arithmetic::{extended_product, extended_quotient, extended_sum, flush_subnormal};
use crate::x87_exp::excel_exp;

/// Where the small-x rational stops: x = 0.5 still takes it, 0.5 + 2^-53 does not (research
/// update 4, "The split is at x = 0.5 + 2^-53").
const SMALL_X_LIMIT: f64 = 0.5;

/// Where the (8,9) tail takes over: x = 4 still takes the (9,10) piece (research update 7, the
/// `kmyst-erfc-r6*` bisections).
const TAIL_START: f64 = 4.0;

/// Largest x the tail rational covers (`round7/erfc_tail_coef.txt`, 4 < x <= 26.55). ERFC is below
/// `DBL_MIN`, and so flushed to 0, from about x = 26.5432 on (`kmyst-erfc-cutoff`), so 0 beyond
/// this is the flush, not a threshold of Excel's.
const ERFC_UNDERFLOW: f64 = 26.55;

/// Small-x numerator P(z), highest degree first: docs/research/t1778/round3/sol100.txt (`P`, there
/// lowest degree first). P(0) is 2/sqrt(pi) * Q(0) + 1.067E-16, Excel's half-ulp bias on erf.
const SMALL_P: [f64; 5] = [
    f64::from_bits(0x3F29_3276_7DF3_2A4C), // 0x1.932767df32a4cp-13
    f64::from_bits(0x3F5D_93A5_813E_2E4F), // 0x1.d93a5813e2e4fp-10
    f64::from_bits(0x3FA6_F289_D165_B309), // 0x1.6f289d165b309p-5
    f64::from_bits(0x3FC2_01D8_627A_7D2C), // 0x1.201d8627a7d2cp-3
    f64::from_bits(0x3FF1_DF5E_70C7_956B), // 0x1.1df5e70c7956bp+0
];

/// Small-x denominator Q(z), highest degree first: docs/research/t1778/round3/sol100.txt (`Q`).
const SMALL_Q: [f64; 6] = [
    f64::from_bits(0x3EF4_F20A_4710_C8FE), // 0x1.4f20a4710c8fep-16
    f64::from_bits(0x3F46_0975_1B5C_0562), // 0x1.609751b5c0562p-11
    f64::from_bits(0x3F85_6DE5_DA86_AD98), // 0x1.56de5da86ad98p-7
    f64::from_bits(0x3FB7_9F85_BA68_8040), // 0x1.79f85ba68804p-4
    f64::from_bits(0x3FDD_191A_CA8D_2499), // 0x1.d191aca8d2499p-2
    f64::from_bits(0x3FEF_ADA1_5778_ECE6), // 0x1.fada15778ece6p-1
];

/// Numerator N(x) for 0.5 < x <= 4, highest degree first: docs/research/t1778/round9/
/// `erfc_mid_coef.txt` (`N`); each coefficient is unique to the ulp on the probes.
const MID_N: [f64; 10] = [
    f64::from_bits(0x3EFD_ECDB_B07D_9E2F), // 0x1.decdbb07d9e2fp-16
    f64::from_bits(0x3F44_1CFB_B83E_D0E4), // 0x1.41cfbb83ed0e4p-11
    f64::from_bits(0x3F7A_0160_6645_9C2D), // 0x1.a016066459c2dp-8
    f64::from_bits(0x3FA5_0B66_7465_37EA), // 0x1.50b66746537eap-5
    f64::from_bits(0x3FC7_6480_F4D9_7FDA), // 0x1.76480f4d97fdap-3
    f64::from_bits(0x3FE2_7FC8_B2D9_7507), // 0x1.27fc8b2d97507p-1
    f64::from_bits(0x3FF4_DB9F_CFB3_C6E7), // 0x1.4db9fcfb3c6e7p+0
    f64::from_bits(0x4000_4481_1191_D2CB), // 0x1.044811191d2cbp+1
    f64::from_bits(0x4000_223D_9F70_ED04), // 0x1.0223d9f70ed04p+1
    f64::from_bits(0x3FEF_FFFF_FFFF_FF9F), // 0x1.fffffffffff9fp-1
];

/// Denominator D(x) for 0.5 < x <= 4, highest degree first, D(0) = 1: docs/research/t1778/round9/
/// `erfc_mid_coef.txt` (`D`).
const MID_D: [f64; 11] = [
    f64::from_bits(0x3F0A_8542_411A_723B), // 0x1.a8542411a723bp-15
    f64::from_bits(0x3F51_D32A_E54A_52B7), // 0x1.1d32ae54a52b7p-10
    f64::from_bits(0x3F87_1934_F4E7_B285), // 0x1.71934f4e7b285p-7
    f64::from_bits(0x3FB2_CA1B_B9DF_5335), // 0x1.2ca1bb9df5335p-4
    f64::from_bits(0x3FD5_16F2_9541_7CA8), // 0x1.516f295417ca8p-2
    f64::from_bits(0x3FF0_F7F2_7E5D_CCD7), // 0x1.0f7f27e5dccd7p+0
    f64::from_bits(0x4003_BC73_0D3C_1C66), // 0x1.3bc730d3c1c66p+1
    f64::from_bits(0x4010_5350_002D_8FFA), // 0x1.05350002d8ffap+2
    f64::from_bits(0x4012_5449_C578_E4BD), // 0x1.25449c578e4bdp+2
    f64::from_bits(0x4009_2929_4792_38DB), // 0x1.92929479238dbp+1
    f64::from_bits(0x3FF0_0000_0000_0000), // 0x1p+0
];

/// Numerator N(x) for 4 < x <= 26.55, highest degree first: docs/research/t1778/round7/
/// `erfc_tail_coef.txt` (`N`). The last three are equivalent on all data within a few ulps (ranges
/// in that file); these are the file's choices.
const TAIL_N: [f64; 9] = [
    f64::from_bits(0x3FD4_DC75_6D76_1350), // 0x1.4dc756d76135p-2
    f64::from_bits(0x4006_CA0F_8ECF_C798), // 0x1.6ca0f8ecfc798p+1
    f64::from_bits(0x4030_191F_8363_4B97), // 0x1.0191f83634b97p+4
    f64::from_bits(0x404D_936F_5138_EFD9), // 0x1.d936f5138efd9p+5
    f64::from_bits(0x4063_96F0_FC8E_5C9C), // 0x1.396f0fc8e5c9cp+7
    f64::from_bits(0x4072_6C40_9D38_4172), // 0x1.26c409d384172p+8
    f64::from_bits(0x4078_0FB2_B571_6D18), // 0x1.80fb2b5716d18p+8
    f64::from_bits(0x4073_DA26_6CD0_39E0), // 0x1.3da266cd039ep+8
    f64::from_bits(0x4060_00AE_BF22_DE16), // 0x1.000aebf22de16p+7
];

/// Denominator D(x) for 4 < x <= 26.55, highest degree first, D(0) = 128: docs/research/t1778/
/// `round7/erfc_tail_coef.txt` (`D`).
const TAIL_D: [f64; 10] = [
    f64::from_bits(0x3FE2_7CDB_B941_913D), // 0x1.27cdbb941913dp-1
    f64::from_bits(0x4014_324D_3EF0_F238), // 0x1.4324d3ef0f238p+2
    f64::from_bits(0x403C_D273_9010_3553), // 0x1.cd27390103553p+4
    f64::from_bits(0x405A_D794_D2E2_C325), // 0x1.ad794d2e2c325p+6
    f64::from_bits(0x4072_3C05_0F5F_C4A3), // 0x1.23c050f5fc4a3p+8
    f64::from_bits(0x4081_E2D9_55B8_FD9D), // 0x1.1e2d955b8fd9dp+9
    f64::from_bits(0x4089_3DA0_F9DA_BA69), // 0x1.93da0f9daba69p+9
    f64::from_bits(0x4088_50E3_F0A8_DBB6), // 0x1.850e3f0a8dbb6p+9
    f64::from_bits(0x407C_E328_FFEB_78AC), // 0x1.ce328ffeb78acp+8
    f64::from_bits(0x4060_0000_0000_0000), // 0x1p+7
];

/// The polynomial with coefficients `c` (highest degree first) at `x`, by Horner's rule with every
/// product and sum stored to a double through extended precision.
fn horner(c: &[f64], x: f64) -> f64 {
    c.iter()
        .skip(1)
        .fold(c.first().copied().unwrap_or(0.0), |s, &k| {
            extended_sum(extended_product(s, x), k)
        })
}

/// ERF for 0 <= x <= 0.5: fl(fl(x*N)/W), N = P(z) and W = Q(z), z = fl(x*x) (round2/sc.c mode
/// 0300; 775,272 of 775,272 probe points).
fn small_erf(x: f64) -> f64 {
    let z = extended_product(x, x);
    extended_quotient(
        extended_product(x, horner(&SMALL_P, z)),
        horner(&SMALL_Q, z),
    )
}

/// ERFC for 0.5 < x: fl(E*fl(N/D)), with E = Excel's EXP(-fl(x*x)) and a result below `DBL_MIN`
/// flushed to 0 (`round9/mid_check.c` and `round7/erfc_tail_check.c`).
fn erfc_kernel(x: f64) -> f64 {
    if x > ERFC_UNDERFLOW {
        return 0.0;
    }
    let (n, d) = if x <= TAIL_START {
        (&MID_N[..], &MID_D[..])
    } else {
        (&TAIL_N[..], &TAIL_D[..])
    };
    let e = excel_exp(-extended_product(x, x));
    flush_subnormal(extended_product(
        e,
        extended_quotient(horner(n, x), horner(d, x)),
    ))
}

/// The error function erf(x), as Excel's ERF.PRECISE computes it (module docs).
pub(crate) fn erf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x < 0.0 {
        return -erf(-x);
    }
    if x <= SMALL_X_LIMIT {
        small_erf(x)
    } else {
        extended_sum(1.0, -erfc_kernel(x))
    }
}

/// The complementary error function erfc(x) = 1 - erf(x), as Excel's ERFC.PRECISE computes it
/// (module docs).
pub(crate) fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x < 0.0 {
        return extended_sum(2.0, -erfc(-x));
    }
    if x <= SMALL_X_LIMIT {
        extended_sum(1.0, -small_erf(x))
    } else {
        erfc_kernel(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values are excel-graph oracle cells of the T-1778 probes; the curated copies are in
    // eval/suites/probes-numeric/cases/t1916-*. Every comparison is bit for bit.

    #[test]
    fn small_x_rational() {
        // kmyst-erf-gamma!C30: x = 2^-30, where ERF is fl(x*P(0)/Q(0)).
        assert_eq!(erf(9.313_225_746_154_785e-10), 1.050_884_991_041_862_2e-9);
        // kmyst-erf-gamma!C2.
        assert_eq!(erf(0.25), 0.276_326_390_168_237);
        assert_eq!(erf(0.0), 0.0);
    }

    #[test]
    fn split_above_one_half() {
        // x = 0.5 takes the small-x rational (kmyst-erf-gamma!C1, F1); ERFC = fl(1 - ERF).
        assert_eq!(erf(0.5), 0.520_499_877_813_046_5);
        assert_eq!(erfc(0.5), 0.479_500_122_186_953_5);
        // The double below 0.5 (kmyst-erfc-split05!C512).
        assert_eq!(erfc(0.499_999_999_999_999_94), 0.479_500_122_186_953_5);
        // 0.5 + 2^-53 takes the (9,10) kernel (kmyst-erfc-r9side3!B1, kmyst-erf-wbig!B2): ERFC is
        // three ulps lower, and ERF = fl(1 - ERFC) is not fl(1 - ERF) of the small side.
        assert_eq!(erfc(0.500_000_000_000_000_1), 0.479_500_122_186_953_3);
        assert_eq!(erf(0.500_000_000_000_000_1), 0.520_499_877_813_046_7);
    }

    #[test]
    fn mid_rational() {
        // kmyst-erfc-branchscan!B129.
        assert_eq!(erfc(1.0), 0.157_299_207_050_285_13);
    }

    #[test]
    fn split_above_four() {
        // x = 4 takes the (9,10) kernel (kmyst-erfc-branchscan!B1665, kmyst-erfc-r9side3!H4096),
        // the next double the (8,9) tail (kmyst-erfc-branchscan!B1666).
        assert_eq!(erfc(4.0), 1.541_725_790_028_001_7e-8);
        assert_eq!(erf(4.0), 0.999_999_984_582_742_1);
        assert_eq!(erfc(4.000_000_000_000_001), 1.541_725_790_027_990_4e-8);
    }

    #[test]
    fn tail_rational_and_cutoff() {
        // kmyst-erfc-branchscan!B2945 and kmyst-erfc-cutoff!B1.
        assert_eq!(erfc(10.0), 2.088_487_583_762_544_6e-45);
        assert_eq!(erfc(26.5), 2.210_907_664_263_734_6e-307);
        // The last non-zero probe, and the next one, whose result is below DBL_MIN and flushes to 0
        // (kmyst-erfc-cutoff).
        assert_eq!(erfc(26.543_200_618_033_99), 2.231_920_902_025_842e-308);
        assert_eq!(erfc(26.543_300_618_033_99), 0.0);
        assert_eq!(erfc(30.0), 0.0);
        assert_eq!(erf(30.0), 1.0);
    }

    #[test]
    fn negative_x() {
        // ERFC(-x) = fl(2 - ERFC(x)) on both sides of the split (kmyst-erfc-split05!D513, D514).
        assert_eq!(erfc(-0.5), 1.520_499_877_813_046_5);
        assert_eq!(erfc(-0.500_000_000_000_000_1), 1.520_499_877_813_046_7);
        assert_eq!(erf(-0.25), -0.276_326_390_168_237);
        assert_eq!(erfc(-30.0), 2.0);
    }

    #[test]
    fn non_finite() {
        assert!(erf(f64::NAN).is_nan());
        assert!(erfc(f64::NAN).is_nan());
        assert_eq!(erf(f64::INFINITY), 1.0);
        assert_eq!(erfc(f64::NEG_INFINITY), 2.0);
    }
}
