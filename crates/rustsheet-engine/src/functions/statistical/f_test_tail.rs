//! F.TEST's tail for both shapes above 1 as TOMS 708 `bratio` with Excel's pieces (T-2681): the
//! branches of `bratio` for min(a, b) > 1 and a smaller-side shape b0 below 40 (after bratio's
//! swap on lambda), as T-2464 r2-bratio's integrated model has them (`BEST` in
//! `docs/research/t2464/r2-bratio/bratio_excel.py`; "w3" and "w4" name that study's workers):
//! - b0 x0 <= 0.7: w3's `bpser`, a series summed in stored x87 ops times
//!   EXP(fl64(fl(a0 LN x0) - ln B(a0, b0))) ([`bpser`]);
//! - otherwise, while x0 <= 0.7 (TOMS's label L140): `bup` from the shape b0 - n (1/2 or 1) up by n
//!   with Excel's beta density times x0 y0 as its prefactor (BETA.DIST's
//!   [`beta_density_small_shape`]) and w4's stored ratios ([`bup`]), plus [`bpser`] at that shape.
//!
//! Over the 291 F.TEST cells of `uv run rustsheet-fitlab f-test` the 113 with both shapes above 1
//! go from 31 to 76 bit-exact and from 7 visibly off to 1 against the remainder model of
//! `fisher.rs` (`f_test_upper_tail`), which keeps every other shape: with a shape of 1/2 bratio's
//! `bpser` is worse than that model (28 bit-exact of 116 against 75, 23 visibly off against 0;
//! docs/research/t2681/README.md, docs/excel-semantics/statistics.md "F.TEST / FTEST").
//!
//! Every loop is bounded (bratio's series bound, and `bup`'s n below 40).

use super::beta_density::beta_density_small_shape;
use super::bratio::{bpser_with_ln_beta, EPS};
use super::gammaln::excel_gammaln;
use crate::arithmetic::{extended_quotient, extended_sum};
use crate::x87_exp::{dd_add, dd_to_extended};
use crate::x87_log10::{excel_ln, ln_double_double};
use crate::x87_register::Register;

/// The shape on bratio's smaller side from which it leaves `bpser` and `bup` (TOMS `b0 < 40`).
const BRATIO_SERIES_SHAPE_MAX: f64 = 40.0;
/// TOMS's `bpser` and L140 cut on b0 x0 and on x0.
const BPSER_CUT: f64 = 0.7;
/// TOMS `gamln1(1/2)`, ln Gamma(3/2), as TOMS's rational approximation gives it in doubles: w3's
/// ln Gamma(1/2) is fl64(gamln1(1/2) - LN(1/2)) (`lnb_w3`, `w3_lg = "toms"`); it enters at the
/// L140 shape 1/2 (gen-m9/58e3253bbd3a7a82 A141 and 8f2ac25945af87da A119, bit-exact with it).
const GAMLN1_HALF: f64 = -0.120_782_237_635_245_15;

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
        bpser(a0, b0, x0)?
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
        bup(shape, a0, y0, x0, n)? + bpser(a0, shape, x0)?
    };
    // bratio's other tail, and the swap back.
    let w1 = (0.5 - w) + 0.5;
    Some(2.0 * w.min(w1))
}

/// w3's `bpser`: bratio's series and EXP ([`bpser_with_ln_beta`]) with L [`neg_ln_beta`]'s
/// ln B(a, b).
fn bpser(a: f64, b: f64, x: f64) -> Option<f64> {
    bpser_with_ln_beta(a, b, x, neg_ln_beta(a, b))
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

/// TOMS `bup` for `I_x(a, b) - I_x(a + n, b)` as w4 found Excel's: the prefactor is Excel's beta
/// density times x y ([`beta_density_small_shape`], a of 1/2 or 1), over a, and each term
/// d = d ((a+b+i)/(a+1+i)) x keeps the ratio stored and ratio times x in a register. `None` where
/// that density is not finite.
fn bup(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let prefactor = beta_density_small_shape(x, a, b)? * (x * y) / a;
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

#[cfg(test)]
mod tests {
    use super::super::bratio::bpser_neg_ln_beta;
    use super::{f_test_bratio_p, neg_ln_beta};
    use crate::test::util::ulps;
    use crate::x87_register::Register;

    // (x, y, a, b, Excel's F.TEST, the port's ulps from it) at the F.TEST cells the remainder
    // model had visibly off (excel-graph; T-2681): none is visibly off now.
    #[test]
    fn excel_cells_both_shapes_above_one() {
        let cells = [
            // gen-m9/58e3253bbd3a7a82 Calc!A141 (L140 at the shape 1/2)
            (
                0.677_419_354_838_714_3,
                0.322_580_645_161_285_7,
                7.5,
                2.5,
                0.540_798_569_556_812_7,
                0,
            ),
            // gen-m9/8f2ac25945af87da Calc!A119 (L140)
            (
                0.506_329_113_924_051_8,
                0.493_670_886_075_948_1,
                6.0,
                4.5,
                0.657_910_021_992_518,
                0,
            ),
            // gen-m9/a7bf0d8d81d33ba6 Calc!A81 (L140 at the shape 1)
            (
                0.514_705_892_361_292_6,
                0.485_294_107_638_707_5,
                5.0,
                3.0,
                0.502_782_788_443_525_9,
                0,
            ),
            // gen-m9/fec39efaba9284c3 Calc!A36 (L140, the accurate ln B above 8)
            (
                0.531_249_979_746_619_9,
                0.468_750_020_253_38,
                8.5,
                2.5,
                0.080_701_192_695_023_8,
                1,
            ),
            // gen-m9/caa1eed48b1883fe Calc!A380 (bpser)
            (
                0.075_604_065_115_223_84,
                0.924_395_934_884_776_1,
                2.5,
                6.0,
                0.084_052_537_265_491_03,
                3,
            ),
            // gen-m9/f34f66fb7f81e673 Calc!A4 (L140)
            (
                0.256_880_727_398_555_1,
                0.743_119_272_601_445,
                4.0,
                10.0,
                0.877_645_926_559_382_3,
                3,
            ),
            // gen-m9/bcb24385baafb2f0 Calc!A377 (bpser deep in the tail)
            (
                8.676_177_114_371_624e-28,
                1.0,
                3.5,
                7.5,
                6.414_538_185_730_402_6e-93,
                3,
            ),
        ];
        for (x, y, a, b, excel, off) in cells {
            let p = f_test_bratio_p(x, y, a, b).unwrap_or(f64::NAN);
            assert_eq!(ulps(p, excel), off, "I_{x}({a}, {b}): {p} against {excel}");
        }
    }

    // Above a shape of 8, -ln B is the accurate value rounded to a register: B(9, 1/2) and
    // B(1/2, 11.5) against 50-digit mpmath (-log(beta(9, 0.5)), -log(beta(0.5, 11.5))).
    #[test]
    fn neg_ln_beta_above_eight_is_accurate() {
        for (a, b, reference) in [
            (9.0, 0.5, 0.512_365_575_127_318_2),
            (0.5, 11.5, 0.637_942_426_381_944_4),
        ] {
            assert_eq!(neg_ln_beta(a, b).store(), reference, "B({a}, {b})");
        }
    }

    // F.TEST's ln B is not BETA.DIST's bpser ln B (`bratio::bpser_neg_ln_beta`), so bpser takes it
    // as an argument: at the L140 shape 1/2 it reads GAMMALN(1/2) as gamln1(1/2) - LN(1/2), an ulp
    // apart once stored (A141 and A119 are bit-exact with it), and above a shape of 8 it has the
    // accurate value where BETA.DIST's port has none yet.
    #[test]
    fn ln_beta_differs_from_beta_dist_bpser() {
        let beta_dist = |a, b| bpser_neg_ln_beta(a, b).map(Register::store);
        for (a, b, f_test, beta_dist_value) in [
            (7.5, 0.5, 0.418_432_181_381_979_2, 0.418_432_181_381_979_24),
            (6.0, 0.5, 0.302_705_374_150_394_95, 0.302_705_374_150_395),
        ] {
            assert_eq!(neg_ln_beta(a, b).store(), f_test, "B({a}, {b})");
            assert_eq!(beta_dist(a, b), Some(beta_dist_value), "B({a}, {b})");
        }
        assert_eq!(beta_dist(9.0, 2.5), None);
    }

    // Not ported: a shape of 1/2 or 1 (the remainder model), a smaller-side shape of 40 or more
    // (bfrac and basym), and L140 with x0 > 0.7 (bgrat).
    #[test]
    fn outside_the_port() {
        assert_eq!(f_test_bratio_p(0.3, 0.7, 0.5, 4.5), None);
        assert_eq!(f_test_bratio_p(0.3, 0.7, 1.0, 4.5), None);
        assert_eq!(f_test_bratio_p(0.5, 0.5, 45.0, 45.0), None);
        assert_eq!(f_test_bratio_p(0.8, 0.2, 30.0, 2.0), None);
    }
}
