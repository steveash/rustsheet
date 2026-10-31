//! Excel's incomplete beta kernel K(a, b, x, y): Didonato and Morris's TOMS 708 `bratio`, the 1992
//! Fortran, ported one branch at a time (SPEC-0025; docs/excel-semantics/statistics.md "Incomplete beta
//! is TOMS 708 `bratio`"). This file is the dispatch: which branch bratio takes for a point, after
//! its swap, and the branches ported so far ([`ported_tails`]): bfrac ([`bfrac`]) for b0 >= 40, bpser ([`bpser`]) for
//! w = bpser(a0, b0, x0), w1 = bpser(b0, a0, y0) and min(a, b) > 1, and L140's bup + bpser
//! ([`bup`]). The line-by-line model is `docs/research/t2464/r2-bratio/bratio_excel.py`
//! `kernel`; every branch not ported yet keeps today's kernel (`incomplete_beta.rs`, SPEC-0025's
//! migration).

mod bfrac;
mod bgrat;
pub(super) mod bpser;
mod bup;
pub(super) mod ln_beta;

use super::incomplete_beta::{beta_reg_tails, closed_form_tails};
use crate::arithmetic::extended_sum;
use bfrac::bfrac;
use bgrat::bgrat_half;
use bpser::bpser;
use bup::{bup, l140_bup_bpser};

/// TOMS 708's `eps` in `bratio`: max(2^-52, 1e-15).
pub(super) const EPS: f64 = 1e-15;

/// bratio's switch to w1 = bpser(b0, a0, y0) for min(a, b) <= 1: the 1992 Fortran's 0.3, not R's 0.29
/// (probes-numeric t2464-r2-bratio-x29: Excel takes the 0.3 dispatch on 284 of the 434 cells in
/// [0.29, 0.3) where the two differ, the 0.29 one on 87).
const W1_BPSER_MIN_X: f64 = 0.3;

/// bratio's tolerance for bfrac, 15·eps.
const BFRAC_EPS_FACTOR: f64 = 15.0;

/// The bup terms bratio takes before bgrat for b0 <= 15 (its `n = 20`).
const BUP_BGRAT_TERMS: f64 = 20.0;

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
    /// bratio's |λ| for min(a, b) > 1 (bfrac's and basym's argument), 0 elsewhere.
    pub lambda: f64,
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
        lambda: 0.0,
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
            lambda: 0.0,
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
        lambda,
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
    // The two powf here only pick a branch (against 0.7 and 0.9); bratio's own pow may differ
    // from libm's by an ulp right at those cuts.
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
/// table): [`kernel_tails`]'s w, plus bratio's bgrat branches at a0 = 1/2 (T-2509's `bgrat_half`).
/// Those are a net visible gain for BETA.DIST (T-2510: 30 fewer visibly-off cells, none more), not
/// for F (see [`t_kernel_tails`]), so F's callers (`fisher.rs`) keep [`kernel_tails`].
pub(crate) fn beta_dist_lower_tail(x: f64, y: f64, a: f64, b: f64) -> f64 {
    closed_form_tails(x, y, a, b)
        .or_else(|| ported_tails_with(a, b, x, y, true))
        .unwrap_or_else(|| beta_reg_tails(x, y, a, b))
        .0
}

/// (w, w1) of K(a, b, x, y), x and y formed by the caller per SPEC-0025's caller table:
/// [`closed_or_ported_tails`], and today's kernel (`incomplete_beta::beta_reg_tails`) for every
/// branch not ported yet.
pub(crate) fn kernel_tails(x: f64, y: f64, a: f64, b: f64) -> (f64, f64) {
    closed_or_ported_tails(x, y, a, b).unwrap_or_else(|| beta_reg_tails(x, y, a, b))
}

/// (w, w1) of K(a, b, x, y), x and y formed by the caller per SPEC-0025's caller table: Excel's
/// closed forms first (`incomplete_beta::closed_form_tails`, T-2502), then the branches of the
/// port ([`ported_tails`]). `None` where neither applies, and the caller keeps today's kernel.
pub(crate) fn closed_or_ported_tails(x: f64, y: f64, a: f64, b: f64) -> Option<(f64, f64)> {
    closed_form_tails(x, y, a, b).or_else(|| ported_tails(a, b, x, y))
}

/// (w, w1) of K(a, b, x, y) on the branches ported so far, `None` elsewhere. bratio computes one
/// tail and forms the other as (0.5 - w) + 0.5, each step an x87 op stored to a double
/// (statistics.md "Incomplete beta is TOMS 708 `bratio`").
///
/// A shape of 1 outside the closed forms (a0 = 1 with y0 > 0.9) stays on
/// `incomplete_beta::shape_one_tails`: bpser(1, b0, x0) is further off Excel's w there (T-3007).
pub(super) fn ported_tails(a: f64, b: f64, x: f64, y: f64) -> Option<(f64, f64)> {
    ported_tails_with(a, b, x, y, false)
}

/// (w, w1) of the Student t kernel call K(n/2, 1/2, y, z) ([`ported_tails`] plus bgrat at b = 1/2,
/// T-2509). Only the Student t tails and BETA.DIST ([`beta_dist_lower_tail`]) take bgrat at
/// b = 1/2 so far: F with a numerator or denominator df of 1 reaches the same branches, but there
/// bgrat moved gated FDIST cells by 7 ulp and made F.DIST sweep cells visibly off
/// (t2220-f-dist-sweep d1-1, d2-1), so F stays on today's kernel (T-2510's measurement).
pub(super) fn t_kernel_tails(a: f64, y: f64, z: f64) -> Option<(f64, f64)> {
    ported_tails_with(a, 0.5, y, z, true)
}

/// [`ported_tails`], with bratio's bgrat branches at a0 = 1/2 when `half_bgrat`.
// TODO(T-2510): drop `half_bgrat` once bgrat is ported for every b and F takes it too.
fn ported_tails_with(a: f64, b: f64, x: f64, y: f64, half_bgrat: bool) -> Option<(f64, f64)> {
    if a == 1.0 || b == 1.0 {
        return None;
    }
    let d = dispatch(a, b, x, y)?;
    let other = half_complement;
    let (w, w1) = match d.branch {
        Branch::WBpser | Branch::BpserLarge => {
            let w = bpser(d.a0, d.b0, d.x0)?;
            (w, other(w))
        }
        Branch::W1Bpser => {
            let w1 = bpser(d.b0, d.a0, d.y0)?;
            (other(w1), w1)
        }
        // bgrat's b is a0: only b = 1/2 is ported (T-2509; T-2510 and T-2512 own the rest).
        Branch::Bgrat if half_bgrat && d.a0 == 0.5 => {
            let w1 = bgrat_half(d.b0, d.y0, d.x0, 0.0)?;
            (other(w1), w1)
        }
        Branch::BupBgrat if half_bgrat && d.a0 == 0.5 => {
            let up = bup(d.b0, d.a0, d.y0, d.x0, BUP_BGRAT_TERMS)?;
            let w1 = bgrat_half(d.b0 + BUP_BGRAT_TERMS, d.y0, d.x0, up)?;
            (other(w1), w1)
        }
        Branch::L140BupBpser => {
            let w = l140_bup_bpser(d.a0, d.b0, d.x0, d.y0)?;
            (w, other(w))
        }
        Branch::Bfrac => {
            let w = bfrac(d.a0, d.b0, d.x0, d.y0, d.lambda, BFRAC_EPS_FACTOR * EPS)?;
            (w, other(w))
        }
        _ => return None,
    };
    Some(if d.swapped { (w1, w) } else { (w, w1) })
}

/// (0.5 - v) + 0.5, each step an x87 op stored to a double: bratio's complement and bpser's
/// series factor 1 - b/n.
pub(super) fn half_complement(v: f64) -> f64 {
    extended_sum(extended_sum(0.5, -v), 0.5)
}

#[cfg(test)]
mod tests;
