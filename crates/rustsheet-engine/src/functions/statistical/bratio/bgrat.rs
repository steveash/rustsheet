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

/// TOMS 708 `bgrat(a, 1/2, x, y, w)`: w + `I_x(a, 1/2)` for a large a (bratio calls it for a > 15),
/// x + y = 1, as Excel computes it (module docs). `None` where the prefactor leaves the double
/// range (TOMS's `ierr` cases), so the caller keeps today's kernel.
pub(super) fn bgrat_half(a: f64, x: f64, y: f64, w: f64) -> Option<f64> {
    let lnx = if y > LOG1P_MAX_Y {
        excel_ln(x)
    } else {
        excel_log1p(-y)
    };
    let nu = two_sum(a, -0.25);
    let z = Register::load(a)
        .add(Register::load(-0.25))
        .mul(Register::load(-lnx))
        .store();
    if !(z > 0.0 && z <= MAX_Z) {
        return None;
    }
    let root = z.sqrt();
    let q = if root * root >= 0.25 {
        erfc(root)
    } else {
        0.5 + (0.5 - erf(root))
    };
    // r = b z^b e^-z / Gamma(b + 1) and u = r e^-(algdiv(b, a) + b ln nu), at b = 1/2.
    // bgrat's r = b z^b e^-z / Gamma(b + 1) is z^(1/2) e^-z / sqrt(pi) at b = 1/2.
    let r = dd_mul(dd_mul(dd_sqrt((z, 0.0)), dd_exp((-z, 0.0))), FRAC_1_SQRT_PI);
    let ln_nu = dd_add(ln_double_double(nu.0), (nu.1 / nu.0, 0.0));
    let exponent = dd_add(algdiv_half(a), (0.5 * ln_nu.0, 0.5 * ln_nu.1));
    let u = dd_mul(r, dd_exp((-exponent.0, -exponent.1)));
    if u.0 == 0.0 || !u.0.is_finite() {
        return None;
    }
    let l = if w == 0.0 {
        (0.0, 0.0)
    } else {
        dd_div((w, 0.0), u)
    };
    let sum = asymptotic_sum(dd_div((q, 0.0), r), z, lnx, nu, l);
    let total = dd_add((w, 0.0), dd_mul(u, sum));
    Some(total.0)
}

/// bgrat's sum `J_0 + sum d_n J_n` (TOMS's `j`, `c`, `d` recurrences at b = 1/2) from `J_0` =
/// Q(1/2, z)/r, stopping at TOMS's rule against `sum + l`.
fn asymptotic_sum(j0: Dd, z: f64, lnx: f64, nu: Dd, l: Dd) -> Dd {
    const B: f64 = 0.5;
    let v = dd_div((0.25, 0.0), dd_mul(nu, nu));
    let t2 = dd_mul((0.25 * lnx, 0.0), (lnx, 0.0));
    let mut c = [(0.0, 0.0); MAX_TERMS];
    let mut d = [(0.0, 0.0); MAX_TERMS];
    let (mut j, mut sum) = (j0, j0);
    let (mut t, mut cn) = ((1.0, 0.0), (1.0, 0.0));
    let mut n2 = 0.0;
    for n in 1..=MAX_TERMS {
        let bp2n = B + n2;
        let jt = dd_mul(dd_add((z, 0.0), (bp2n + 1.0, 0.0)), t);
        j = dd_mul(dd_add(dd_mul((bp2n * (bp2n + 1.0), 0.0), j), jt), v);
        n2 += 2.0;
        t = dd_mul(t, t2);
        cn = dd_div_small(cn, n2 * (n2 + 1.0));
        c[n - 1] = cn;
        let mut acc = (0.0, 0.0);
        let mut coefficient = B - n as f64;
        for i in 1..n {
            acc = dd_add(
                acc,
                dd_mul((coefficient, 0.0), dd_mul(c[i - 1], d[n - 1 - i])),
            );
            coefficient += B;
        }
        let dn = dd_add((-0.5 * cn.0, -0.5 * cn.1), dd_div_small(acc, n as f64));
        d[n - 1] = dn;
        let dj = dd_mul(dn, j);
        sum = dd_add(sum, dj);
        if sum.0 <= 0.0 || dj.0.abs() <= STOP_TOLERANCE * (sum.0 + l.0) {
            break;
        }
    }
    sum
}

/// TOMS 708 `algdiv(1/2, a)` = ln(Gamma(a)/Gamma(a + 1/2)) for a >= 8 by its Stirling form,
/// in double-double.
fn algdiv_half(a: f64) -> Dd {
    let [c0, c1, c2, c3, c4, c5] = ALGDIV_C;
    let h = dd_div((0.5, 0.0), (a, 0.0));
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
    // u = d ln(1 + h) with d = a + (1/2 - 1/2) = a; v = (1/2)(ln a - 1).
    let u = dd_mul((a, 0.0), ln_one_plus(h));
    let ln_a = ln_double_double(a);
    let v = dd_add((0.5 * ln_a.0, 0.5 * ln_a.1), (-0.5, 0.0));
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
