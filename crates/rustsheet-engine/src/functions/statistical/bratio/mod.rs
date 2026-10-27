//! Excel's incomplete beta kernel K(a, b, x, y): Didonato and Morris's TOMS 708 `bratio`, the 1992
//! Fortran, ported one branch at a time (SPEC-0025; docs/excel-semantics/statistics.md "Incomplete beta
//! is TOMS 708 `bratio`"). This file is the dispatch: which branch bratio takes for a point, after
//! its swap. The line-by-line model is `docs/research/t2464/r2-bratio/bratio_excel.py` `kernel`.

/// TOMS 708's `eps` in `bratio`: max(2^-52, 1e-15).
const EPS: f64 = 1e-15;

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

#[cfg(test)]
mod tests;
