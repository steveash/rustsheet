//! F.TEST's tail for both shapes above 1 as TOMS 708 `bratio` with Excel's pieces (T-2681): the
//! branches of `bratio` for min(a, b) > 1 and a smaller-side shape below 40 (after its swap on
//! lambda), as T-2464 r2-bratio's integrated model (`docs/research/t2464/r2-bratio/bratio_excel.py`,
//! `Cfg()`) has them:
//! - b0 x0 <= 0.7: `bpser` as w3 found it, a series summed in stored x87 ops times
//!   EXP(fl64(fl(a0 LN x0) - ln B(a0, b0))) ([`bpser`]);
//! - otherwise, while x0 <= 0.7 (TOMS L140): `bup` from the shape b0 - n (1/2 or 1) up by n with
//!   Excel's beta density times x0 y0 as its prefactor and w4's stored ratios ([`bup`]), plus
//!   [`bpser`] at that shape.
//!
//! Over the 291 F.TEST cells of the fit (docs/excel-semantics/statistics.md "F.TEST / FTEST: larger
//! variance on top") the 104 such cells go from 31 to 66 bit-exact and from 7 visibly off to 1 against
//! the remainder model of `fisher.rs` (`f_test_upper_tail`), which keeps every other shape: for
//! min(a, b) <= 1 bratio's `bpser` there is worse than that model (28 bit-exact against 75, 23 cells
//! visibly off against 0). The cells whose density needs TOMS `gsumln` (a shape 1 against one of
//! at most 2) keep the model too: they are not ported.
//!
//! Every loop is bounded ([`SERIES_TERMS`], and `bup`'s n below 40).

use super::gammaln::excel_gammaln;
use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_exp::{dd_add, dd_to_extended, excel_exp_extended, two_sum};
use crate::x87_log10::{excel_ln, ln_double_double};
use crate::x87_register::Register;

/// The shape on bratio's smaller side from which it leaves `bpser` and `bup` (TOMS `b0 < 40`).
const BRATIO_SERIES_SHAPE_MAX: f64 = 40.0;
/// TOMS's `bpser` and L140 cut on b0 x0 and on x0.
const BPSER_CUT: f64 = 0.7;
/// bratio's `eps` (TOMS: max(2^-52, 1e-15)), `bup`'s relative stop.
const EPS: f64 = 1e-15;
/// Bound on [`bpser`]'s terms: x0 <= 0.7 there and the terms shrink like x0^n, so it stops after
/// a few hundred.
const SERIES_TERMS: u32 = 100_000;
/// TOMS `gamln1(1/2)`, ln Gamma(3/2), as TOMS's rational approximation gives it in doubles: w3's
/// ln Gamma(1/2) is fl64(gamln1(1/2) - LN(1/2)) (`lnb_w3`, `w3_lg = "toms"`).
const GAMLN1_HALF: f64 = -0.120_782_237_635_245_15;
/// The Stirling coefficients of TOMS's `algdiv` and `bcorr` (`del`).
const STIRLING_DEL: [f64; 6] = [
    0.083_333_333_333_333_3,
    -0.002_777_777_777_609_91,
    7.936_506_668_253_9e-4,
    -5.952_029_313_518_7e-4,
    8.373_080_340_312_15e-4,
    -0.001_653_229_627_807_13,
];

/// F.TEST's p, `2 min(I_x(a,b), 1 - I_x(a,b))`, for half-integer shapes a, b > 1 with y = 1 - x,
/// or `None` where this port does not reach and the caller keeps its model.
pub(super) fn f_test_bratio_p(x: f64, y: f64, a: f64, b: f64) -> Option<f64> {
    if !(a > 1.0 && b > 1.0 && x > 0.0 && y > 0.0) {
        return None;
    }
    let lambda = if a > b {
        (a + b) * y - b
    } else {
        a - (a + b) * x
    };
    let (a0, b0, x0, y0) = if lambda < 0.0 {
        (b, a, y, x)
    } else {
        (a, b, x, y)
    };
    if b0 >= BRATIO_SERIES_SHAPE_MAX {
        return None;
    }
    let w = if b0 * x0 <= BPSER_CUT {
        bpser(a0, b0, x0)
    } else {
        if x0 > BPSER_CUT {
            return None;
        }
        // TOMS L140: the integer part n of b0 goes to bup, leaving a shape of 1/2 or 1.
        let mut n = b0.trunc();
        let mut shape = b0 - n;
        if shape == 0.0 {
            n -= 1.0;
            shape = 1.0;
        }
        bup(shape, a0, y0, x0, n)? + bpser(a0, shape, x0)
    };
    // bratio's other tail, and the swap back.
    let w1 = (0.5 - w) + 0.5;
    Some(2.0 * w.min(w1))
}

/// w3's `bpser`: I_x(a, b) as s = 1/a + sum c_n/(a+n), c_n = c_(n-1) ((1/2 - b/n) + 1/2) x, every
/// step a stored x87 op, summed until s stops changing, times EXP(fl64(fl(a LN x) - L)) with L
/// [`neg_ln_beta`]'s ln B(a, b).
fn bpser(a: f64, b: f64, x: f64) -> f64 {
    let mut s = extended_quotient(1.0, a);
    let mut c = 1.0;
    let mut n = 0.0;
    for _ in 0..SERIES_TERMS {
        n += 1.0;
        let factor = extended_sum(extended_sum(0.5, -extended_quotient(b, n)), 0.5);
        c = extended_product(extended_product(c, factor), x);
        let next = extended_sum(s, extended_quotient(c, extended_sum(a, n)));
        if next == s {
            break;
        }
        s = next;
    }
    let z = Register::load(extended_product(a, excel_ln(x))).add(neg_ln_beta(a, b));
    extended_product(excel_exp_extended(z), s)
}

/// -ln B(a, b) in a register as w3's `lnb_w3` forms it: above a shape of 8 the accurate value
/// (w3's proxy for `algdiv`), else fl64(lg(a) + fl64(lg(b) - lg(a+b))) with lg Excel's GAMMALN
/// from 1 and fl64(gamln1(1/2) - LN(1/2)) at 1/2 (the only half-integer below 1).
fn neg_ln_beta(a: f64, b: f64) -> Register {
    if a.max(b) > 8.0 {
        let (hi, lo) = super::fisher::half_integer_beta(a, b);
        let ln_beta = dd_add(ln_double_double(hi), (lo / hi, 0.0));
        // ln B < 0 for both shapes above 1/2 and one above 1, so -ln B is positive.
        let (significand, exponent) = dd_to_extended((-ln_beta.0, -ln_beta.1));
        return Register::from_parts(false, significand, exponent);
    }
    let neg_lg = |v: f64| {
        if v >= 1.0 {
            Register::load(-excel_gammaln(v))
        } else {
            Register::load(-GAMLN1_HALF).add(Register::load(excel_ln(v)))
        }
    };
    neg_lg(b)
        .add(Register::load(excel_gammaln(a + b)))
        .add(neg_lg(a))
}

/// TOMS `bup` for I_x(a, b) - I_x(a + n, b) as w4 found Excel's: the prefactor is Excel's beta
/// density times x y ([`density_m1`]), over a, and each term d = d ((a+b+i)/(a+1+i)) x keeps the
/// ratio stored and ratio times x in a register. `None` where the density is not ported.
fn bup(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let prefactor = density_m1(x, a, b)? * (x * y) / a;
    if n <= 1.0 {
        return Some(prefactor);
    }
    let nm1 = n - 1.0;
    let apb = a + b;
    let ap1 = a + 1.0;
    // TOMS: no relative stop before the largest term.
    let k = if y > 1e-4 {
        let r = (b - 1.0) * x / y - a;
        if r >= 1.0 {
            r.trunc().min(nm1)
        } else {
            0.0
        }
    } else {
        nm1
    };
    let mut d = 1.0;
    let mut w = d;
    let mut i = 0.0;
    while i < nm1 {
        let ratio = extended_quotient(apb + i, ap1 + i);
        d = Register::load(d)
            .mul(Register::load(ratio).mul(Register::load(x)))
            .store();
        w = extended_sum(w, d);
        if i >= k && d <= EPS * w {
            break;
        }
        i += 1.0;
    }
    Some(prefactor * w)
}

/// Excel's beta density at x for a shape `a` of 1/2 or 1 and `b` > 1, as T-2464 w1's model M1:
/// EXP(fl64(tx + fl64(ty - ln B))) with tx = fl((a-1) LN x), ty = fl((b-1) ln(1-x)), ln(1-x) the
/// correctly rounded log1p(-x) up to x = 0.375 (w1's stand-in) and LN(fl(1-x)) above, and ln B
/// [`density_ln_beta`]'s.
fn density_m1(x: f64, a: f64, b: f64) -> Option<f64> {
    let ln_beta = density_ln_beta(a, b)?;
    let ln_y = if x <= 0.375 {
        let (hi, lo) = two_sum(1.0, -x);
        dd_add(ln_double_double(hi), (lo / hi, 0.0)).0
    } else {
        excel_ln(1.0 - x)
    };
    let tx = extended_product(extended_sum(a, -1.0), excel_ln(x));
    let ty = extended_product(extended_sum(b, -1.0), ln_y);
    let e = Register::load(ty)
        .add(Register::load(-ln_beta))
        .add(Register::load(tx));
    Some(excel_exp_extended(e))
}

/// M1's ln B(a, b) for a in {1/2, 1} and b > 1: with a = 1/2 and b <= 8 the GAMMALN form
/// fl(GL(a) + fl(GL(b) - GL(a+b))); with a = 1 and 2 < b < 8, -LN(b); otherwise TOMS `betaln`
/// with Excel's GAMMALN and LN, gamln(a) + algdiv(a, b), which it takes for b >= 8. `None` for a = 1
/// with b <= 2, where `betaln` takes `gsumln` (not ported).
fn density_ln_beta(a: f64, b: f64) -> Option<f64> {
    if a < 1.0 && b <= 8.0 {
        let tail = extended_sum(excel_gammaln(b), -excel_gammaln(a + b));
        return Some(extended_sum(excel_gammaln(a), tail));
    }
    if a == 1.0 && b > 2.0 && b < 8.0 {
        return Some(-excel_ln(b));
    }
    if b >= 8.0 {
        return Some(excel_gammaln(a) + algdiv(a, b));
    }
    None
}

/// TOMS `algdiv`: ln(Gamma(b)/Gamma(a+b)) for b >= 8 with Excel's LN, in doubles.
fn algdiv(a: f64, b: f64) -> f64 {
    let (c, x, d) = if a > b {
        let h = b / a;
        (1.0 / (h + 1.0), h / (h + 1.0), a + (b - 0.5))
    } else {
        let h = a / b;
        (h / (h + 1.0), 1.0 / (h + 1.0), b + (a - 0.5))
    };
    let w = stirling_del(c, x, b);
    let u = d * alnrel(a / b);
    let v = a * (excel_ln(b) - 1.0);
    if u > v {
        w - v - u
    } else {
        w - u - v
    }
}

/// TOMS's `del(b)` correction of `algdiv`: c/b times the series in t = (1/b)^2 with
/// s_(2k+1) = 1 + x + ... + x^(2k).
fn stirling_del(c: f64, x: f64, b: f64) -> f64 {
    let x2 = x * x;
    let mut s = [0.0; 5];
    let mut acc = 1.0;
    for slot in &mut s {
        acc = x + x2 * acc + 1.0;
        *slot = acc;
    }
    let [s3, s5, s7, s9, s11] = s;
    let t = (1.0 / b) * (1.0 / b);
    let [c0, c1, c2, c3, c4, c5] = STIRLING_DEL;
    let w = ((((c5 * s11 * t + c4 * s9) * t + c3 * s7) * t + c2 * s5) * t + c1 * s3) * t + c0;
    w * (c / b)
}

/// TOMS `alnrel`, ln(1 + a): Excel's LN above |a| = 0.375, below it TOMS's (3,3) rational in
/// t = a/(a+2).
fn alnrel(a: f64) -> f64 {
    if a.abs() > 0.375 {
        return excel_ln(1.0 + a);
    }
    let t = a / (a + 2.0);
    let t2 = t * t;
    let num = ((-0.017_887_454_601_221_4 * t2 + 0.405_303_492_862_024) * t2 - 1.294_189_230_219_93)
        * t2
        + 1.0;
    let den = ((-0.084_510_421_794_556_5 * t2 + 0.747_811_014_037_616) * t2 - 1.627_522_563_553_23)
        * t2
        + 1.0;
    t * 2.0 * (num / den)
}

#[cfg(test)]
mod tests {
    use super::f_test_bratio_p;
    use crate::test::util::ulps;

    // (x, y, a, b, Excel's F.TEST) at F.TEST cells the remainder model had visibly off
    // (excel-graph; T-2681): bit-exact, and within 3 ulp (not visibly off) for the last three.
    #[test]
    fn excel_cells_both_shapes_above_one() {
        let exact = [
            // gen-m9/58e3253bbd3a7a82 Calc!A141 (L140, the a = 1/2 GAMMALN density)
            (
                0.677_419_354_838_714_3,
                0.322_580_645_161_285_7,
                7.5,
                2.5,
                0.540_798_569_556_812_7,
            ),
            // gen-m9/8f2ac25945af87da Calc!A119 (L140)
            (
                0.506_329_113_924_051_8,
                0.493_670_886_075_948_1,
                6.0,
                4.5,
                0.657_910_021_992_518,
            ),
            // gen-m9/a7bf0d8d81d33ba6 Calc!A81 (L140, the a = 1 density, -LN(b))
            (
                0.514_705_892_361_292_6,
                0.485_294_107_638_707_5,
                5.0,
                3.0,
                0.502_782_788_443_525_9,
            ),
            // gen-m9/fec39efaba9284c3 Calc!A36 (L140, betaln through algdiv, accurate ln B)
            (
                0.531_249_979_746_619_9,
                0.468_750_020_253_38,
                8.5,
                2.5,
                0.080_701_192_695_023_8,
            ),
        ];
        for (x, y, a, b, excel) in exact {
            assert_eq!(f_test_bratio_p(x, y, a, b), Some(excel), "I_{x}({a}, {b})");
        }
        let close = [
            // gen-m9/caa1eed48b1883fe Calc!A380 (bpser)
            (
                0.075_604_065_115_223_84,
                0.924_395_934_884_776_1,
                2.5,
                6.0,
                0.084_052_537_265_491_03,
            ),
            // gen-m9/f34f66fb7f81e673 Calc!A4 (L140, swapped)
            (
                0.256_880_727_398_555_1,
                0.743_119_272_601_445,
                4.0,
                10.0,
                0.877_645_926_559_382_3,
            ),
            // gen-m9/bcb24385baafb2f0 Calc!A377 (bpser deep in the tail)
            (
                8.676_177_114_371_624e-28,
                1.0,
                3.5,
                7.5,
                6.414_538_185_730_402_6e-93,
            ),
        ];
        for (x, y, a, b, excel) in close {
            let p = f_test_bratio_p(x, y, a, b).unwrap_or(f64::NAN);
            assert!(ulps(p, excel) <= 3, "I_{x}({a}, {b}): {p} against {excel}");
        }
    }

    // Not ported: a shape of 1/2 or 1 (the remainder model), a smaller-side shape of 40 or more
    // (bfrac and basym), and L140 with x0 > 0.7 (bgrat).
    #[test]
    fn outside_the_port() {
        assert_eq!(f_test_bratio_p(0.3, 0.7, 0.5, 4.5), None);
        assert_eq!(f_test_bratio_p(0.3, 0.7, 1.0, 4.5), None);
        assert_eq!(f_test_bratio_p(0.5, 0.5, 45.0, 45.0), None);
        // a = 1 against b = 2 in L140 needs gsumln.
        assert_eq!(f_test_bratio_p(0.45, 0.55, 2.0, 2.0), None);
    }
}
