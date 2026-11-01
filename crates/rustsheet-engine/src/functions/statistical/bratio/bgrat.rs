//! TOMS 708 `bgrat` at b = 1/2 as Excel evaluates it (docs/excel-semantics/statistics.md "Student t
//! tails are one kernel call, K(n/2, 1/2, y, z), with bgrat at b = 1/2 in extended precision";
//! T-2464 r2-tdist, model `docs/research/t2464/r2-tdist/xlk.py` `bgrat_half_ld`).
//!
//! Excel's bgrat carries everything in more than double precision except three stored doubles:
//! - lnx: Excel's LN(x) for y > 0.01, else Excel's log1p(-y) (the TOMS switch is at 0.375);
//! - z = fl(-ν·lnx) with ν = a - 1/4, an x87 product stored to a double;
//! - the incomplete gamma ratio Q(1/2, z) that bgrat starts from: Excel's ERFC.PRECISE of fl(sqrt(z))
//!   (0.5 + (0.5 - ERF) below z = 1/4).
//!
//! The rest (the prefactor's algdiv, ν^(1/2), z^(1/2) e^-z and the asymptotic sum) is evaluated here
//! in double-double, which is closer to r2-tdist's exact identity than the x87-extended form
//! (1,560 against 1,545 of 1,564 bit-exact on its T cells), and the result is rounded once.

use super::super::beta_density::ALGDIV_C;
use super::super::erf::{erf, erfc};
use super::super::half_integer_beta::FRAC_1_SQRT_PI;
use super::super::incomplete_gamma::ln_gamma_1p_small;
use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_exp, dd_mul, dd_sqrt, two_sum};
use crate::x87_log10::{excel_ln, ln_double_double};
use crate::x87_log1p::excel_log1p;
use crate::x87_register::Register;

/// A double-double: an unevaluated sum `hi + lo`.
type Dd = (f64, f64);

/// Excel's switch from LN(x) to log1p(-y) for bgrat's ln x: y at or below it takes the series
/// (r2-tdist probe p1 part B: every cell with y <= 0.01, the double 0.01 included, fits log1p).
/// FITTED(T-2464, 2000 cells, n 100..3000, z straddling 0.01)
const LOG1P_MAX_Y: f64 = 0.01;

/// TOMS bgrat's series terms: at most 30.
const MAX_TERMS: usize = 30;

/// TOMS bgrat's stop rule, |term| <= 15 eps (sum + l), with bratio's eps = 1e-15.
const STOP_TOLERANCE: f64 = 15e-15;

/// Past this z the prefactor e^-z is past [`dd_exp`]'s double-double range (its tail word turns
/// subnormal); such points, tails below about 1E-282, keep today's kernel.
const MAX_Z: f64 = 650.0;

/// The smallest z [`upper_gamma_fraction`] takes: Legendre's fraction converges slowly below
/// about 1 (every L140 bup+bgrat point in the probes has z >= 1.1: RESULTS.md finding 3).
const MIN_CF_Z: f64 = 1.0;

/// The most terms [`upper_gamma_fraction`] takes (about 350 reach [`CF_TOLERANCE`] at z = 1).
const MAX_CF_TERMS: u32 = 2000;

/// [`upper_gamma_fraction`]'s stop rule on |delta - 1|: past double-double's working precision of
/// the final rounding.
const CF_TOLERANCE: f64 = 1e-30;

/// TOMS 708 `bgrat(a, 1/2, x, y, w)`: w + `I_x(a, 1/2)` for a large a (bratio calls it for a > 15),
/// x + y = 1, as Excel computes it (module docs). `None` where the prefactor leaves the double
/// range (TOMS's `ierr` cases), so the caller keeps today's kernel.
pub(super) fn bgrat_half(a: f64, x: f64, y: f64, w: f64) -> Option<f64> {
    let total = bgrat_dd(a, 0.5, x, y, w, |z| {
        let root = z.sqrt();
        let q = if root * root >= 0.25 {
            erfc(root)
        } else {
            0.5 + (0.5 - erf(root))
        };
        // At b = 1/2, r = b z^b e^-z / Gamma(b + 1) is z^(1/2) e^-z / sqrt(pi).
        let r = dd_mul(dd_mul(dd_sqrt((z, 0.0)), dd_exp((-z, 0.0))), FRAC_1_SQRT_PI);
        Some((r, dd_div((q, 0.0), r)))
    })?;
    Some(total.0)
}

/// TOMS 708 `bgrat(a, b, x, y, 0)` = `I_x(a, b)` for 0 < b < 1 and a large a, as an accurate
/// double-double: bratio's stored lnx and z (module docs), and the rest, the incomplete gamma ratio
/// `J_0 = Q(b, z)/r` included, in double-double, not TOMS's grat1, whose continued fraction stops
/// tens of ulps short at TOMS's 15·eps (docs/research/t2464/r3-bgrat/RESULTS.md finding 3: Excel's
/// increment is within 4 ulp of the accurate one on 1,598 of 2,908 cells against 142 for TOMS in
/// doubles). `None` where z is below the continued fraction's range or the prefactor leaves the
/// double range.
pub(super) fn bgrat_increment(a: f64, b: f64, x: f64, y: f64) -> Option<Dd> {
    if !(b > 0.0 && b < 1.0) {
        return None;
    }
    bgrat_dd(a, b, x, y, 0.0, |z| {
        if z < MIN_CF_Z {
            return None;
        }
        // r = b z^b e^-z / Gamma(b + 1) = exp(b ln z - z - ln Gamma(b)).
        let ln_gamma = if b < 0.5 {
            dd_add((ln_gamma_1p_small(b), 0.0), negate(ln_double_double(b)))
        } else {
            (ln_gamma_1p_small(b - 1.0), 0.0)
        };
        let exponent = dd_add(
            dd_add(dd_mul((b, 0.0), ln_double_double(z)), (-z, 0.0)),
            negate(ln_gamma),
        );
        Some((dd_exp(exponent), upper_gamma_fraction(b, z)?))
    })
}

/// Q(b, z)·Gamma(b)·e^z·z^-b for z >= [`MIN_CF_Z`], by Legendre's continued fraction
/// 1/(z + 1 - b - 1(1 - b)/(z + 3 - b - 2(2 - b)/(z + 5 - b - ...))) in double-double (modified
/// Lentz); `None` if it has not converged in [`MAX_CF_TERMS`] terms.
fn upper_gamma_fraction(b: f64, z: f64) -> Option<Dd> {
    let tiny = (1e-300, 0.0);
    let mut f = dd_add((z + 1.0, 0.0), (-b, 0.0));
    let mut c = f;
    let mut d = (0.0, 0.0);
    for n in 1..=MAX_CF_TERMS {
        let n = f64::from(n);
        let an = dd_mul((-n, 0.0), dd_add((n, 0.0), (-b, 0.0)));
        let bn = dd_add((z, 0.0), (2.0 * n + 1.0 - b, 0.0));
        d = dd_add(bn, dd_mul(an, d));
        if d.0 == 0.0 {
            d = tiny;
        }
        c = dd_add(bn, dd_div(an, c));
        if c.0 == 0.0 {
            c = tiny;
        }
        d = dd_div((1.0, 0.0), d);
        let delta = dd_mul(c, d);
        f = dd_mul(f, delta);
        if (delta.0 - 1.0 + delta.1).abs() <= CF_TOLERANCE {
            return Some(dd_div((1.0, 0.0), f));
        }
    }
    None
}

/// -v for a double-double v.
fn negate(v: Dd) -> Dd {
    (-v.0, -v.1)
}

/// bgrat in double-double: `w + u·(J_0 + sum d_n J_n)`, with lnx and z stored as Excel stores them
/// (module docs); `prefactor(z)` gives r = b z^b e^-z / Gamma(b + 1) and `J_0` = Q(b, z)/r.
fn bgrat_dd(
    a: f64,
    b: f64,
    x: f64,
    y: f64,
    w: f64,
    prefactor: impl FnOnce(f64) -> Option<(Dd, Dd)>,
) -> Option<Dd> {
    let lnx = if y > LOG1P_MAX_Y {
        excel_ln(x)
    } else {
        excel_log1p(-y)
    };
    // TOMS's bm1 = b - 0.5 - 0.5 and nu = a + 0.5·bm1.
    let half_bm1 = 0.5 * (b - 0.5 - 0.5);
    let nu = two_sum(a, half_bm1);
    let z = Register::load(a)
        .add(Register::load(half_bm1))
        .mul(Register::load(-lnx))
        .store();
    if !(z > 0.0 && z <= MAX_Z) {
        return None;
    }
    let (r, j0) = prefactor(z)?;
    // u = r e^-(algdiv(b, a) + b ln nu).
    let ln_nu = dd_add(ln_double_double(nu.0), (nu.1 / nu.0, 0.0));
    let exponent = dd_add(algdiv_small(b, a), dd_mul((b, 0.0), ln_nu));
    let u = dd_mul(r, dd_exp(negate(exponent)));
    if u.0 == 0.0 || !u.0.is_finite() {
        return None;
    }
    let l = if w == 0.0 {
        (0.0, 0.0)
    } else {
        dd_div((w, 0.0), u)
    };
    let sum = asymptotic_sum(b, j0, z, lnx, nu, l);
    Some(dd_add((w, 0.0), dd_mul(u, sum)))
}

/// bgrat's sum `J_0 + sum d_n J_n` (TOMS's `j`, `c`, `d` recurrences) from `J_0` = Q(b, z)/r,
/// stopping at TOMS's rule against `sum + l`.
fn asymptotic_sum(b: f64, j0: Dd, z: f64, lnx: f64, nu: Dd, l: Dd) -> Dd {
    let bm1 = b - 0.5 - 0.5;
    let v = dd_div((0.25, 0.0), dd_mul(nu, nu));
    let t2 = dd_mul((0.25 * lnx, 0.0), (lnx, 0.0));
    let mut c = [(0.0, 0.0); MAX_TERMS];
    let mut d = [(0.0, 0.0); MAX_TERMS];
    let (mut j, mut sum) = (j0, j0);
    let (mut t, mut cn) = ((1.0, 0.0), (1.0, 0.0));
    let mut n2 = 0.0;
    for n in 1..=MAX_TERMS {
        let bp2n = b + n2;
        let jt = dd_mul(dd_add((z, 0.0), (bp2n + 1.0, 0.0)), t);
        j = dd_mul(dd_add(dd_mul((bp2n * (bp2n + 1.0), 0.0), j), jt), v);
        n2 += 2.0;
        t = dd_mul(t, t2);
        cn = dd_div_small(cn, n2 * (n2 + 1.0));
        c[n - 1] = cn;
        let mut acc = (0.0, 0.0);
        let mut coefficient = b - n as f64;
        for i in 1..n {
            acc = dd_add(
                acc,
                dd_mul((coefficient, 0.0), dd_mul(c[i - 1], d[n - 1 - i])),
            );
            coefficient += b;
        }
        let dn = dd_add(dd_mul((bm1, 0.0), cn), dd_div_small(acc, n as f64));
        d[n - 1] = dn;
        let dj = dd_mul(dn, j);
        sum = dd_add(sum, dj);
        if sum.0 <= 0.0 || dj.0.abs() <= STOP_TOLERANCE * (sum.0 + l.0) {
            break;
        }
    }
    sum
}

/// TOMS 708 `algdiv(b, a)` = ln(Gamma(a)/Gamma(a + b)) for b <= a, a >= 8, by its Stirling form,
/// in double-double.
fn algdiv_small(b: f64, a: f64) -> Dd {
    let [c0, c1, c2, c3, c4, c5] = ALGDIV_C;
    let h = dd_div((b, 0.0), (a, 0.0));
    let h1 = dd_add(h, (1.0, 0.0));
    let c = dd_div(h, h1);
    let x = dd_div((1.0, 0.0), h1);
    let x2 = dd_mul(x, x);
    // s_n = (1 - x^n) / (1 - x) for n = 3, 5, ..., 11, by TOMS's recurrence.
    let next = |s: Dd| dd_add(dd_add(x, dd_mul(x2, s)), (1.0, 0.0));
    let s3 = next((1.0, 0.0));
    let s5 = next(s3);
    let s7 = next(s5);
    let s9 = next(s7);
    let s11 = next(s9);
    let inverse = dd_div((1.0, 0.0), (a, 0.0));
    let t = dd_mul(inverse, inverse);
    let mut w = dd_mul(dd_mul((c5, 0.0), s11), t);
    for (coefficient, s) in [(c4, s9), (c3, s7), (c2, s5), (c1, s3)] {
        w = dd_mul(dd_add(w, dd_mul((coefficient, 0.0), s)), t);
    }
    w = dd_mul(dd_add(w, (c0, 0.0)), dd_mul(c, inverse));
    // u = d ln(1 + h) with d = a + (b - 1/2); v = b (ln a - 1).
    let u = dd_mul(two_sum(a, b - 0.5), ln_one_plus(h));
    let ln_a = ln_double_double(a);
    let v = dd_add(dd_mul((b, 0.0), ln_a), (-b, 0.0));
    let uv = dd_add(u, v);
    dd_add(w, (-uv.0, -uv.1))
}

/// ln(1 + h) in double-double for a small positive double-double h: ln of the double 1 + h plus
/// the first-order term of the rest.
fn ln_one_plus(h: Dd) -> Dd {
    let (head, error) = two_sum(1.0, h.0);
    let rest = error + h.1;
    dd_add(ln_double_double(head), (rest / head, 0.0))
}
