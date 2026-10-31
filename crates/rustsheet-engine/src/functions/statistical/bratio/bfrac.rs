//! TOMS 708 `bfrac`, the continued fraction for `I_x(a, b)` with a, b > 1 and b0 >= 40 (SPEC-0025's
//! `bfrac.rs`), with Excel's beta density times x·y as its prefactor.

use super::super::loader::excel_beta_density;

/// The most terms [`bfrac`] takes before giving the point back to today's kernel (TOMS's own
/// bound, directive 5).
const BFRAC_MAX_TERMS: f64 = 1_000.0;

/// TOMS 708 `bfrac(a, b, x, y, λ, eps)` as Excel computes it (`bratio_excel.py` `bfrac`, BEST:
/// `brcomp = "pdf"`, `bgrat = "orig"`; statistics.md "Incomplete beta L140 and bfrac: the prefactor
/// is Excel's own density times x·y"):
/// - brcomp = fl(P·fl(x·y)) with P Excel's BETA.DIST density at x (T-2507's Loader form);
/// - the fraction is the 1992 Fortran's, w = n·(b - n)·x and β = n + w/s + e·(c + n·(y + 1)), in
///   doubles (its own arithmetic is not found, T-2512).
///
/// λ is bratio's |λ| after its swap. `None` where the density is not formed or the fraction has
/// not settled within [`BFRAC_MAX_TERMS`] terms.
pub(super) fn bfrac(a: f64, b: f64, x: f64, y: f64, lambda: f64, eps: f64) -> Option<f64> {
    let brc = excel_beta_density(x, a, b)? * (x * y);
    if brc == 0.0 {
        return Some(0.0);
    }
    let c = lambda + 1.0;
    let c0 = b / a;
    let c1 = 1.0 / a + 1.0;
    let yp1 = y + 1.0;
    let (mut n, mut p, mut s) = (0.0, 1.0, a + 1.0);
    let (mut an, mut bn, mut anp1) = (0.0, 1.0, 1.0);
    let mut bnp1 = c / c1;
    let mut r = c1 / c;
    loop {
        if n >= BFRAC_MAX_TERMS {
            return None;
        }
        n += 1.0;
        let t = n / a;
        let w = n * (b - n) * x;
        let e = a / s;
        let alpha = p * (p + c0) * e * e * (w * x);
        let e = (t + 1.0) / (c1 + t + t);
        let beta = n + w / s + e * (c + n * yp1);
        p = t + 1.0;
        s += 2.0;
        let next_a = alpha * an + beta * anp1;
        an = anp1;
        anp1 = next_a;
        let next_b = alpha * bn + beta * bnp1;
        bn = bnp1;
        bnp1 = next_b;
        let r0 = r;
        r = anp1 / bnp1;
        if (r - r0).abs() <= eps * r {
            break;
        }
        an /= bnp1;
        bn /= bnp1;
        anp1 = r;
        bnp1 = 1.0;
    }
    let w = brc * r;
    w.is_finite().then_some(w)
}
