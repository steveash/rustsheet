//! Excel's bpser, TOMS 708's power series for `I_x(a, b)`, and the ln B it takes (SPEC-0025's
//! `bpser.rs`): bratio's w = bpser(a0, b0, x0), w1 = bpser(b0, a0, y0), bpser for min(a, b) > 1 and
//! L140's series term ([`super::bup`]).

use super::super::beta_density::algdiv;
use super::super::gammaln::excel_gammaln;
use super::{half_complement, ln_beta};
use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_exp::excel_exp_extended;
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;

/// The most bpser series terms [`bpser`] sums before giving the point back to today's kernel
/// (directive 5). Every bpser branch has its ratio b·x below 0.7 or x0 <= 0.7, so the terms fall
/// at least geometrically and the sum settles within a few hundred terms.
const BPSER_MAX_TERMS: u32 = 2_000;

/// bpser's ln B(a, b) is a GAMMALN sum up to a larger shape of 8 (`bratio_excel.py` `lnb_w3`):
/// below it for min(a, b) < 1 (above: algdiv), up to it for min(a, b) >= 1 (above: an accurate
/// ln B, not ported yet).
const BPSER_GAMMALN_FORM_MAX: f64 = 8.0;

/// Excel's bpser, `I_x(a, b)` by the power series (statistics.md "Incomplete beta bpser: a forward
/// series times EXP(fl(a LN x) - lnB)"; model `r2-bpser-small/lib.py` `bpser_w3` with GL53):
/// - s = fl(1/a) + sum `c_n/(a + n)`, `c_n = (c_{n-1}·((0.5 - b/n) + 0.5))·x`, every op an x87 op
///   stored to a double, summed forward until s stops changing (no TOMS tolerance);
/// - z = fl64(fl(a·LN x) - L), never stored, and I = fl(EXP(z)·s) with Excel's EXP of the
///   extended argument (T-2645).
///
/// `None` where L is not ported (both shapes >= 1 and one above 8), and where
/// [`bpser_with_ln_beta`] gives `None`.
pub(super) fn bpser(a: f64, b: f64, x: f64) -> Option<f64> {
    bpser_with_ln_beta(a, b, x, bpser_neg_ln_beta(a, b)?)
}

/// [`bpser`] with -L given by the caller: F.TEST's tail takes the series and EXP as BETA.DIST's
/// do but forms ln B its own way (`f_test_tail::neg_ln_beta`, T-2681). `None` at a shape of 0,
/// where fl(a·LN x) is not finite, or where the series has not settled within
/// [`BPSER_MAX_TERMS`].
pub(in super::super) fn bpser_with_ln_beta(
    a: f64,
    b: f64,
    x: f64,
    neg_ln_beta: Register,
) -> Option<f64> {
    if a <= 0.0 {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }
    let mut s = extended_quotient(1.0, a);
    let mut c = 1.0;
    let mut n = 0.0;
    let mut settled = false;
    for _ in 0..BPSER_MAX_TERMS {
        n += 1.0;
        let factor = half_complement(extended_quotient(b, n));
        c = extended_product(extended_product(c, factor), x);
        let next = extended_sum(s, extended_quotient(c, extended_sum(a, n)));
        if next == s {
            settled = true;
            break;
        }
        s = next;
    }
    if !settled {
        return None;
    }
    // A load reads a non-finite double as 0, so fl(a·LN x) = -inf (a shape near 1E308) would
    // drop out of z: such points keep today's kernel. Excel for the web has no value to match
    // there: it refuses a workbook holding BETA.DIST with a shape of 1E+295 or more
    // (statistics.md "Incomplete beta at a shape near the double range", T-3007).
    let a_ln_x = extended_product(a, excel_ln(x));
    if !a_ln_x.is_finite() {
        return None;
    }
    let z = Register::load(a_ln_x).add(neg_ln_beta);
    Some(extended_product(excel_exp_extended(z), s))
}

/// -L, bpser's ln B(a, b) negated, as an x87 register (`bratio_excel.py` `lnb_w3`):
/// - min(a, b) < 1: r2-bpser-small's GL53, fl(GL(a0) + fl(GL(b0) - GL(a0 + b0))) with Excel's
///   GAMMALN, smaller shape first, a0 + b0 an x87 sum, and fl(GL(a0) + algdiv(a0, b0)) from
///   b0 = 8. T-2503's density form (`beta_density::ln_beta_small_shape`) differs at its edges: it
///   takes b0 = 8 by GAMMALN, B(a0, 1) as 1/a0 and a plain a0 + b0;
/// - both >= 1 and max <= 8: GL(a) + (GL(b) - GL(fl(a + b))) in argument order, held unstored;
/// - min(a, b) = 1: -LN(max(a, b)), B(1, b) = 1/b as the density takes it (T-2506, T-2847);
/// - both > 1 and max > 8: the accurate ln B at half-integer shapes, TOMS `betaln` elsewhere
///   ([`ln_beta`], T-3007; Excel's own form there is open, T-2924).
///
/// `None` where [`ln_beta::toms_betaln`] is off its domain, which only a non-finite shape reaches
/// here.
pub(in super::super) fn bpser_neg_ln_beta(a: f64, b: f64) -> Option<Register> {
    let (small, large) = if a < b { (a, b) } else { (b, a) };
    if small < 1.0 {
        let ln_beta = if large < BPSER_GAMMALN_FORM_MAX {
            extended_sum(
                excel_gammaln(small),
                extended_sum(
                    excel_gammaln(large),
                    -excel_gammaln(extended_sum(small, large)),
                ),
            )
        } else {
            extended_sum(excel_gammaln(small), algdiv(small, large))
        };
        return Some(Register::load(-ln_beta));
    }
    if small == 1.0 {
        // B(1, b) = 1/b, so L = -LN(b), the density's form (`beta_density::ln_beta_small_shape`),
        // exact where the GAMMALN sum is not. Only L140's bpser(a0, 1, x0) at a whole b0 reaches
        // it (`ported_tails` keeps a caller's shape of 1 on today's kernel), which BINOM.DIST's
        // whole shapes always take (ironcalc-calc BINOM.xlsx E5:E8, E75 are Excel's exact value
        // here and 2 ulp off on the sum). Counts and the cells it moves: statistics.md "Incomplete
        // beta bpser" (T-2506, T-2847).
        return Some(Register::load(excel_ln(large)));
    }
    if large > BPSER_GAMMALN_FORM_MAX {
        return ln_beta::neg_ln_beta_half_integer(a, b)
            .or_else(|| Some(Register::load(-ln_beta::toms_betaln(a, b)?)));
    }
    // -(p + q) rounds as (-p) + (-q) does: round to nearest is symmetric.
    let inner =
        Register::load(-excel_gammaln(b)).add(Register::load(excel_gammaln(extended_sum(a, b))));
    Some(Register::load(-excel_gammaln(a)).add(inner))
}
