//! Excel's far upper tail of Q(a, x) at a shape from 3/2 to [`FAR_TAIL_MAX_SHAPE`] (T-3269):
//! POISSON's cdf and CHISQ.DIST.RT (CHIDIST, CHISQ.TEST) past x = [`FAR_TAIL_MIN_X`].
//!
//! There Excel is 10 to 400 ulp from the true Q (probes-numeric/t3284-gamma-deep-upper-tail,
//! docs/excel-semantics/statistics.md "Excel's far upper tail (T-3269)"), and its error is the one
//! T-2638 found above shape 20 away from the mode: the prefactor is Stirling's ratio form
//! sqrt(a / 2 pi) exp(-B - correction(a)) ([`stirling_prefactor`]), whose exponent, rounded to a
//! double, carries an ulp of a number near ln Q into the result. B is Loader's direct deviance
//! a ln(a/x) + (x - a) ([`direct_deviance`]), not the ratio form a ln(x/a) + (a - x) T-3269 first
//! took: the two round x/a and a/x differently, which put the ratio form 25 to 189 ulp off at
//! the cells T-3469 owns (shape 6 at x = 38 and 42, x / a = 10). Over the 32 adjacent doubles of
//! `POISSON(2,100+j*2^-46,TRUE)` Excel jumps by 64 ulp exactly where that exponent's rounding
//! does. The continued fraction it multiplies is taken accurately ([`accurate_fraction`]); the
//! correction is the exact Stirling error ([`stirlerr_exact`]), since TOMS's series is only good
//! from 8.
//!
//! The form is applied only where it was measured (decision 0188 R4.3): x in
//! ([`FAR_TAIL_MIN_X`], [`FAR_TAIL_MAX_X`]] at the multiples of 1/2 up to [`FAR_TAIL_MAX_SHAPE`].
//! At and below x = 30 Excel is the accurate value at integer shapes (where this form is up to 22
//! ulp off), and above shape 11 next to the cut Excel is not this form, so the callers keep their
//! accurate kernels there.

use super::super::half_integer_beta::is_half_integer_shape;
use super::super::loader::stirlerr_exact;
use super::density::{direct_deviance, in_window, window_exponent};
use super::fraction::accurate_fraction;
use super::{stirling_prefactor, Terms};

/// The x above which Excel takes the ratio form. Bracketed by the T-3269 boundary probe
/// (probes-numeric/t3269-gamma-tail-switch, x from 26 to 40 by 0.5 at shapes 2 to 4): at shapes
/// 2, 3 and 4 Excel is the correctly rounded Q at every x up to 30 and the ratio form from 30.5 on
/// (`POISSON(2,30,TRUE)` exact, `POISSON(2,30.5,TRUE)` 32 ulp from exact and 3 from the form).
/// FITTED(T-3269, 145 cells, x 26..40, shape 2..4)
const FAR_TAIL_MIN_X: f64 = 30.0;

/// The largest shape the form is applied at: the cut at [`FAR_TAIL_MIN_X`] is Excel's up to here.
/// On the R4 hold-out's train cells next to the cut (30 < x <= 45, docs/research/t3269/holdout.py),
/// counting only gaps above the oracle's 4-ulp floor (decision 0188 R4.4), the form is closer to
/// Excel than the accurate kernel at 80 cells and farther at 12 for shapes 3/2 to 11 (test: 8 and
/// 1); above 11 it is closer at 1 and farther at 3 (`CHISQ.DIST.RT(63.1168,23)` 19 ulp against 10,
/// `CHISQ.DIST.RT(68.6972159,27)` 10 against 2, `POISSON(17,31.4903,TRUE)` 36 against 28), so
/// Excel's switch is elsewhere there (T-3469) and the accurate kernel stays.
/// FITTED(T-3269, 153 cells, shape 1.5..19.5, x 30..45)
const FAR_TAIL_MAX_SHAPE: f64 = 11.0;

/// The largest x the form was measured at (`POISSON(19,600,TRUE)`, t3284-gamma-deep-upper-tail
/// A111); beyond it the callers keep the accurate kernel (decision 0188 R4.3). (UNVERIFIED,
/// T-3469): no cell past it says where Excel leaves the form.
const FAR_TAIL_MAX_X: f64 = 600.0;

/// Excel's Q(a, x) for a shape that is a multiple of 1/2 from 3/2 to [`FAR_TAIL_MAX_SHAPE`] and
/// [`FAR_TAIL_MIN_X`] < x <= [`FAR_TAIL_MAX_X`]; `None` elsewhere (a = 1 is exp(-x), which
/// Excel has correctly rounded), where the result is not a normal double, or past `terms`.
pub(super) fn excel_far_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    let measured = is_half_integer_shape(a) && a > 1.0 && a <= FAR_TAIL_MAX_SHAPE;
    if !(measured && x > FAR_TAIL_MIN_X && x <= FAR_TAIL_MAX_X) {
        return None;
    }
    let fraction = accurate_fraction(a, x, terms)?;
    // Loader's direct deviance, not the ratio form a ln(x/a) + (a - x): x > a here, so the two
    // differ by the rounding of x/a against a/x, which |ln Q| (30 to 90) amplifies to the 25 and 72
    // ulp T-3469 found at Q(6, 42) and Q(10, 100).
    let prefactor = stirling_prefactor(a, -direct_deviance(a, x), stirlerr_exact(a).0);
    let upper = prefactor * (fraction.0 + fraction.1);
    upper.is_normal().then_some(upper)
}

/// The smallest shape the far-above-the-mean form ([`excel_far_above_mean`]) is applied at:
/// POISSON's k = 30, the lowest k of the T-4302 probes (probes/t4291-poisson-far-above-mean,
/// probes/t4302-poisson-cdf-beside-pmf); below it Excel is unprobed on this form.
const FAR_ABOVE_MIN_SHAPE: f64 = 31.0;

/// The largest shape the far-above-the-mean form was measured at (k = 300).
const FAR_ABOVE_MAX_SHAPE: f64 = 301.0;

/// The lower end of x / a for the far-above-the-mean form: the upper end of Excel's gamma window
/// (T-2638, T-4290), inside which its prefactor takes another chain (T-4308). The T-4302 probes
/// bracket it in (1.495, 1.549): at x / a 1.29 to 1.495 the cdf is up to 82 ulp from this form
/// (within 4 ulp at 5 of 24 points).
/// FITTED(T-4302, 96 cells, x/a 1.26..2.99, shape 31..301)
const FAR_ABOVE_MIN_RATIO: f64 = 1.5;

/// The upper end of x / a measured: the largest probed point is 900/301 = 2.99003a
/// (`POISSON.DIST(300,900,TRUE)`); past it the accurate kernel stays (decision 0188 R4.3).
const FAR_ABOVE_MAX_RATIO: f64 = 2.9901;

/// Excel's POISSON cdf Q(a, x) far above the mean (T-4302): at an integer shape from
/// [`FAR_ABOVE_MIN_SHAPE`] to [`FAR_ABOVE_MAX_SHAPE`] and 1.5a < x <= 2.9901a, the prefactor
/// sqrt(a / 2 pi) exp(-B - c(a)) with Loader's direct deviance B = a ln(a/x) + (x - a), its
/// difference x - a taken first, and the exact Stirling error c, times the accurate fraction.
/// Excel's cdf there is its own gamma density times x times the fraction to a few ulp, and that
/// density is exp(-c - B) / sqrt(2 pi a) times (a / x) bit for bit at all 120 probed points
/// (train, hold-out and the fresh sweep; docs/excel-semantics/statistics.md, the T-4302
/// paragraph). Excel's implied B is within half an ulp of this one at 118 of them, and of the
/// ratio form a ln(x/a) + (a - x) at 68. `None` elsewhere, where the result is not a normal
/// double, or past `terms`.
pub(super) fn excel_far_above_mean(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    if !(measured_poisson_shape(a) && x > a * FAR_ABOVE_MIN_RATIO && x <= a * FAR_ABOVE_MAX_RATIO) {
        return None;
    }
    loader_upper(a, x, direct_deviance(a, x), terms)
}

/// Whether `a` is an integer shape (POISSON's k + 1) from [`FAR_ABOVE_MIN_SHAPE`] to
/// [`FAR_ABOVE_MAX_SHAPE`], the k of every T-4302 and T-4322 probe.
fn measured_poisson_shape(a: f64) -> bool {
    a.fract() == 0.0 && (FAR_ABOVE_MIN_SHAPE..=FAR_ABOVE_MAX_SHAPE).contains(&a)
}

/// sqrt(a / 2 pi) exp(-B - c(a)) times the accurate fraction, for Excel's deviance B and the exact
/// Stirling error c: `None` where the result is not a normal double, or past `terms`.
fn loader_upper(a: f64, x: f64, deviance: f64, terms: &mut Terms) -> Option<f64> {
    let fraction = accurate_fraction(a, x, terms)?;
    let prefactor = stirling_prefactor(a, -deviance, stirlerr_exact(a).0);
    let upper = prefactor * (fraction.0 + fraction.1);
    upper.is_normal().then_some(upper)
}

/// The smallest x / a at which POISSON's cdf takes the window exponent ([`excel_window_upper`]).
/// Bracketed by the T-4302 probes in (1.2957, 1.3068): from 1.3068 to 1.495 Excel's cdf is the
/// window form to 0 to 6 ulp at 15 of 15 points (probes/t4302-poisson-cdf-beside-pmf B21, F24, A19,
/// E21, F20, H24, A22 and probes/t4291-poisson-far-above-mean row 2), and at 1.258 to 1.2957 it is
/// up to 62 ulp from it at shapes 81 to 301 (t4291 D1:H1, probes/t3252-gamma-prefactor-window A137).
/// 1.3 (Cephes' `SMALLRATIO`, the cut of its asymptotic igamc series) was frozen inside that
/// bracket; the ladder of the fresh sweep then put Excel's cut in (1.298, 1.3] at k = 100 and 300
/// (probes-numeric/t4322-poisson-window-r4-fresh A8:C13: at 1.3 the form is 2, 0 and -5 ulp off at
/// k = 100, 200 and 300 where the correctly rounded Q is 14, -5 and -63, and at 1.298 and below it
/// is 13 to 85 ulp off at k = 100 and 300). At k = 200 the form is as close as the correctly rounded
/// Q just below 1.3 (B8:B10), so the cut is not bracketed there.
/// What Excel takes below the cut is T-4345's.
/// FITTED(T-4322, 24 cells, x/a 1.26..1.5, shape 31..301)
const WINDOW_MIN_RATIO: f64 = 1.3;

/// Excel's POISSON cdf Q(a, x) in the upper part of its gamma window (T-4322): at an integer shape
/// from [`FAR_ABOVE_MIN_SHAPE`] to [`FAR_ABOVE_MAX_SHAPE`] and 1.3a <= x inside the density's
/// window (x <= 1.5a, [`in_window`]), the prefactor sqrt(a / 2 pi) exp(-B - c(a)) with the
/// density's window exponent B ([`window_exponent`], T-4308) times the accurate fraction. Below
/// 1.3a Excel is not this form (it is a few ulp from the true Q there), and above 1.5a the far
/// form takes over ([`excel_far_above_mean`]). `None` elsewhere, where the result is not a
/// normal double, or past `terms`.
pub(super) fn excel_window_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    if !(measured_poisson_shape(a) && x >= a * WINDOW_MIN_RATIO && in_window(a, x)) {
        return None;
    }
    loader_upper(a, x, window_exponent(a, x), terms)
}

#[cfg(test)]
mod tests {
    use super::super::{gamma_ur, poisson_upper};

    // Excel's far upper tail (T-3469): the Stirling prefactor's exponent is Loader's direct
    // deviance a ln(a/x) + (x - a), not the ratio form a ln(x/a) + (a - x), which was 25 and 72
    // ulp off at these cells (gen-m9/43c6227e0ff368c7 Calc!A108 `POISSON(5,42,-1)`,
    // gen-m9/5919660b7bb4d195 Calc!A80 `POISSON(9,100,..)` and its twin probes-numeric/
    // t1049-chisqdistrt Calc!A28 `CHISQ.DIST.RT(200,20)`).
    #[test]
    fn far_upper_tail_keeps_the_reciprocal_log_argument() {
        let excel = [
            (6.0, 42.0, 7.083_529_869_768_314e-13_f64),
            (10.0, 100.0, 1.125_347_396_084_264e-31),
        ];
        for (a, x, q) in excel {
            assert_eq!(poisson_upper(a, x).to_bits(), q.to_bits(), "Q({a}, {x})");
        }
    }

    // Excel's cdf far above the mean (T-4302): the prefactor with Loader's direct deviance
    // times the accurate fraction gives Excel's own bits at these cells
    // (probes/t4291-poisson-far-above-mean G6, C6; probes/t4302-poisson-cdf-beside-pmf B20,
    // F19), where Excel is 208 to 265 ulp from the true Q.
    #[test]
    fn far_above_the_mean_the_cdf_takes_loader_s_deviance() {
        let excel = [
            (250.0, 750.0, 5.134_850_238_125_346e-100_f64),
            (60.0, 180.0, 2.497_452_789_099_686_4e-25),
            (173.0, 511.142, 1.595_806_199_959_312_8e-67),
            (211.0, 620.901, 3.197_252_522_551_427_3e-81),
        ];
        for (k, mean, q) in excel {
            assert_eq!(
                poisson_upper(k + 1.0, mean).to_bits(),
                q.to_bits(),
                "POISSON({k}, {mean}, TRUE)"
            );
            assert_ne!(gamma_ur(k + 1.0, mean).to_bits(), q.to_bits());
        }
        // Outside the measured range the accurate kernel stays: past shape 301, below shape 31,
        // and at a non-integer shape.
        for (a, x) in [(302.0, 700.0), (30.0, 70.0), (100.5, 200.0)] {
            assert_eq!(poisson_upper(a, x).to_bits(), gamma_ur(a, x).to_bits());
        }
    }

    // Excel's cdf in the upper gamma window (T-4322): the prefactor with the density's window
    // exponent times the accurate fraction gives Excel's own bits at these cells, from the 1.3a cut
    // (`POISSON.DIST(200,261.3,TRUE)`) to 1.49a (probes-numeric/t4322-poisson-window-r4-fresh B11,
    // D2, H4; probes/t4302-poisson-cdf-beside-pmf F24; probes/t4291-poisson-far-above-mean F2).
    #[test]
    fn in_the_upper_window_the_cdf_takes_the_window_exponent() {
        let excel = [
            (200.0, 261.3, 4.564_449_907_593_518e-5_f64),
            (174.0, 256.207, 3.169_584_283_420_252_6e-8),
            (162.0, 217.186, 5.360_543_525_047_952_4e-5),
            (58.0, 78.4329, 0.009_691_412_009_331_235),
            (150.0, 225.0, 6.617_514_047_185_189e-8),
        ];
        for (k, mean, q) in excel {
            assert_eq!(
                poisson_upper(k + 1.0, mean).to_bits(),
                q.to_bits(),
                "POISSON({k}, {mean}, TRUE)"
            );
        }
        // Below the 1.3a cut Excel is not the window form, so the accurate kernel stays.
        for (a, x) in [(101.0, 130.0), (301.0, 389.0)] {
            assert_eq!(poisson_upper(a, x).to_bits(), gamma_ur(a, x).to_bits());
        }
    }
}
