//! Double-double pieces of the incomplete beta at half-integer shapes (the F distribution's df/2):
//! `B(a, b)` by its exact recurrence and the remainder series `I_x(a,b)/(x^a (1-x)^b)`, shared by
//! F.TEST's tail (`fisher.rs`), the accurate F tails (`incomplete_beta.rs`, T-2219) and bratio's
//! accurate ln B (`bratio/ln_beta.rs`).

use crate::x87_exp::{dd_add, dd_div, dd_div_small, dd_mul};

/// Bound on the terms of [`half_integer_remainder`]'s series: it converges well before it while
/// x stays below the mean and a + b is at most F.TEST's 200 (about 30,000 terms at x = 0.9975).
const SERIES_TERMS: u32 = 100_000;

/// pi as a double-double (hi + lo).
const PI_DD: (f64, f64) = (std::f64::consts::PI, 1.224_646_799_147_353_2e-16);

/// Whether `v` is a half-integer shape of at least 1/2 (1/2, 1, 3/2, ...): df/2 for a whole df.
pub(super) fn is_half_integer_shape(v: f64) -> bool {
    v >= 0.5 && (2.0 * v).fract() == 0.0
}

/// `I_x(a,b)/(x^a (1-x)^b) = 2F1(a+b, 1; a+1; x) / (a B(a,b))` for half-integer shapes, in
/// double-double: `B(a,b)` by [`half_integer_beta`] and the series of positive terms
/// `t_(n+1) = t_n (a+b+n) x/(a+1+n)`, which nothing cancels.
pub(super) fn half_integer_remainder(x: (f64, f64), a: f64, b: f64) -> f64 {
    let beta = half_integer_beta(a, b);
    let mut term = (1.0, 0.0);
    let mut sum = (1.0, 0.0);
    let mut n = 0.0;
    for _ in 0..SERIES_TERMS {
        term = dd_div_small(dd_mul(dd_mul(term, (a + b + n, 0.0)), x), a + 1.0 + n);
        sum = dd_add(sum, term);
        n += 1.0;
        // 2^-110 of the sum: past double-double precision.
        if term.0 < sum.0 * 7.7e-34 {
            break;
        }
    }
    dd_div(sum, dd_mul(beta, (a, 0.0))).0
}

/// `B(a,b)` in double-double for half-integer shapes a, b >= 1/2: `B(a+1,b) = B(a,b) a/(a+b)` and
/// `B(a,b+1) = B(a,b) b/(a+b)` from `B(1,1) = 1`, `B(1/2,1) = B(1,1/2) = 2` or `B(1/2,1/2) = pi`.
pub(super) fn half_integer_beta(a: f64, b: f64) -> (f64, f64) {
    let a0 = if a.fract() == 0.0 { 1.0 } else { 0.5 };
    let b0 = if b.fract() == 0.0 { 1.0 } else { 0.5 };
    let mut beta = match (a0 == 1.0, b0 == 1.0) {
        (true, true) => (1.0, 0.0),
        (true, false) | (false, true) => (2.0, 0.0),
        (false, false) => PI_DD,
    };
    let (mut p, mut q) = (a0, b0);
    while p < a {
        beta = dd_div_small(dd_mul(beta, (p, 0.0)), p + q);
        p += 1.0;
    }
    while q < b {
        beta = dd_div_small(dd_mul(beta, (q, 0.0)), p + q);
        q += 1.0;
    }
    beta
}
