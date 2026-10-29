//! TOMS 708 `bup` and bratio's L140 for x0 <= 0.7 (SPEC-0025's `bup.rs`): w = bup(b0', a0, y0, x0, n) +
//! bpser(a0, b0', x0).

use super::super::beta_density::beta_density_small_shape;
use super::bpser::bpser;
use super::EPS;
use crate::arithmetic::{extended_quotient, extended_sum};
use crate::x87_register::Register;

/// bratio's L140 for x0 <= 0.7 (a, b > 1, b0 < 40, b0·x0 > 0.7): with n = int(b0) and
/// b0' = b0 - n (b0' = 1 and n - 1 at an integer b0), w = bup(b0', a0, y0, x0, n) + bpser(a0, b0', x0)
/// (statistics.md "Incomplete beta L140 and bfrac: the prefactor is Excel's own density times x·y";
/// model `w4-fractions/tomsx.py` `l140_best`, `bratio_excel.py` `bup` with `bup_arith = "w4"` and
/// `brcmp1 = "pdf"`).
pub(super) fn l140_bup_bpser(a0: f64, b0: f64, x0: f64, y0: f64) -> Option<f64> {
    // b0 < 40, so n is a small whole number of bup terms.
    let mut n = b0.trunc();
    let mut b0_frac = b0 - n;
    if b0_frac == 0.0 {
        n -= 1.0;
        b0_frac = 1.0;
    }
    let up = bup(b0_frac, a0, y0, x0, n)?;
    let series = bpser(a0, b0_frac, x0)?;
    Some(up + series)
}

/// TOMS 708 `bup(a, b, x, y, n)` = `I_x(a, b) - I_x(a + n, b)` for a whole n >= 1 and a <= 1, as
/// Excel computes it (`bratio_excel.py` `bup` and `_bup_term`, `bup_arith = "w4"`):
/// - the prefactor is Excel's own beta density P of Beta(a, b) at x (T-2503, a <= 1) times x·y, so
///   bup = fl(fl(P·fl(x·y))/a)·W, with no exp(mu) scaling;
/// - W = 1 + `d_1` + ... with `d_l = d_{l-1}·(r_l·x)`, `r_l = fl(fl(apb + l)/fl(ap1 + l))` stored and `r_l·x`
///   held in a register; each partial sum stored;
/// - the sum stops after term k (TOMS's (b - 1)x/y - a) once d <= eps·W.
fn bup(a: f64, b: f64, x: f64, y: f64, n: f64) -> Option<f64> {
    let density = beta_density_small_shape(x, a, b)?;
    let ret = density * (x * y) / a;
    if n == 1.0 || ret == 0.0 {
        return Some(ret);
    }
    let nm1 = n - 1.0;
    let mut k = 0.0;
    if b > 1.0 {
        if y > 1e-4 {
            let r = (b - 1.0) * x / y - a;
            if r >= 1.0 {
                k = if r < nm1 { r.trunc() } else { nm1 };
            }
        } else {
            k = nm1;
        }
    }
    let (apb, ap1) = (a + b, a + 1.0);
    let x_reg = Register::load(x);
    let mut d = 1.0;
    let mut w = 1.0;
    let mut l = 0.0;
    while l < nm1 {
        let ratio = extended_quotient(apb + l, ap1 + l);
        d = Register::load(d)
            .mul(Register::load(ratio).mul(x_reg))
            .store();
        w = extended_sum(w, d);
        if l >= k && d <= EPS * w {
            break;
        }
        l += 1.0;
    }
    Some(ret * w)
}
