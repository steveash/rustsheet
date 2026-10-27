//! Excel's incomplete beta kernel K(a, b, x, y): Didonato and Morris's TOMS 708 `bratio`, the 1992
//! Fortran, ported one branch at a time (SPEC-0025; docs/excel-semantics/statistics.md "Incomplete beta
//! is TOMS 708 `bratio`"). This file is the dispatch: which branch bratio takes for a point, after
//! its swap, and the branches ported so far: bpser ([`bpser`]) for w = bpser(a0, b0, x0) and
//! w1 = bpser(b0, a0, y0). The line-by-line model is `docs/research/t2464/r2-bratio/bratio_excel.py`
//! `kernel`; every branch not ported yet keeps today's kernel (`incomplete_beta.rs`, SPEC-0025's
//! migration).

use super::beta_density::algdiv;
use super::gammaln::excel_gammaln;
use super::incomplete_beta::{beta_reg_complement, closed_form_tails};
use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_exp::excel_exp_extended;
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;

/// TOMS 708's `eps` in `bratio`: max(2^-52, 1e-15).
const EPS: f64 = 1e-15;

/// The most bpser series terms [`bpser`] sums before giving the point back to today's kernel
/// (directive 5). Every bpser branch has its ratio b·x below 0.7 or x0 <= 0.7, so the terms fall
/// at least geometrically and the sum settles within a few hundred terms.
const BPSER_MAX_TERMS: u32 = 2_000;

/// bpser's ln B(a, b) is a GAMMALN sum up to a larger shape of 8 (`bratio_excel.py` `lnb_w3`):
/// below it for min(a, b) < 1 (above: algdiv), up to it for min(a, b) >= 1 (above: an accurate
/// ln B, not ported yet).
const BPSER_GAMMALN_FORM_MAX: f64 = 8.0;

/// bratio's switch to w1 = bpser(b0, a0, y0) for min(a, b) <= 1: the 1992 Fortran's 0.3, not R's 0.29
/// (probes-numeric t2464-r2-bratio-x29: Excel takes the 0.3 dispatch on 284 of the 434 cells in
/// [0.29, 0.3) where the two differ, the 0.29 one on 87).
const W1_BPSER_MIN_X: f64 = 0.3;

/// The branch bratio computes a point with (the model's names; `bratio_excel.py` `kernel`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Branch {
    /// x = 0: (0, 1).
    Zero,
    /// y = 0: (1, 0).
    One,
    /// max(a, b) < eps/1000: (b/(a+b), a/(a+b)).
    Tiny,
    Fpser,
    Apser,
    /// w = bpser(a0, b0, x0), for min(a, b) <= 1.
    WBpser,
    /// w1 = bpser(b0, a0, y0), for min(a, b) <= 1 and x0 >= 0.3.
    W1Bpser,
    /// w1 = bup(b0, a0, y0, x0, 20) then bgrat from b0 + 20.
    BupBgrat,
    /// w1 = bgrat(b0, a0, y0, x0) for b0 > 15.
    Bgrat,
    /// w = bpser(a0, b0, x0) for min(a, b) > 1 and b0·x0 <= 0.7.
    BpserLarge,
    L140BupBpser,
    L140BupBgrat,
    L140BupBupBgrat,
    Bfrac,
    Basym,
}

/// A point after bratio's swap: the oriented shapes and arguments the branch computes with, and
/// whether they are (b, a, y, x), in which case the branch's w is the caller's w1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dispatch {
    pub branch: Branch,
    pub swapped: bool,
    pub a0: f64,
    pub b0: f64,
    pub x0: f64,
    pub y0: f64,
}

/// bratio's dispatch for K(a, b, x, y), x and y formed by the caller; `None` off bratio's domain
/// (a non-positive or non-finite shape, x or y outside [0, 1], both shapes 0). The closed forms
/// (`incomplete_beta::closed_form_tails`, T-2502) are tried by the caller before this.
pub(crate) fn dispatch(a: f64, b: f64, x: f64, y: f64) -> Option<Dispatch> {
    let in_unit = |v: f64| (0.0..=1.0).contains(&v);
    if !(a.is_finite() && b.is_finite() && a >= 0.0 && b >= 0.0 && a + b > 0.0)
        || !(in_unit(x) && in_unit(y))
    {
        return None;
    }
    let plain = |branch| Dispatch {
        branch,
        swapped: false,
        a0: a,
        b0: b,
        x0: x,
        y0: y,
    };
    if x == 0.0 {
        return Some(plain(Branch::Zero));
    }
    if y == 0.0 {
        return Some(plain(Branch::One));
    }
    if a.max(b) < EPS * 0.001 {
        return Some(plain(Branch::Tiny));
    }
    if a.min(b) <= 1.0 {
        let swapped = x > 0.5;
        let (a0, b0, x0, y0) = if swapped { (b, a, y, x) } else { (a, b, x, y) };
        return Some(Dispatch {
            branch: small_shape_branch(a0, b0, x0),
            swapped,
            a0,
            b0,
            x0,
            y0,
        });
    }
    // λ as TOMS forms it, so its sign (the swap) matches bit for bit.
    let lambda = if a > b {
        (a + b) * y - b
    } else {
        a - (a + b) * x
    };
    let swapped = lambda < 0.0;
    let (a0, b0, x0, y0) = if swapped { (b, a, y, x) } else { (a, b, x, y) };
    let lambda = lambda.abs();
    let branch = if b0 < 40.0 {
        if b0 * x0 <= 0.7 {
            Branch::BpserLarge
        } else if x0 <= 0.7 {
            Branch::L140BupBpser
        } else if a0 <= 15.0 {
            Branch::L140BupBupBgrat
        } else {
            Branch::L140BupBgrat
        }
    } else if (a0 > b0 && (b0 <= 100.0 || lambda > b0 * 0.03))
        || (a0 <= b0 && (a0 <= 100.0 || lambda > a0 * 0.03))
    {
        Branch::Bfrac
    } else {
        Branch::Basym
    };
    Some(Dispatch {
        branch,
        swapped,
        a0,
        b0,
        x0,
        y0,
    })
}

/// bratio's branch for min(a, b) <= 1 on the oriented point (x0 <= 0.5).
fn small_shape_branch(a0: f64, b0: f64, x0: f64) -> Branch {
    if b0 < EPS.min(EPS * a0) {
        return Branch::Fpser;
    }
    if a0 < EPS.min(EPS * b0) && b0 * x0 <= 1.0 {
        return Branch::Apser;
    }
    if a0.max(b0) > 1.0 {
        if b0 <= 1.0 {
            return Branch::WBpser;
        }
        if x0 >= W1_BPSER_MIN_X {
            return Branch::W1Bpser;
        }
        if x0 < 0.1 && (x0 * b0).powf(a0) <= 0.7 {
            return Branch::WBpser;
        }
        if b0 > 15.0 {
            return Branch::Bgrat;
        }
    } else {
        if a0 >= 0.2_f64.min(b0) || x0.powf(a0) <= 0.9 {
            return Branch::WBpser;
        }
        if x0 >= W1_BPSER_MIN_X {
            return Branch::W1Bpser;
        }
    }
    Branch::BupBgrat
}

/// BETA.DIST's cdf, w of K(a, b, x, y) with y = fl(1 - x) formed by the caller (SPEC-0025's caller
/// table): Excel's closed forms first (T-2502), then the bpser branches of the port, and today's
/// kernel for every branch not ported yet.
pub(crate) fn beta_dist_lower_tail(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if let Some((w, _)) = closed_form_tails(x, y, a, b) {
        return w;
    }
    ported_tails(a, b, x, y).map_or_else(|| beta_reg_complement(x, y, a, b), |(w, _)| w)
}

/// (w, w1) of K(a, b, x, y) on the branches ported so far, `None` elsewhere. bratio computes one
/// tail and forms the other as (0.5 - w) + 0.5, each step an x87 op stored to a double
/// (statistics.md "Incomplete beta is TOMS 708 `bratio`").
///
/// A shape of 1 outside the closed forms (a0 = 1 with y0 > 0.9) stays on today's kernel, whose
/// x^a there is Excel's value: `BETA.DIST(0.9169139465875371,3.5,1,TRUE)` = 0.7381591796246966
/// (probes/ftest-decomposition B12) is bit-exact on it and 1 ulp high through bpser(1, 3.5, x0).
fn ported_tails(a: f64, b: f64, x: f64, y: f64) -> Option<(f64, f64)> {
    if a == 1.0 || b == 1.0 {
        return None;
    }
    let d = dispatch(a, b, x, y)?;
    let other = |v: f64| extended_sum(extended_sum(0.5, -v), 0.5);
    let (w, w1) = match d.branch {
        Branch::WBpser | Branch::BpserLarge => {
            let w = bpser(d.a0, d.b0, d.x0)?;
            (w, other(w))
        }
        Branch::W1Bpser => {
            let w1 = bpser(d.b0, d.a0, d.y0)?;
            (other(w1), w1)
        }
        _ => return None,
    };
    Some(if d.swapped { (w1, w) } else { (w, w1) })
}

/// Excel's bpser, I_x(a, b) by the power series (statistics.md "Incomplete beta bpser: a forward
/// series times EXP(fl(a LN x) - lnB)"; model `r2-bpser-small/lib.py` `bpser_w3` with GL53):
/// - s = fl(1/a) + sum c_n/(a + n), c_n = (c_{n-1}·((0.5 - b/n) + 0.5))·x, every op an x87 op
///   stored to a double, summed forward until s stops changing (no TOMS tolerance);
/// - z = fl64(fl(a·LN x) - L), never stored, and I = fl(EXP(z)·s) with Excel's EXP of the
///   extended argument (T-2645).
///
/// `None` where L is not ported (both shapes >= 1 and one above 8) or the series has not settled
/// within [`BPSER_MAX_TERMS`].
fn bpser(a: f64, b: f64, x: f64) -> Option<f64> {
    if x == 0.0 {
        return Some(0.0);
    }
    let neg_ln_beta = bpser_neg_ln_beta(a, b)?;
    let mut s = extended_quotient(1.0, a);
    let mut c = 1.0;
    let mut n = 0.0;
    let mut settled = false;
    for _ in 0..BPSER_MAX_TERMS {
        n += 1.0;
        let factor = extended_sum(extended_sum(0.5, -extended_quotient(b, n)), 0.5);
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
    let z = Register::load(extended_product(a, excel_ln(x))).add(neg_ln_beta);
    Some(extended_product(excel_exp_extended(z), s))
}

/// -L, bpser's ln B(a, b) negated, as an x87 register (`bratio_excel.py` `lnb_w3`):
/// - min(a, b) < 1: r2-bpser-small's GL53, fl(GL(a0) + fl(GL(b0) - GL(a0 + b0))) with Excel's
///   GAMMALN, smaller shape first, a0 + b0 an x87 sum, and fl(GL(a0) + algdiv(a0, b0)) from
///   b0 = 8. T-2503's density form (`beta_density::ln_beta_small_shape`) differs at its edges: it
///   takes b0 = 8 by GAMMALN, B(a0, 1) as 1/a0 and a plain a0 + b0;
/// - both >= 1 and max <= 8: GL(a) + (GL(b) - GL(fl(a + b))) in argument order, held unstored.
///
/// `None` for both shapes >= 1 with one above 8, where the model takes an accurate ln B.
fn bpser_neg_ln_beta(a: f64, b: f64) -> Option<Register> {
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
    if large > BPSER_GAMMALN_FORM_MAX {
        return None;
    }
    // -(p + q) rounds as (-p) + (-q) does: round to nearest is symmetric.
    let inner =
        Register::load(-excel_gammaln(b)).add(Register::load(excel_gammaln(extended_sum(a, b))));
    Some(Register::load(-excel_gammaln(a)).add(inner))
}

#[cfg(test)]
mod tests;
