//! The regularized incomplete gamma functions behind GAMMA.DIST, GAMMA.INV, CHISQ.DIST,
//! CHISQ.DIST.RT, CHISQ.INV, CHISQ.INV.RT and POISSON.DIST's cdf (and GAMMADIST, GAMMAINV,
//! CHIDIST, CHIINV, POISSON, CHISQ.TEST), with a per-call cap of their own (SPEC-0011 security
//! rule, T-1872).
//!
//! `lower_within` and [`gamma_ur`] are statrs 0.18.0's `function::gamma::checked_gamma_lr` and
//! `checked_gamma_ur` (a power series and a continued fraction run to a relative tolerance of
//! 1E-15), and [`gamma_inverse_cdf`] is its `Gamma::inverse_cdf` (halve and double a bracket,
//! then 8 bisections and 4 Newton steps), ported line for line so that every call that ends in
//! statrs ends here with the same double. statrs' loops are open: the series needs about
//! 6.8 * sqrt(a) terms where x is near a (6.4 million at a = 1E12, 17 ms in release), never
//! converges once `a + 1 == a` (a of 2^53 or more: `GAMMA.DIST(1E16,1E16,1,TRUE)` never
//! finished), and the continued fraction never ends once its terms overflow to NaN. statrs also
//! panics (an unwrapped domain error) where `x * rate` overflows (`GAMMA.DIST(1E300,1,1E-10,TRUE)`)
//! or underflows to 0 (`GAMMA.DIST(1E-300,2,1E100,TRUE)`, which is 0 here as in Excel: statrs'
//! value at its smallest x). Otherwise the port returns NaN,
//! which its callers turn into `#NUM!` (docs/excel-semantics/statistics.md "Incomplete gamma:
//! per-call cap and lifting", a DEVIATION), where statrs panics or would not finish, and past
//! [`MAX_TERMS`] series or continued-fraction terms in one cdf or in one whole inversion.
//!
//! statrs is distributed under this notice:
//!
//! > MIT License
//! >
//! > Copyright (c) 2016 Michael Ma
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy of this software
//! > and associated documentation files (the "Software"), to deal in the Software without
//! > restriction, including without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
//! > Software is furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all copies or
//! > substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
//! > BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! > NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! > OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use statrs::distribution::{Continuous, ContinuousCDF, Gamma};
// statrs' own `ln_gamma`, not `super::special::ln_gamma`: bit-for-bit parity with statrs needs
// the same function.
use statrs::function::gamma::ln_gamma;
use statrs::prec;

/// The most series or continued-fraction terms one cdf, or one whole inversion, may take. The
/// series needs about 6.8 * sqrt(a) terms at its slowest, x = a (it is used for x <= a, and needs
/// fewer as x falls): 481,117 at a = 5E9, the largest shape CHISQ.* takes (df 1E10), so 2^19 keeps
/// every chi-squared cdf statrs gives, and every gamma or Poisson cdf with a shape up to about
/// 5.9E9 (675,286 terms at 1E10 are past it). The continued fraction (x above a) also grows with
/// sqrt(a) at x just above a: 8,839 terms at a = 1E9, 90,016 at 1E12, 402,191 at 1E14, and past
/// the cap at about 1E15 (`GAMMA.DIST(1000000000000001,1E15,1,TRUE)`). An inversion's cdfs together need about 26 * sqrt(a) terms for a
/// large shape: in 3 million samples (shapes 1E-3 to 2.4E10, rates 1E-12 to 1E12, probabilities
/// 1E-307 to 1-1E-16, T-1872) the smallest shape whose inversion statrs ends with a number and
/// that needed more than 2^19 terms was 3.5E8, so the inversions with a smaller shape are kept.
const MAX_TERMS: u32 = 1 << 19;

/// The terms a cdf or an inversion has left. Once spent, the cdf in progress is NaN.
struct Terms(u32);

impl Terms {
    /// Takes one term, or `false` when none is left.
    fn take(&mut self) -> bool {
        match self.0.checked_sub(1) {
            Some(left) => {
                self.0 = left;
                true
            }
            None => false,
        }
    }
}

const EPS: f64 = 0.000_000_000_000_001;
const BIG: f64 = 4_503_599_627_370_496.0;
const BIG_INV: f64 = 2.220_446_049_250_313e-16;
/// `ln` of the smallest normal double, below which statrs takes `exp(ax)` as 0 (statrs writes
/// both constants with more digits; these are the same doubles).
const MIN_LN: f64 = -709.782_712_893_384;

/// statrs' `checked_gamma_lr`: P(a, x) for `a, x > 0`, NaN where statrs errs (and its
/// `gamma_lr` panics) or past `terms`.
fn lower_within(a: f64, x: f64, terms: &mut Terms) -> f64 {
    if a.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if a <= 0.0 || a == f64::INFINITY || x <= 0.0 || x == f64::INFINITY {
        return f64::NAN;
    }

    if prec::almost_eq(a, 0.0, prec::DEFAULT_F64_ACC) {
        return 1.0;
    }
    if prec::almost_eq(x, 0.0, prec::DEFAULT_F64_ACC) {
        return 0.0;
    }

    let ax = a * x.ln() - x - ln_gamma(a);
    if ax < MIN_LN {
        if a < x {
            return 1.0;
        }
        return 0.0;
    }
    if x <= 1.0 || x <= a {
        let mut r2 = a;
        let mut c2 = 1.0;
        let mut ans2 = 1.0;
        loop {
            // statrs' series never ends once `r2 + 1 == r2` (a of 2^53 or more).
            if !terms.take() {
                return f64::NAN;
            }
            r2 += 1.0;
            c2 *= x / r2;
            ans2 += c2;

            if c2 / ans2 <= EPS {
                break;
            }
        }
        return ax.exp() * ans2 / a;
    }

    let mut y = 1.0 - a;
    let mut z = x + y + 1.0;
    let mut c = 0.0;

    let mut p3 = 1.0;
    let mut q3 = x;
    let mut p2 = x + 1.0;
    let mut q2 = z * x;
    let mut ans = p2 / q2;

    loop {
        // statrs' fraction never ends once its terms are NaN.
        if !terms.take() {
            return f64::NAN;
        }
        y += 1.0;
        z += 2.0;
        // statrs counts in an i32 converted with `f64::from`: the same doubles up to the cap.
        c += 1.0;
        let yc = y * c;

        let p = p2 * z - p3 * yc;
        let q = q2 * z - q3 * yc;

        p3 = p2;
        p2 = p;
        q3 = q2;
        q2 = q;

        if p.abs() > BIG {
            p3 *= BIG_INV;
            p2 *= BIG_INV;
            q3 *= BIG_INV;
            q2 *= BIG_INV;
        }

        if q != 0.0 {
            let nextans = p / q;
            let error = ((ans - nextans) / nextans).abs();
            ans = nextans;

            if error <= EPS {
                break;
            }
        }
    }
    1.0 - ax.exp() * ans
}

/// statrs' `checked_gamma_ur`: Q(a, x) for `a, x > 0`, NaN where statrs errs (and its
/// `gamma_ur` panics) or past `terms`.
fn upper_within(a: f64, x: f64, terms: &mut Terms) -> f64 {
    if a.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    if a <= 0.0 || a == f64::INFINITY || x <= 0.0 || x == f64::INFINITY {
        return f64::NAN;
    }

    if x < 1.0 || x <= a {
        return 1.0 - lower_within(a, x, terms);
    }

    let mut ax = a * x.ln() - x - ln_gamma(a);
    if ax < MIN_LN {
        return if a < x { 0.0 } else { 1.0 };
    }

    ax = ax.exp();
    let mut y = 1.0 - a;
    let mut z = x + y + 1.0;
    let mut c = 0.0;
    let mut pkm2 = 1.0;
    let mut qkm2 = x;
    let mut pkm1 = x + 1.0;
    let mut qkm1 = z * x;
    let mut ans = pkm1 / qkm1;
    loop {
        if !terms.take() {
            return f64::NAN;
        }
        y += 1.0;
        z += 2.0;
        c += 1.0;
        let yc = y * c;
        let pk = pkm1 * z - pkm2 * yc;
        let qk = qkm1 * z - qkm2 * yc;

        pkm2 = pkm1;
        pkm1 = pk;
        qkm2 = qkm1;
        qkm1 = qk;

        if pk.abs() > BIG {
            pkm2 *= BIG_INV;
            pkm1 *= BIG_INV;
            qkm2 *= BIG_INV;
            qkm1 *= BIG_INV;
        }

        if qk != 0.0 {
            let r = pk / qk;
            let t = ((ans - r) / r).abs();
            ans = r;

            if t <= EPS {
                break;
            }
        }
    }
    ans * ax
}

/// The regularized upper incomplete gamma Q(a, x) (POISSON's cdf is Q(k + 1, mean)): statrs'
/// value, or NaN where statrs panics or would not finish, in at most [`MAX_TERMS`] terms.
pub(crate) fn gamma_ur(a: f64, x: f64) -> f64 {
    upper_within(a, x, &mut Terms(MAX_TERMS))
}

/// statrs' `Gamma::cdf` within `terms`: its closed branches are statrs' own.
fn cdf_within(dist: &Gamma, x: f64, terms: &mut Terms) -> f64 {
    if x <= 0.0 || dist.rate().is_infinite() || x.is_infinite() {
        return dist.cdf(x);
    }
    let scaled = x * dist.rate();
    if scaled == 0.0 {
        return lower_of_underflow(dist.shape());
    }
    lower_within(dist.shape(), scaled, terms)
}

/// P(a, x) where `x / beta` underflows to 0 for an x above 0: statrs panics there (a domain
/// error), so this is statrs' value at the smallest x it takes, its `x < 1.1E-15` branch: 0, or 1
/// for a shape that small too. Excel's `GAMMA.DIST(1E-300,2,1E100,TRUE)` is 0
/// (probes/incomplete-gamma-cap E7).
fn lower_of_underflow(a: f64) -> f64 {
    if prec::almost_eq(a, 0.0, prec::DEFAULT_F64_ACC) {
        1.0
    } else {
        0.0
    }
}

/// statrs' `Gamma::cdf` (and `ChiSquared::cdf`, a gamma of shape df/2 and rate 1/2), or NaN
/// where statrs panics or would not finish, in at most [`MAX_TERMS`] terms.
pub(crate) fn gamma_cdf(dist: &Gamma, x: f64) -> f64 {
    cdf_within(dist, x, &mut Terms(MAX_TERMS))
}

/// statrs' `Gamma::sf` (and `ChiSquared::sf`), likewise capped.
pub(crate) fn gamma_sf(dist: &Gamma, x: f64) -> f64 {
    if x <= 0.0 || dist.rate().is_infinite() || x.is_infinite() {
        return dist.sf(x);
    }
    let scaled = x * dist.rate();
    if scaled == 0.0 {
        return 1.0 - lower_of_underflow(dist.shape());
    }
    gamma_ur(dist.shape(), scaled)
}

/// statrs' `Gamma::inverse_cdf` (and `ChiSquared::inverse_cdf`) for `p` in [0, 1]: the quantile,
/// or NaN where statrs panics, would not finish, or its cdfs together pass [`MAX_TERMS`] terms.
/// It evaluates at most about 2,120 cdfs (about 1,075 halvings, 1,025 doublings, 8 bisections,
/// 4 Newton steps), all of them sharing the one budget.
pub(crate) fn gamma_inverse_cdf(dist: &Gamma, p: f64) -> f64 {
    inverse_cdf_within(dist, p, MAX_TERMS)
}

fn inverse_cdf_within(dist: &Gamma, p: f64, max_terms: u32) -> f64 {
    // statrs panics outside [0, 1]; its callers never pass such a p.
    if !(0.0..=1.0).contains(&p) {
        return f64::NAN;
    }
    if p == 0.0 {
        return 0.0;
    }
    if p == 1.0 {
        return f64::INFINITY;
    }
    let mut terms = Terms(max_terms);
    // A NaN cdf is where statrs panicked or the budget ran out: the search ends there.
    let mut cdf = |x: f64| -> Option<f64> {
        let value = cdf_within(dist, x, &mut terms);
        (!value.is_nan()).then_some(value)
    };

    let mut high = 2.0;
    let mut low = 1.0;
    // Each loop ends within about 1,075 steps: `low` reaches 0, where the cdf is 0, and `high`
    // infinity, where it is 1.
    loop {
        let Some(at_low) = cdf(low) else {
            return f64::NAN;
        };
        if at_low <= p {
            break;
        }
        low /= 2.0;
    }
    loop {
        let Some(at_high) = cdf(high) else {
            return f64::NAN;
        };
        if at_high >= p {
            break;
        }
        high *= 2.0;
    }
    let mut x_0 = (high + low) / 2.0;

    for _ in 0..8 {
        let Some(at_x) = cdf(x_0) else {
            return f64::NAN;
        };
        if at_x >= p {
            high = x_0;
        } else {
            low = x_0;
        }
        if prec::convergence(&mut x_0, (high + low) / 2.0) {
            break;
        }
    }

    for _ in 0..4 {
        let Some(at_x) = cdf(x_0) else {
            return f64::NAN;
        };
        let x_next = x_0 - (at_x - p) / dist.pdf(x_0);
        if prec::convergence(&mut x_0, x_next) {
            break;
        }
    }

    x_0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::{
        gamma_cdf, gamma_inverse_cdf, gamma_sf, gamma_ur, inverse_cdf_within, lower_within,
        upper_within, Terms, MAX_TERMS,
    };
    use statrs::distribution::{ChiSquared, ContinuousCDF, DiscreteCDF, Gamma, Poisson};
    use statrs::function::gamma::{gamma_lr as statrs_lr, gamma_ur as statrs_ur};

    /// A small xorshift, so the sweep is the same on every run.
    struct Sweep(u64);
    impl Sweep {
        fn unit(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// The same double, or both NaN: where statrs' search ends in NaN (a Newton step divided by a
    /// zero density) it evaluates a cdf at NaN, and the port stops there with its own NaN.
    fn same(ours: f64, theirs: f64) -> bool {
        ours.to_bits() == theirs.to_bits() || (ours.is_nan() && theirs.is_nan())
    }

    // Where statrs finishes, the port gives its double, bit for bit: shapes 1E-3 to 1E8, x from
    // 1E-3 to 1E3 times the shape, and the quantiles GAMMA.INV and CHISQ.INV hand it.
    #[test]
    fn the_port_is_statrs_where_statrs_finishes() {
        let mut sweep = Sweep(0x1872);
        for _ in 0..20_000 {
            let a = 10f64.powf(sweep.unit() * 11.0 - 3.0);
            let x = a * 10f64.powf(sweep.unit() * 6.0 - 3.0);
            let near = a * (1.0 + (sweep.unit() - 0.5) * 8.0 / a.sqrt());
            for x in [x, near.max(1e-300)] {
                let mut terms = Terms(MAX_TERMS);
                assert_eq!(
                    lower_within(a, x, &mut terms).to_bits(),
                    statrs_lr(a, x).to_bits(),
                    "P({a},{x})"
                );
                assert_eq!(
                    gamma_ur(a, x).to_bits(),
                    statrs_ur(a, x).to_bits(),
                    "Q({a},{x})"
                );
            }
        }
        for _ in 0..3_000 {
            let shape = 10f64.powf(sweep.unit() * 9.0 - 3.0);
            let rate = 10f64.powf(sweep.unit() * 12.0 - 6.0);
            let p = if sweep.unit() < 0.3 {
                10f64.powf(-sweep.unit() * 300.0)
            } else {
                sweep.unit()
            };
            let dist = Gamma::new(shape, rate).unwrap();
            let x = shape / rate * (0.5 + sweep.unit());
            assert_eq!(gamma_cdf(&dist, x).to_bits(), dist.cdf(x).to_bits());
            assert_eq!(gamma_sf(&dist, x).to_bits(), dist.sf(x).to_bits());
            assert!(
                same(gamma_inverse_cdf(&dist, p), dist.inverse_cdf(p)),
                "Gamma({shape},{rate}) at {p}"
            );
        }
        for df in [1.0, 2.0, 5.0, 10.0, 99.0, 1e4, 1e6, 1e10] {
            let chi = ChiSquared::new(df).unwrap();
            let dist = Gamma::new(df / 2.0, 0.5).unwrap();
            for p in [1e-10, 0.05, 0.4, 0.5, 0.95, 0.999_999_999_9] {
                assert!(
                    same(gamma_inverse_cdf(&dist, p), chi.inverse_cdf(p)),
                    "ChiSquared({df}) at {p}"
                );
                let x = df * (0.5 + p);
                assert_eq!(gamma_cdf(&dist, x).to_bits(), chi.cdf(x).to_bits());
                assert_eq!(gamma_sf(&dist, x).to_bits(), chi.sf(x).to_bits());
            }
        }
        for (k, lambda) in [(5u64, 1.0), (0, 0.5), (100, 98.5), (1_000_000, 999_000.0)] {
            let poisson = Poisson::new(lambda).unwrap();
            assert_eq!(
                gamma_ur(k as f64 + 1.0, lambda).to_bits(),
                poisson.cdf(k).to_bits()
            );
        }
    }

    // The arguments where statrs panics or never finishes end, as NaN (`#NUM!` in the functions).
    #[test]
    fn statrs_panics_and_hangs_end_as_nan() {
        // statrs: `a + 1 == a`, the series never ends (GAMMA.DIST(1E16,1E16,1,TRUE)).
        assert!(gamma_cdf(&Gamma::new(1e16, 1.0).unwrap(), 1e16).is_nan());
        assert!(gamma_ur(1e16 + 1.0, 1e16).is_nan());
        // statrs: x * rate overflows, an unwrapped domain error.
        assert!(gamma_cdf(&Gamma::new(1.0, 1e10).unwrap(), 1e300).is_nan());
        // Where it underflows to 0 (statrs panicked too) the value is statrs' at its smallest x.
        assert_eq!(gamma_cdf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 0.0);
        assert_eq!(gamma_sf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 1.0);
        assert_eq!(gamma_cdf(&Gamma::new(1e-16, 1e-100).unwrap(), 1e-300), 1.0);
        // A shape of 1E300 and a mean of 1: the inversion's cdfs never finish.
        assert!(gamma_inverse_cdf(&Gamma::new(1e300, 1e300).unwrap(), 0.5).is_nan());
        // The ends and invalid arguments are statrs' (its panics outside [0, 1] are NaN).
        let dist = Gamma::new(2.0, 1.0).unwrap();
        assert_eq!(gamma_inverse_cdf(&dist, 0.0), 0.0);
        assert_eq!(gamma_inverse_cdf(&dist, 1.0), f64::INFINITY);
        assert!(gamma_inverse_cdf(&dist, 1.5).is_nan());
        assert_eq!(gamma_cdf(&dist, 0.0), 0.0);
        assert_eq!(gamma_sf(&dist, 0.0), 1.0);
        assert!(gamma_ur(0.0, 1.0).is_nan());
        assert!(gamma_ur(1.0, f64::NAN).is_nan());
    }

    // The cap branches, both sides: a cdf or an inversion that needs more terms than its budget is
    // NaN, and the same call with exactly the terms it needs is statrs' value.
    #[test]
    fn a_call_past_its_cap_is_nan() {
        // The series at a = x = 5E9 (CHISQ.DIST(1E10,1E10,TRUE)) takes 481,117 terms.
        let (a, x) = (5e9, 5e9);
        let full = statrs_lr(a, x);
        assert_eq!(
            lower_within(a, x, &mut Terms(481_117)).to_bits(),
            full.to_bits()
        );
        assert!(lower_within(a, x, &mut Terms(481_116)).is_nan());
        // At a = x = 1E10 it needs 675,286, past the cap.
        assert!(lower_within(1e10, 1e10, &mut Terms(MAX_TERMS)).is_nan());
        assert_eq!(
            lower_within(1e10, 1e10, &mut Terms(675_286)).to_bits(),
            statrs_lr(1e10, 1e10).to_bits()
        );
        // The continued fraction (x above a) and Q's own fraction.
        let (a, x) = (1e8, 1.0001e8);
        let full = statrs_lr(a, x);
        let mut terms = Terms(MAX_TERMS);
        assert_eq!(lower_within(a, x, &mut terms).to_bits(), full.to_bits());
        let used = MAX_TERMS - terms.0;
        assert_eq!(
            lower_within(a, x, &mut Terms(used)).to_bits(),
            full.to_bits()
        );
        assert!(lower_within(a, x, &mut Terms(used - 1)).is_nan());
        let full = statrs_ur(a, x);
        let mut terms = Terms(MAX_TERMS);
        assert_eq!(upper_within(a, x, &mut terms).to_bits(), full.to_bits());
        let used = MAX_TERMS - terms.0;
        assert!(upper_within(a, x, &mut Terms(used - 1)).is_nan());
        // An inversion shares one budget over its cdfs.
        let dist = Gamma::new(7.943_282_347_242_821e8, 1e6).unwrap();
        let full = dist.inverse_cdf(1e-5);
        assert_eq!(
            inverse_cdf_within(&dist, 1e-5, 476_534).to_bits(),
            full.to_bits()
        );
        assert!(inverse_cdf_within(&dist, 1e-5, 476_533).is_nan());
    }
}
