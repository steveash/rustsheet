//! Excel's own `log10`, an x87 `FLDLG2; FYL2X` evaluation (T-2004; docs/excel-semantics/numbers.md
//! "LOG with one argument is LOG10"). FYL2X forms `y * log2(x)` with `y = RN64(log10 2)`, the
//! constant FLDLG2 loads, rounds it to a 64-bit significand, and the store rounds that to a double:
//! two roundings, so it is not always the correctly rounded log10. It matches 92 of 92 cells of
//! probes/log10-kernel (excel-graph), where the correctly rounded log10 matches 78 and glibc's 56.
//! LOG10, one-argument LOG and MULTINOMIAL's `log10(k)` terms use it.

use crate::arithmetic::{round_extended_to_double, significand_and_exponent};
use crate::x87_exp::{dd_div_small, dd_mul, dd_to_extended, two_sum, LN_2_HI, LN_2_LO};

/// `RN64(log10 2) / ln 2` as a double-double (hi + lo), from 80-digit decimal arithmetic:
/// `y * log2(x)` is `ln(x)` times this. RN64(log10 2) is `0x9A209A84FBCFF799 * 2^-65` (log10 2 =
/// `0x9A209A84FBCFF798_8F89`... rounds up).
const LOG10_2_64_OVER_LN_2_HI: f64 = f64::from_bits(0x3FDB_CB7B_1526_E50E); // 0x1.bcb7b1526e50ep-2
const LOG10_2_64_OVER_LN_2_LO: f64 = f64::from_bits(0x3C69_5D79_BCB5_84BC); // 0x1.95d79bcb584bcp-57

/// The atanh series below stops after this many terms: |s| <= 3 - 2√2 < 0.1716, so the next term
/// is under 0.1716^46 < 2^-117 of the first.
const ATANH_TERMS: u32 = 23;

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
    for k in (0..ATANH_TERMS).rev() {
        let term = dd_div_small((1.0, 0.0), f64::from(2 * k + 1));
        series = dd_add(term, dd_mul(s2, series));
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

/// `a + b` in double-double arithmetic.
fn dd_add(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let (s, e) = two_sum(a.0, b.0);
    two_sum(s, e + a.1 + b.1)
}

/// `a / b` in double-double arithmetic: a quotient, then one correction from the remainder.
fn dd_div(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let q = a.0 / b.0;
    let product = dd_mul((q, 0.0), b);
    let remainder = dd_add(a, (-product.0, -product.1));
    two_sum(q, remainder.0 / b.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn non_positive_and_non_finite_are_nan() {
        for x in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(excel_log10(x).is_nan(), "{x}");
        }
    }
}
