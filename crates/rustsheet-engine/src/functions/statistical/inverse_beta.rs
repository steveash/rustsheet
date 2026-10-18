//! The inverse of the regularized incomplete beta function behind the quantiles of BETA.INV,
//! F.INV and T.INV (and BETAINV, F.INV.RT, FINV, T.INV.2T, TINV, CONFIDENCE.T), with a per-call
//! cap of its own (SPEC-0011 security rule, T-1848).
//!
//! [`inv_beta_reg`] is statrs 0.18.0's `function::beta::inv_beta_reg` (Algorithm AS 64 / AS 109
//! with Remark AS R83), ported line for line so that every call that ends in statrs ends here with
//! the same double. statrs' loops are open: the outer Newton loop runs until a step no longer
//! moves, and its step-shrinking loops never end once the Newton step is not finite (a correction
//! of ±inf or NaN keeps `adj * adj < prev` false forever). Measured with statrs on the arguments
//! the functions hand it (T-1848): it panics for every F.INV below p = 1.6E-162 (`x * x`
//! underflows and its first estimate is NaN, `F.INV(1E-300,5,5)`), never finishes for about 7% of F.INV
//! calls below p = 3.6E-27 (`F.INV(1E-50,6000,4)`) and for
//! `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)`, and a call that finishes takes at
//! most 377 Newton steps and 1,189 step adjustments in 13 million samples (F: degrees of freedom 1
//! to 99,999 and p from 1E-320; Beta: shapes 1E-5 to 1E5 and p from 1E-15; Student t: 27 steps at
//! most). The port returns NaN, which its callers turn into `#NUM!` (docs/excel-semantics/
//! statistics.md "Quantile searches: per-call cap and lifting", a DEVIATION), where statrs panics
//! or would not finish and past [`MAX_NEWTON_STEPS`] ([`T_MAX_NEWTON_STEPS`] for Student's t) or
//! [`MAX_STEP_ADJUSTMENTS`].
//!
//! statrs' code carries this notice, from the `special` crate it is based on:
//!
//! > Copyright 2014–2019 The special Developers
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy of this software
//! > and associated documentation files (the “Software”), to deal in the Software without
//! > restriction, including without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
//! > Software is furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all copies or
//! > substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
//! > BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! > NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! > OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//!
//! statrs itself is Copyright (c) 2016 Michael Ma, under the MIT License (the same terms).

use statrs::function::beta::{checked_beta_reg, checked_ln_beta};

/// The most Newton steps one inversion for BETA.INV or F.INV takes. The steps grow with -log10 of
/// the probability, about 2.3 a decade, and stop at 377 where `x * x` underflows (p near 1E-162,
/// below which the first estimate is NaN), so 512 keeps every result statrs reaches.
const MAX_NEWTON_STEPS: u32 = 512;

/// The most Newton steps one inversion for Student's t takes: its probabilities are 2E-8 or more
/// (`t_dist.rs` keeps statrs to `T_STATRS_MIN_P`), where 27 was the most seen in 3 million samples
/// (degrees of freedom 1 to 99,999), so 64 keeps every result statrs reaches there and lets the
/// lifting table charge T.INV an eighth of what BETA.INV is charged.
const T_MAX_NEWTON_STEPS: u32 = 64;

/// The most step adjustments (each a division of the step by 3) one inversion makes over all its
/// Newton steps; 1,189 was the most seen. A finite step needs at most about 680 in one Newton step
/// (the factor underflows to 0, and a zero step is always accepted).
const MAX_STEP_ADJUSTMENTS: u32 = 4096;

/// The `x` with `I_x(a, b) = p` for `a, b > 0` and `p` in [0, 1], or NaN where statrs' search
/// panics, would not finish, or passes the caps. At most [`MAX_NEWTON_STEPS`] incomplete-beta
/// evaluations (each a continued fraction of at most 140 steps) and [`MAX_STEP_ADJUSTMENTS`]
/// adjustments, whatever the arguments.
pub(crate) fn inv_beta_reg(a: f64, b: f64, x: f64) -> f64 {
    inv_beta_reg_within(a, b, x, MAX_NEWTON_STEPS)
}

/// [`inv_beta_reg`] with at most `max_steps` Newton steps.
fn inv_beta_reg_within(mut a: f64, mut b: f64, mut x: f64, max_steps: u32) -> f64 {
    const SAE: i32 = -30;
    const FPU: f64 = 1e-30; // 10^SAE

    let Ok(ln_beta) = checked_ln_beta(a, b) else {
        return f64::NAN;
    };
    if !(0.0..=1.0).contains(&x) {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }

    let mut p;
    let mut q;

    let flip = 0.5 < x;
    if flip {
        p = a;
        a = b;
        b = p;
        x = 1.0 - x;
    }

    p = (-(x * x).ln()).sqrt();
    q = p - (2.30753 + 0.27061 * p) / (1.0 + (0.99229 + 0.04481 * p) * p);

    if 1.0 < a && 1.0 < b {
        let r = (q * q - 3.0) / 6.0;
        let s = 1.0 / (2.0 * a - 1.0);
        let t = 1.0 / (2.0 * b - 1.0);
        let h = 2.0 / (s + t);
        let w = q * (h + r).sqrt() / h - (t - s) * (r + 5.0 / 6.0 - 2.0 / (3.0 * h));
        p = a / (a + b * (2.0 * w).exp());
    } else {
        let mut t = 1.0 / (9.0 * b);
        t = 2.0 * b * (1.0 - t + q * t.sqrt()).powf(3.0);
        if t <= 0.0 {
            p = 1.0 - ((((1.0 - x) * b).ln() + ln_beta) / b).exp();
        } else {
            t = 2.0 * (2.0 * a + b - 1.0) / t;
            if t <= 1.0 {
                p = (((x * a).ln() + ln_beta) / a).exp();
            } else {
                p = 1.0 - 2.0 / (t + 1.0);
            }
        }
    }

    p = p.clamp(0.0001, 0.9999);
    // A NaN first estimate (x * x underflows to 0) is where statrs' incomplete beta panics.
    if p.is_nan() {
        return f64::NAN;
    }

    // `e` is at least about -1E7 for x >= 1E-35, and saturates (to i32::MIN) below, where the
    // comparison with SAE picks FPU as in statrs.
    let e = (-5.0 / a / a - 1.0 / x.powf(0.2) - 13.0) as i32;
    let acu = if e > SAE { f64::powi(10.0, e) } else { FPU };

    let mut pnext;
    let mut qprev = 0.0;
    let mut sq = 1.0;
    let mut prev = 1.0;
    let mut steps = 0;
    let mut adjustments = 0;

    'outer: loop {
        steps += 1;
        if steps > max_steps {
            return f64::NAN;
        }
        let Ok(cdf) = checked_beta_reg(a, b, p) else {
            return f64::NAN;
        };
        q = (cdf - x) * (ln_beta + (1.0 - a) * p.ln() + (1.0 - b) * (1.0 - p).ln()).exp();
        // statrs' adjustment loop below never ends for a step that is not finite.
        if !q.is_finite() {
            return f64::NAN;
        }

        if q * qprev <= 0.0 {
            prev = if sq > FPU { sq } else { FPU };
        }

        let mut g = 1.0;
        loop {
            loop {
                adjustments += 1;
                if adjustments > MAX_STEP_ADJUSTMENTS {
                    return f64::NAN;
                }
                let adj = g * q;
                sq = adj * adj;

                if sq < prev {
                    pnext = p - adj;
                    if (0.0..=1.0).contains(&pnext) {
                        break;
                    }
                }
                g /= 3.0;
            }

            if prev <= acu || q * q <= acu {
                p = pnext;
                break 'outer;
            }

            if pnext != 0.0 && pnext != 1.0 {
                break;
            }

            g /= 3.0;
        }

        if pnext == p {
            break;
        }

        p = pnext;
        qprev = q;
    }

    if flip {
        1.0 - p
    } else {
        p
    }
}

/// statrs' `Beta::inverse_cdf` with [`inv_beta_reg`]'s caps: the quantile of `p` in [0, 1].
pub(crate) fn beta_inverse_cdf(alpha: f64, beta: f64, p: f64) -> f64 {
    inv_beta_reg(alpha, beta, p)
}

/// statrs' `FisherSnedecor::inverse_cdf` with [`inv_beta_reg`]'s caps: the quantile of `p` in
/// [0, 1] of the F distribution.
pub(crate) fn f_inverse_cdf(df1: f64, df2: f64, p: f64) -> f64 {
    let z = inv_beta_reg(df1 / 2.0, df2 / 2.0, p);
    df2 / (df1 * (1.0 / z - 1.0))
}

/// statrs' `StudentsT::inverse_cdf` (location 0, scale 1) with [`inv_beta_reg`]'s caps, at most
/// [`T_MAX_NEWTON_STEPS`] Newton steps: the quantile of `p` in [0, 1] of Student's t.
pub(crate) fn t_inverse_cdf(df: f64, p: f64) -> f64 {
    let x1 = if p >= 0.5 { 1.0 - p } else { p };
    let mut y = inv_beta_reg_within(0.5 * df, 0.5, 2.0 * x1, T_MAX_NEWTON_STEPS);
    y = (df * (1. - y) / y).sqrt();
    y = if p >= 0.5 { y } else { -y };
    // statrs adds its location, 0, which turns a -0 into 0.
    0.0 + y
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::{beta_inverse_cdf, f_inverse_cdf, inv_beta_reg, t_inverse_cdf};
    use statrs::distribution::{Beta, ContinuousCDF, FisherSnedecor, StudentsT};

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

    // Where statrs finishes, the port gives its double, bit for bit: the domains BETA.INV, F.INV
    // and T.INV hand it (beta.rs, fisher.rs and t_dist.rs keep statrs below their thresholds).
    #[test]
    fn the_port_is_statrs_where_statrs_finishes() {
        let mut sweep = Sweep(0x1848);
        for _ in 0..20_000 {
            let shape_a = 10f64.powf(sweep.unit() * 8.0 - 3.0);
            let shape_b = 10f64.powf(sweep.unit() * 8.0 - 3.0);
            let p = if sweep.unit() < 0.5 {
                10f64.powf(-sweep.unit() * 15.0)
            } else {
                sweep.unit()
            };
            let ours = beta_inverse_cdf(shape_a, shape_b, p);
            let Ok(dist) = Beta::new(shape_a, shape_b) else {
                continue;
            };
            let theirs = dist.inverse_cdf(p);
            assert_eq!(
                ours.to_bits(),
                theirs.to_bits(),
                "Beta({shape_a},{shape_b}) at {p}"
            );
        }
        for _ in 0..20_000 {
            let df1 = (10f64.powf(sweep.unit() * 5.0))
                .trunc()
                .clamp(1.0, 99_999.0);
            let df2 = (10f64.powf(sweep.unit() * 5.0))
                .trunc()
                .clamp(1.0, 99_999.0);
            let p = sweep.unit();
            let ours = f_inverse_cdf(df1, df2, p);
            let theirs = FisherSnedecor::new(df1, df2).unwrap().inverse_cdf(p);
            assert_eq!(ours.to_bits(), theirs.to_bits(), "F({df1},{df2}) at {p}");
        }
        for _ in 0..20_000 {
            let df = (10f64.powf(sweep.unit() * 5.0))
                .trunc()
                .clamp(1.0, 99_999.0);
            let p = sweep.unit().max(1e-8);
            let ours = t_inverse_cdf(df, p);
            let theirs = StudentsT::new(0.0, 1.0, df).unwrap().inverse_cdf(p);
            assert_eq!(ours.to_bits(), theirs.to_bits(), "t({df}) at {p}");
        }
    }

    // The arguments where statrs panics or never finishes end, as NaN (`#NUM!` in the functions).
    #[test]
    fn statrs_panics_and_hangs_end_as_nan() {
        // statrs: NaN first estimate, its incomplete beta panics (F.INV(1E-300,5,5)).
        assert!(f_inverse_cdf(5.0, 5.0, 1e-300).is_nan());
        // statrs: an infinite Newton step, the adjustment loop never ends (F.INV(1E-50,6000,4),
        // BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)).
        assert!(f_inverse_cdf(6000.0, 4.0, 1e-50).is_nan());
        assert!(beta_inverse_cdf(
            1.002_401_701_325_834_6,
            12_177.162_026_553_196,
            1.0 - 1.1e-16
        )
        .is_nan());
        // The ends and invalid arguments.
        assert_eq!(inv_beta_reg(2.0, 3.0, 0.0), 0.0);
        assert_eq!(inv_beta_reg(2.0, 3.0, 1.0), 1.0);
        assert!(inv_beta_reg(0.0, 3.0, 0.5).is_nan());
        assert!(inv_beta_reg(2.0, 3.0, f64::NAN).is_nan());
    }

    // The slowest finishing case seen takes 377 Newton steps, under the cap of 512.
    #[test]
    fn the_longest_search_seen_still_finishes() {
        let x = inv_beta_reg(2266.0, 21.5, 3.718_810_279_553_989e-162);
        assert!(x > 0.0 && x < 1.0, "{x}");
    }
}
