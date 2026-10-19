//! Excel's own `log10`, an x87 `FLDLG2; FYL2X` evaluation (T-2004; docs/excel-semantics/numbers.md
//! "LOG with one argument is LOG10"). FYL2X forms `y * log2(x)` with `y = RN64(log10 2)`, the
//! constant FLDLG2 loads, rounds it to a 64-bit significand, and the store rounds that to a double:
//! two roundings, so it is not always the correctly rounded log10. It matches 92 of 92 cells of
//! probes/log10-kernel (excel-graph), where the correctly rounded log10 matches 78 and glibc's 56.
//! LOG10, one-argument LOG and MULTINOMIAL's `log10(k)` terms use it.

use crate::arithmetic::{round_extended_to_double, significand_and_exponent};
use crate::x87_exp::{dd_add, dd_div, dd_mul, dd_to_extended, two_sum, LN_2_HI, LN_2_LO};

/// `RN64(log10 2) / ln 2` as a double-double (hi + lo), from 80-digit decimal arithmetic:
/// `y * log2(x)` is `ln(x)` times this. RN64(log10 2) is `0x9A209A84FBCFF799 * 2^-65` (log10 2 =
/// `0x9A209A84FBCFF798_8F89`... rounds up).
const LOG10_2_64_OVER_LN_2_HI: f64 = f64::from_bits(0x3FDB_CB7B_1526_E50E); // 0x1.bcb7b1526e50ep-2
const LOG10_2_64_OVER_LN_2_LO: f64 = f64::from_bits(0x3C69_5D79_BCB5_84BC); // 0x1.95d79bcb584bcp-57

/// The coefficients 1/(2k+1) of the atanh series below as double-doubles, k = 0..23 (from 60-digit
/// decimal arithmetic): |s| <= 3 - 2√2 < 0.1716, so the next term is under 0.1716^46 < 2^-117 of
/// the first.
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

/// Excel's LOG10(x) for a positive finite `x` (module docs); anything else is NaN.
pub(crate) fn excel_log10(x: f64) -> f64 {
    if !(x > 0.0 && x.is_finite()) {
        return f64::NAN;
    }
    if x == 1.0 {
        return 0.0;
    }
    // x = f * 2^e with f in [√2/2, √2], f a double (an exact scaling of x's significand).
    let (significand, exponent) = significand_and_exponent(x);
    let mut f = significand as f64 * f64::EPSILON; // in [1, 2)
    let mut e = exponent + 52;
    if f > std::f64::consts::SQRT_2 {
        f *= 0.5;
        e += 1;
    }
    // ln f = 2 atanh(s), s = (f - 1)/(f + 1); f - 1 is exact (Sterbenz), f + 1 is a double-double.
    let denominator = two_sum(f, 1.0);
    let s = dd_div((f - 1.0, 0.0), denominator);
    let s2 = dd_mul(s, s);
    // atanh(s)/s = sum of s^(2k)/(2k+1), by Horner from the last term.
    let mut series = (0.0, 0.0);
    for &coefficient in ATANH_COEFFICIENTS.iter().rev() {
        series = dd_add(coefficient, dd_mul(s2, series));
    }
    let ln_f = dd_mul(s, series);
    let ln_f = (2.0 * ln_f.0, 2.0 * ln_f.1);
    // ln x = e ln 2 + ln f; |e| <= 1075 is exact in a double.
    let ln_x = dd_add(dd_mul((f64::from(e), 0.0), (LN_2_HI, LN_2_LO)), ln_f);
    let log = dd_mul(ln_x, (LOG10_2_64_OVER_LN_2_HI, LOG10_2_64_OVER_LN_2_LO));
    // FYL2X's 64-bit rounding, then the store's rounding to a double (|log| < 324, never tiny:
    // the smallest, at x = 1 ± 2^-53, is about 4.8e-17).
    let negative = log.0 < 0.0;
    let magnitude = if negative { (-log.0, -log.1) } else { log };
    let (extended, extended_exponent) = dd_to_extended(magnitude);
    round_extended_to_double(extended, extended_exponent, negative).unwrap_or(log.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x87_exp::dd_div_small;

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

    #[test]
    fn non_positive_and_non_finite_are_nan() {
        for x in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(excel_log10(x).is_nan(), "{x}");
        }
    }
}
