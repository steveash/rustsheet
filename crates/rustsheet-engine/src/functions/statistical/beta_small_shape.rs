//! Excel's beta density x^(a-1) (1-x)^(b-1) / B(a, b) for min(a, b) <= 1, BETA.DIST(x,a,b,FALSE)'s
//! model M2 (T-2513 r3-m1-residual, verifier-confirmed; docs/excel-semantics/statistics.md "Beta
//! density for min(a,b) <= 1: lnB and a double-double exponent split (T-2464, T-2513)"; reference
//! code `docs/research/t2464/r3-m1-residual/final_score.py` and `verify-r3-m1-residual/vlib.py`).
//!
//! Every named step is an x87 operation stored to a double (`arithmetic.rs` `extended_*`), on
//! Excel's own LN, log1p and GAMMALN:
//! ```text
//! tx = fl(fl(a-1)·LN(x));  ty = fl(fl(b-1)·ly)     ly = Excel's log1p(-x) (x <= 0.375), else LN(fl(1 - x))
//! lnB: a = 1 or b = 1     -> -LN(the other shape)
//!      min < 1, max <= 8  -> fl(GL(min) + fl(GL(max) - GL(fl(a+b))))      GL = GAMMALN
//!      min < 1, max > 8   -> fl(GL(min) + algdiv(min, max))               TOMS 708 algdiv in doubles
//! hi = fl(fl(tx + ty) - lnB) in doubles;   lo = (tx + ty - lnB) - hi      (both additions' errors)
//! density = fl(EXP(hi)·fl(1 + lo))                                        EXP = Excel's EXP
//! ```
//! The products' rounding errors are not carried; both additions' are. Excel's own algdiv is not
//! identified (T-2512): TOMS's in doubles is the best known, about 1 ulp of lnB off on most
//! min < 1 < 8 < max shapes. The GL branch is bit-exact where the engine's GAMMALN is Excel's
//! (T-2122 owns its errors below 1).

use super::gammaln::excel_gammaln;
use crate::arithmetic::{extended_product, extended_sum};
use crate::x87_exp::{excel_exp, two_sum};
use crate::x87_log10::excel_ln;
use crate::x87_log1p::excel_ln_one_minus;

/// Up to this larger shape (inclusive, where TOMS 708 betaln switches at < 8) lnB is the GAMMALN
/// form: Excel's (0.25, 8), (0.5, 8), (8, 0.25), (0.4, 8) and (8, 0.66) fit it and not algdiv
/// (verify-r3-m1-residual `vchecks.out`).
const GAMMALN_FORM_MAX: f64 = 8.0;

/// TOMS 708 algdiv's Stirling-correction coefficients c0..c5.
const ALGDIV_C: [f64; 6] = [
    0.083_333_333_333_333_3,
    -0.002_777_777_777_609_91,
    7.936_506_668_253_9e-4,
    -5.952_029_313_518_7e-4,
    8.373_080_340_312_15e-4,
    -0.001_653_229_627_807_13,
];

/// TOMS 708 alnrel's rational coefficients p1..p3 and q1..q3, for |a| <= 0.375.
const ALNREL_P: [f64; 3] = [
    -1.294_189_230_219_93,
    0.405_303_492_862_024,
    -0.017_887_454_601_221_4,
];
const ALNREL_Q: [f64; 3] = [
    -1.627_522_563_553_23,
    0.747_811_014_037_616,
    -0.084_510_421_794_556_5,
];

/// Excel's beta density for 0 < x < 1 and shapes with min(a, b) <= 1 (module docs). `None`
/// outside that domain, and where the exponent or the density is not finite (x at the edge of a
/// shape below 1), where the caller keeps its other density.
pub(crate) fn small_shape_density(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(x > 0.0 && x < 1.0 && a > 0.0 && b > 0.0 && a.min(b) <= 1.0 && (a + b).is_finite()) {
        return None;
    }
    let tx = extended_product(extended_sum(a, -1.0), excel_ln(x));
    let ty = extended_product(extended_sum(b, -1.0), excel_ln_one_minus(x));
    let ln_beta = small_shape_ln_beta(a, b);
    let (sum, sum_error) = two_sum(tx, ty);
    let (hi, hi_error) = two_sum(sum, -ln_beta);
    if !hi.is_finite() {
        return None;
    }
    // The tail is far below ulp(1), so rounding the two errors' sum first does not move fl(1 + lo)
    // except on an exact tie of that sum with a half ulp of 1.
    let density = excel_exp(hi) * (1.0 + (sum_error + hi_error));
    density.is_finite().then_some(density)
}

/// ln B(a, b) as Excel's density takes it for min(a, b) <= 1 (module docs).
fn small_shape_ln_beta(a: f64, b: f64) -> f64 {
    // B(a, 1) = 1/a: Excel's lnB is -LN of the other shape at any size, (0.37, 1) included, where
    // the GAMMALN form is 1 ulp high (statistics.md, verify-r3-m1-residual).
    if a == 1.0 {
        return -excel_ln(b);
    }
    if b == 1.0 {
        return -excel_ln(a);
    }
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    if large <= GAMMALN_FORM_MAX {
        let shift = extended_sum(excel_gammaln(large), -excel_gammaln(extended_sum(a, b)));
        return extended_sum(excel_gammaln(small), shift);
    }
    extended_sum(excel_gammaln(small), algdiv(small, large))
}

/// TOMS 708 algdiv(a, b) = ln(Gamma(b) / Gamma(a + b)) for a <= b, b >= 8, every operation a
/// double operation and ln b Excel's LN (r3-m1-residual `algdiv_v.py` with `ar="d"`).
fn algdiv(a: f64, b: f64) -> f64 {
    let h = a / b;
    let c = h / (1.0 + h);
    let x = 1.0 / (1.0 + h);
    let d = b + (a - 0.5);
    let x2 = x * x;
    // s_{2k+1} = 1 + (x + x2 s_{2k-1}), s1 = 1: s3, s5, ..., s11.
    let mut s = [0.0; 5];
    let mut previous = 1.0;
    for slot in &mut s {
        previous = 1.0 + (x + x2 * previous);
        *slot = previous;
    }
    let [s3, s5, s7, s9, s11] = s;
    let [c0, c1, c2, c3, c4, c5] = ALGDIV_C;
    let t = 1.0 / b;
    let t = t * t;
    let w = ((((c5 * s11 * t + c4 * s9) * t + c3 * s7) * t + c2 * s5) * t + c1 * s3) * t + c0;
    let w = w * (c / b);
    let u = d * alnrel(a / b);
    let v = a * (excel_ln(b) - 1.0);
    if u > v {
        (w - v) - u
    } else {
        (w - u) - v
    }
}

/// TOMS 708 alnrel(a) = ln(1 + a) for |a| <= 0.375 by its (3, 3) rational in t = a/(a + 2), in
/// doubles (here a = min/max <= 1/8).
fn alnrel(a: f64) -> f64 {
    let [p1, p2, p3] = ALNREL_P;
    let [q1, q2, q3] = ALNREL_Q;
    let t = a / (a + 2.0);
    let t2 = t * t;
    let w = (((p3 * t2 + p2) * t2 + p1) * t2 + 1.0) / (((q3 * t2 + q2) * t2 + q1) * t2 + 1.0);
    2.0 * t * w
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "test helper: a malformed fixture fails the test"
    )]

    use super::*;

    /// A shape (a, b) and its (x, Excel) rows.
    type Shape = ((f64, f64), Vec<(f64, f64)>);

    /// One set of the fixture.
    struct Set {
        name: String,
        shapes: Vec<Shape>,
    }

    fn hex(field: &str) -> f64 {
        f64::from_bits(u64::from_str_radix(field, 16).unwrap())
    }

    /// `test/fixtures/beta_small_shape.fixture`: `@ name` starts a set, `S a b` a shape, and
    /// `x excel` lines are its rows (f64 bit patterns in hex; `#` comments).
    fn fixture() -> Vec<Set> {
        let mut sets: Vec<Set> = Vec::new();
        for line in include_str!("../../test/fixtures/beta_small_shape.fixture").lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [] | ["#", ..] => {}
                ["@", name] => sets.push(Set {
                    name: (*name).to_string(),
                    shapes: Vec::new(),
                }),
                ["S", a, b] => sets
                    .last_mut()
                    .unwrap()
                    .shapes
                    .push(((hex(a), hex(b)), Vec::new())),
                [x, excel] => sets
                    .last_mut()
                    .unwrap()
                    .shapes
                    .last_mut()
                    .unwrap()
                    .1
                    .push((hex(x), hex(excel))),
                _ => panic!("malformed fixture line {line:?}"),
            }
        }
        sets
    }

    /// Within half a unit of the 15th significant digit of `expected` (the eval's
    /// `closeness::within_fifteen_digits`, decision 0083).
    fn within_fifteen_digits(expected: f64, actual: f64) -> bool {
        if !expected.is_finite() || !actual.is_finite() {
            return false;
        }
        let gap = (actual - expected).abs();
        if expected == 0.0 {
            return gap == 0.0;
        }
        let exponent: i32 = format!("{:e}", expected.abs())
            .split_once('e')
            .unwrap()
            .1
            .parse()
            .unwrap();
        gap <= format!("5e{}", exponent - 15).parse::<f64>().unwrap()
    }

    /// (rows, bit-exact, within 15 digits) of a set; Excel's 0 counts as matched by our 0 (the
    /// cell-level subnormal flush, statistics.md).
    fn score(set: &Set) -> (usize, usize, usize) {
        let (mut rows, mut exact, mut digits) = (0, 0, 0);
        for &((a, b), ref xs) in &set.shapes {
            for &(x, excel) in xs {
                let ours = small_shape_density(x, a, b).unwrap();
                let ours = if ours < f64::MIN_POSITIVE { 0.0 } else { ours };
                rows += 1;
                exact += usize::from(ours.to_bits() == excel.to_bits());
                digits += usize::from(within_fifteen_digits(excel, ours));
            }
        }
        (rows, exact, digits)
    }

    /// The task's acceptance counts (T-2503) on Excel's cells: w1's probe1 modelled shapes, the
    /// old pdf cells with min <= 1 (`beta_exact.tsv`), and the two hold-outs t2513-r3-m1-residual-hold
    /// and t2513-verify-r3-m1-residual-hold (verify-r3-m1-residual `vscore_all.out`, engine GL).
    #[test]
    fn scores_on_excels_cells() {
        let sets = fixture();
        let wanted = [
            ("probe1", 12_800, 12_796, 12_799),
            ("old", 1_107, 806, 1_103),
            // 2,810 / 4,065 now; the task's floor is the verifier's 2,804 / 4,060.
            ("hold", 4_096, 2_804, 4_060),
            ("vhold", 864, 562, 846),
        ];
        assert_eq!(sets.len(), wanted.len());
        for (set, (name, rows, exact, digits)) in sets.iter().zip(wanted) {
            assert_eq!(set.name, name);
            let got = score(set);
            assert_eq!(got.0, rows, "{name}");
            assert!(got.1 >= exact && got.2 >= digits, "{name}: {got:?}");
        }
    }

    /// Excel's BETA.DIST(0.00137,a,b,FALSE) on w1's probe1 (T-2464 w1-density/probe), one cell per
    /// lnB branch, bit-exact.
    #[test]
    fn pinned_cells() {
        let cells: [(f64, f64, f64); 8] = [
            // GAMMALN form, a < 1 and b < 1 orders.
            (0.5, 7.0, 39.290_969_766_829_214),
            (0.25, 3.3, 50.550_674_409_533_116),
            (3.0, 0.5, 1.760_800_311_602_430_5e-6),
            // -LN(the other shape): b = 1 below and above 1, a = 1 below and above 8.
            (0.5, 1.0, 13.508_580_673_957_479),
            (1.5, 1.0, 0.055_520_266_569_965_24),
            (1.0, 7.0, 6.942_656_714_880_266),
            (1.0, 20.0, 19.485_769_437_143_922),
            // TOMS algdiv.
            (0.5, 20.0, 66.001_420_671_840_05),
        ];
        for (a, b, excel) in cells {
            let ours = small_shape_density(0.001_37, a, b).unwrap();
            assert_eq!(ours.to_bits(), excel.to_bits(), "({a}, {b}): {ours}");
        }
        // a = b = 1: the uniform density, exactly 1.
        assert_eq!(small_shape_density(0.3, 1.0, 1.0), Some(1.0));
        // Outside the domain the caller keeps its other density.
        assert_eq!(small_shape_density(0.5, 2.0, 3.0), None);
        assert_eq!(small_shape_density(0.0, 0.5, 3.0), None);
        assert_eq!(small_shape_density(1.0, 0.5, 3.0), None);
        assert_eq!(small_shape_density(f64::NAN, 0.5, 3.0), None);
        assert_eq!(small_shape_density(0.5, f64::INFINITY, 0.5), None);
    }

    /// TOMS algdiv against the accurate ln(Gamma(b)/Gamma(a+b)) (60-digit mpmath): within a few
    /// ulp, the size of Excel's own algdiv errors (r3-m1-residual `algneed.py`).
    #[test]
    fn algdiv_is_near_the_accurate_value() {
        // mpmath loggamma(20) - loggamma(20.5) = -1.4916167873313041...
        let value = algdiv(0.5, 20.0);
        assert!((value + 1.491_616_787_331_304_1).abs() < 1e-14, "{value}");
    }
}
