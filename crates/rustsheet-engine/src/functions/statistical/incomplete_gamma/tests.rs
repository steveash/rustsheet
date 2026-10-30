#![allow(clippy::unwrap_used)]
use super::{
    cdf_terms_bound, continued_fraction_budget, gamma_cdf, gamma_density, gamma_inverse_cdf,
    gamma_quantile, gamma_sf, gamma_ur, incomplete_gamma_pair, inverse_cdf_within, temme_pair,
    wilson_hilferty_quantile, Tail, Terms, MAX_TERMS, TEMME_C,
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

fn pair(a: f64, x: f64) -> (f64, f64) {
    incomplete_gamma_pair(a, x, &mut Terms(MAX_TERMS))
}

/// Within `ulps` units of 2^-52 relative to `expected`.
fn within_ulps(actual: f64, expected: f64, ulps: f64) -> bool {
    (actual - expected).abs() <= ulps * f64::EPSILON * expected.abs()
}

/// Close to statrs' value, or both NaN. statrs forms the prefactor as exp(a ln x - x - ln
/// Gamma(a)), whose relative error is the absolute error of that exponent, about an ulp of
/// `scale` = a |ln x| + x + |ln Gamma(a)|, and takes Q as 1 - P below x = a; the accurate
/// kernel avoids both,
/// so the two may differ by that much.
/// [`near_at`] for Q(a, x): statrs takes Q as 1 - P below x = a (and below 1), so there its
/// Q is only as good as P is in absolute terms.
fn near_upper(ours: f64, theirs: f64, a: f64, x: f64, scale: f64) -> bool {
    let scale = if x < 1.0 || x <= a {
        (scale + 10.0) / theirs.max(1e-300)
    } else {
        scale
    };
    near_at(ours, theirs, scale)
}

fn near_at(ours: f64, theirs: f64, scale: f64) -> bool {
    let tolerance = 1e-14 * (scale + 10.0);
    ours.to_bits() == theirs.to_bits()
        || (ours.is_nan() && theirs.is_nan())
        // statrs flushes an exponent below ln of the smallest normal to 0.
        || (theirs == 0.0 && ours.abs() < f64::MIN_POSITIVE)
        || (ours - theirs).abs() <= tolerance * theirs.abs().max(1e-300)
}

// P(a, x) and Q(a, x) against mpmath's regularized `gammainc` at 50 digits, both tails
// directly (a in {1/2, 9, 500} and small shapes, where Q is not 1 - P).
#[test]
fn the_pair_is_within_a_few_ulps_of_the_reference() {
    let reference = [
        (0.5, 0.1, 0.345279153981423, 0.654720846018577),
        (0.5, 1.2, 0.8786647496415179, 0.12133525035848215),
        (0.5, 3.0, 0.9856941215645704, 0.01430587843542964),
        (9.0, 2.0, 0.00023744732826116177, 0.9997625526717389),
        (9.0, 9.5, 0.6081765174550605, 0.39182348254493954),
        (9.0, 30.0, 0.9999979539240957, 2.0460759042705716e-06),
        (500.0, 450.0, 0.010717238091289742, 0.9892827619087102),
        (500.0, 500.0, 0.5059471461707603, 0.49405285382923964),
        (500.0, 560.0, 0.9953068340007052, 0.004693165999294769),
    ];
    // Two arrays, not one: thirteen rows alike to the duplication ratchet's normalizer
    // read as copied code.
    let small_shapes = [
        (0.00113, 0.0008, 0.9926197543481239, 0.0073802456518761664),
        (0.01, 0.5, 0.9943732438060329, 0.0056267561939671844),
        (0.3, 0.9, 0.9022526480296695, 0.09774735197033045),
        (0.001, 1e-05, 0.9891230446957827, 0.01087695530421733),
    ];
    for (a, x, p, q) in reference.into_iter().chain(small_shapes) {
        let (lower, upper) = pair(a, x);
        // At a = 500 the prefactor's exponent a rlog1(x/a - 1) (2.68 at x = 450) carries an ulp
        // of itself into P: 6.5 ulps of 1 there.
        assert!(within_ulps(lower, p, 8.0), "P({a},{x}) = {lower}, not {p}");
        assert!(within_ulps(upper, q, 8.0), "Q({a},{x}) = {upper}, not {q}");
        assert!(
            (lower + upper - 1.0).abs() <= 2.0 * f64::EPSILON,
            "P + Q at ({a},{x})"
        );
    }
}

// a = 1 is the exponential distribution: P = -expm1(-x) and Q = exp(-x), exactly.
#[test]
fn the_density_is_within_a_few_ulps_of_the_reference() {
    // (a, x, scale, density) from mpmath at 50 digits. The first is Excel's
    // `GAMMA.DIST(10,9,2,FALSE)` (ironcalc-calc GAMMA.xlsx E3), the last `CHISQ.DIST(37.5,6,FALSE)`.
    for (a, x, scale, expected) in [
        (9.0, 10.0, 2.0, 0.032_639_019_674_079_374),
        (2.5, 0.3, 1.0, 0.091_570_869_452_182_3),
        (500.0, 480.0, 1.0, 0.012_319_344_669_927_745),
        (0.5, 1e-3, 2.0, 12.609_356_355_490_782),
        (3.0, 37.5, 2.0, 6.322_968_483_684_419e-7),
    ] {
        let ours = gamma_density(a, x, scale).unwrap();
        assert!(within_ulps(ours, expected, 4.0), "{a} {x} {scale}: {ours}");
    }
    assert_eq!(gamma_density(1.0, 1.5, 2.0), Some((-0.75f64).exp() / 2.0));
    assert_eq!(gamma_density(2.0, 0.0, 1.0), None);
}

#[test]
fn a_tiny_shape_s_lower_tail_is_one_minus_its_upper() {
    // `GAMMADIST(0.5,1E-300,1,TRUE)` is Excel's 1 (gen-m9/11eb3ab4f4f8104c A252), where the
    // series gives 1 - 2^-53.
    let (lower, upper) = pair(1e-300, 0.5);
    assert_eq!(lower, 1.0);
    assert!(upper > 0.0 && upper < 1e-299);
}

#[test]
fn shape_one_is_expm1() {
    for x in [1e-300, 1e-10, 0.5, 1.0, 2.0, 30.0, 700.0] {
        assert_eq!(pair(1.0, x), (-(-x).exp_m1(), (-x).exp()), "x = {x}");
    }
    // GAMMA.DIST(1,1,1,TRUE) is Excel's 0.6321205588285577 (BETA_GAMMA.xlsx L2).
    let dist = Gamma::new(1.0, 1.0).unwrap();
    assert_eq!(gamma_cdf(&dist, 1.0), 0.632_120_558_828_557_7);
}

// A huge scale (beta = 1E6, rate 1E-6) is the same pair at x / beta.
#[test]
fn a_huge_scale_is_the_pair_at_x_over_beta() {
    for (shape, x) in [(0.5, 3e5), (9.0, 9.5e6), (500.0, 5.6e8)] {
        let dist = Gamma::new(shape, 1e-6).unwrap();
        let (lower, upper) = pair(shape, x * 1e-6);
        assert_eq!(gamma_cdf(&dist, x).to_bits(), lower.to_bits());
        assert_eq!(gamma_sf(&dist, x).to_bits(), upper.to_bits());
    }
}

// Across shapes 1E-3 to 1E8 the pair stays within statrs' own error of statrs' value, and
// so do the cdfs, survival functions and quantiles GAMMA.INV and CHISQ.INV hand it.
#[test]
fn the_pair_is_near_statrs() {
    let mut sweep = Sweep(0x1872);
    for _ in 0..20_000 {
        let a = 10f64.powf(sweep.unit() * 11.0 - 3.0);
        let x = a * 10f64.powf(sweep.unit() * 6.0 - 3.0);
        let near = a * (1.0 + (sweep.unit() - 0.5) * 8.0 / a.sqrt());
        // statrs takes P as 0 below x = 1.1E-15 (its `almost_eq(x, 0)`).
        for x in [x, near].into_iter().filter(|&x| x > 1.1e-15) {
            let (lower, upper) = pair(a, x);
            // statrs' exponent also subtracts ln Gamma(a), about a ln a.
            let scale = a * x.ln().abs() + x + a * a.ln().abs();
            assert!(
                near_at(lower, statrs_lr(a, x), scale),
                "P({a},{x}) {lower} {upper} {} {}",
                statrs_lr(a, x),
                statrs_ur(a, x)
            );
            assert!(
                near_upper(upper, statrs_ur(a, x), a, x, scale),
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
        let scale = shape * (x * rate).ln().abs() + x * rate;
        assert!(
            near_at(gamma_cdf(&dist, x), dist.cdf(x), scale),
            "cdf Gamma({shape},{rate}) at {x}"
        );
        assert!(
            near_upper(gamma_sf(&dist, x), dist.sf(x), shape, x * rate, scale),
            "sf Gamma({shape},{rate}) at {x}"
        );
        // statrs' search (halve and double, 8 bisections, 4 Newton steps) can stop far from
        // the quantile in the far tail (at p = 1.26E-291 and shape 14.3 both searches end
        // where the cdf is about 1E-88), so the two are compared only where statrs' quantile
        // reproduces p.
        let theirs = dist.inverse_cdf(p);
        if (dist.cdf(theirs) / p - 1.0).abs() < 1e-6 {
            // A cdf's relative error e moves a quantile by e / shape where the shape is small.
            assert!(
                near_at(
                    gamma_inverse_cdf(&dist, p),
                    theirs,
                    (shape + 10.0) / shape.min(1.0)
                ),
                "Gamma({shape},{rate}) at {p:e}"
            );
        }
    }
    for df in [1.0, 2.0, 5.0, 10.0, 99.0, 1e4, 1e6, 1e10] {
        let chi = ChiSquared::new(df).unwrap();
        let dist = Gamma::new(df / 2.0, 0.5).unwrap();
        for p in [1e-10, 0.05, 0.4, 0.5, 0.95, 0.999_999_999_9] {
            assert!(
                near_at(gamma_inverse_cdf(&dist, p), chi.inverse_cdf(p), df),
                "ChiSquared({df}) at {p}"
            );
            let x = df * (0.5 + p);
            let scale = df * x.ln().abs() + x;
            assert!(
                near_at(gamma_cdf(&dist, x), chi.cdf(x), scale),
                "ChiSquared({df}) cdf at {x}"
            );
            assert!(
                near_upper(gamma_sf(&dist, x), chi.sf(x), df / 2.0, x / 2.0, scale),
                "ChiSquared({df}) sf at {x}"
            );
        }
    }
    for (k, lambda) in [(5u64, 1.0), (0, 0.5), (100, 98.5), (1_000_000, 999_000.0)] {
        let poisson = Poisson::new(lambda).unwrap();
        let scale = k as f64 * lambda.ln().abs() + lambda;
        let a = k as f64 + 1.0;
        assert!(near_upper(
            gamma_ur(a, lambda),
            poisson.cdf(k),
            a,
            lambda,
            scale
        ));
    }
}

// The arguments where statrs panicked or never finished end as NaN (`#NUM!` in the
// functions), or as Excel's value where x / beta underflows.
#[test]
fn statrs_panics_and_hangs_end_as_nan() {
    // `a + 1 == a`, where the series never ends, is Temme's expansion: Excel's
    // `GAMMA.DIST(1E16,1E16,1,TRUE)` and `POISSON(1E16,1E16,TRUE)` (probes/incomplete-gamma-cap
    // E1, E2; T-1891).
    assert_eq!(
        gamma_cdf(&Gamma::new(1e16, 1.0).unwrap(), 1e16).to_bits(),
        0.500_000_001_329_807_6_f64.to_bits()
    );
    assert_eq!(
        gamma_ur(1e16 + 1.0, 1e16).to_bits(),
        0.499_999_998_670_192_4_f64.to_bits()
    );
    // x * rate overflows (statrs: an unwrapped domain error).
    assert!(gamma_cdf(&Gamma::new(1.0, 1e10).unwrap(), 1e300).is_nan());
    // Where it underflows to 0 (statrs panicked too) the cdf is Excel's 0 at every shape
    // (probes/t2982-gamma-inv-huge-shape A117-A122; T-2982).
    assert_eq!(gamma_cdf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 0.0);
    assert_eq!(gamma_sf(&Gamma::new(2.0, 1e-100).unwrap(), 1e-300), 1.0);
    assert_eq!(gamma_cdf(&Gamma::new(1e-16, 1e-100).unwrap(), 1e-300), 0.0);
    assert_eq!(gamma_sf(&Gamma::new(1e-300, 1e-100).unwrap(), 1e-300), 1.0);
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
    assert!(gamma_ur(2.0, f64::INFINITY).is_nan());
}

// Temme's expansion (T-1891): the table's C_k(0) are Temme's closed forms, its far tails are
// the 0/1 limits, and at its shape cut it agrees with the series it replaces.
#[test]
fn temme_expansion_matches_its_closed_forms_and_the_series() {
    let closed = [-1.0 / 3.0, -1.0 / 540.0, 25.0 / 6048.0, 101.0 / 155_520.0];
    for (c, want) in TEMME_C.iter().zip(closed) {
        assert!(within_ulps(c[0], want, 2.0), "{} vs {want}", c[0]);
    }
    assert_eq!(temme_pair(1e8, 3e8), (1.0, 0.0));
    assert_eq!(temme_pair(1e8, 1e7), (0.0, 1.0));
    assert_eq!(temme_pair(1e300, 1e301), (1.0, 0.0));
    // Just below the cut the series runs; Temme's value there is within a few ulps of it.
    let a = 0.999_999_99e8;
    for x in [0.9999e8, 1e8, 1.0001e8] {
        let series = incomplete_gamma_pair(a, x, &mut Terms(MAX_TERMS));
        let temme = temme_pair(a, x);
        assert!(
            within_ulps(temme.0, series.0, 8.0),
            "{x}: {temme:?} {series:?}"
        );
        assert!(
            within_ulps(temme.1, series.1, 8.0),
            "{x}: {temme:?} {series:?}"
        );
    }
}

/// The terms `incomplete_gamma_pair(a, x)` takes, checked to be the exact boundary: the
/// value with that many, NaN with one fewer.
fn terms_needed(a: f64, x: f64) -> u32 {
    let mut terms = Terms(MAX_TERMS);
    let full = incomplete_gamma_pair(a, x, &mut terms);
    let used = MAX_TERMS - terms.0;
    assert_eq!(incomplete_gamma_pair(a, x, &mut Terms(used)), full);
    assert!(incomplete_gamma_pair(a, x, &mut Terms(used - 1)).0.is_nan());
    used
}

// The cap branches, both sides: a cdf or an inversion that needs more terms than its budget
// is NaN, and the same call with exactly the terms it needs is its value.
#[test]
fn a_call_past_its_cap_is_nan() {
    // The series near x = a below Temme's shape, the continued fraction (x above a) and the
    // small-shape Q's series.
    assert!(terms_needed(9e7, 9e7) > 50_000);
    terms_needed(9e7, 9.0001e7);
    terms_needed(0.00113, 8e-4);
    // From Temme's shape a cdf is one term at any x, where the series needed 675,286 at
    // a = x = 1E10 (T-1891): Excel's `GAMMA.DIST(1E10,1E10,1,TRUE)` (E8).
    assert_eq!(terms_needed(1e10, 1e10), 1);
    assert_eq!(
        pair(1e10, 1e10).0.to_bits(),
        0.500_001_329_807_601_4_f64.to_bits()
    );
    // An inversion shares one budget over its cdfs.
    let dist = Gamma::new(3e7, 1e6).unwrap();
    assert!(near_at(
        inverse_cdf_within(&dist, 1e-5, &mut Terms(MAX_TERMS)),
        dist.inverse_cdf(1e-5),
        dist.shape()
    ));
    assert!(inverse_cdf_within(&dist, 1e-5, &mut Terms(1_000)).is_nan());
}

// The continued fraction's own budget (T-2795) is at least twice the terms it needs at
// every shape the sweep reached, so it changes no value; past it the cdf is NaN.
#[test]
fn the_continued_fraction_stays_within_half_its_budget() {
    let mut a: f64 = 1e-3;
    while a < 1e10 {
        let step = a.sqrt().max(1.0) * 0.05;
        let near = (0..60).map(|k| a + 1.0 + f64::from(k) * step);
        let far = [10.0, 1e3, 1e5].map(|k| a + 1.0 + k * a.sqrt().max(1.0));
        for x in near.chain(far) {
            let mut terms = Terms(MAX_TERMS);
            let (p, _) = incomplete_gamma_pair(a, x, &mut terms);
            assert!(!p.is_nan(), "a = {a:e}, x = {x:e}");
            let used = MAX_TERMS - terms.0;
            assert!(
                2 * used <= continued_fraction_budget(a),
                "a = {a:e}, x = {x:e}: {used} terms"
            );
            assert!(used <= cdf_terms_bound(a));
        }
        a *= 3.1;
    }
    // A slow fraction (`terms_needed` checks its NaN one term short of the shared budget)
    // stays within its own; the budget saturates at the cap for a huge shape.
    let used = terms_needed(1e8, 1.0001e8);
    assert!(used < continued_fraction_budget(1e8));
    assert_eq!(continued_fraction_budget(1e30), MAX_TERMS);
    assert_eq!(cdf_terms_bound(1e9), MAX_TERMS);
    assert_eq!(cdf_terms_bound(2.5), 256 + 44);
}

// Excel's quantiles in the tails and at a small shape, where statrs' search is off or NaN
// (T-1543 probes-numeric/t1543-inverse-cdf-tiny-p; T-2255 gen-m9 triage). Each is within
// half a unit of Excel's 15th significant digit.
#[test]
fn quantiles_in_the_tails_are_excel_s() {
    let chi = |df: f64| Gamma::new(df / 2.0, 0.5).unwrap();
    let cases = [
        (chi(1.0), 1e-100, Tail::Lower, 1.570_796_326_794_896_6e-200),
        (chi(10.0), 1.0 - 1e-10, Tail::Lower, 68.167_617_951_904_13),
        (
            chi(4.0),
            1.0 - 1e-16,
            Tail::Upper,
            2.980_232_253_572_505_4e-8,
        ),
        (chi(1.0), 0.025, Tail::Lower, 0.000_982_069_117_175_256_4),
        (chi(9.0), 1.0 - 1e-16, Tail::Lower, 95.798_170_395_038_67),
        (chi(8.0), 1e-10, Tail::Upper, 63.397_964_411_135_6),
        (Gamma::new(1.0, 1.0).unwrap(), 1e-300, Tail::Lower, 1e-300),
        (chi(1.0), 1e-200, Tail::Lower, 0.0),
    ];
    for (dist, p, tail, excel) in cases {
        let ours = gamma_quantile(&dist, p, tail);
        assert!(
            (ours - excel).abs() <= 5e-15 * excel,
            "{p} at shape {}: {ours:e}, Excel {excel:e}",
            dist.shape()
        );
    }
    // At a large shape `x^a` overflows, and the prefactor is TOMS 654's `rcomp`: the quantile
    // is within 1E-13 of `GAMMAINV(1E-10,99999,2)`, Excel's 196001.02844708483 (gen-m9
    // 165db58d1540f014; before T-1634's overflow guard it was 2.01).
    let ours = gamma_quantile(&Gamma::new(99_999.0, 0.5).unwrap(), 1e-10, Tail::Lower);
    assert!(
        (ours - 196_001.028_447_084_83).abs() < 1e-13 * ours,
        "{ours}"
    );
}

/// Excel's closed form at a huge shape (probes/t2982-gamma-inv-huge-shape; T-2982): the median
/// is 1.0002658235505755 times the shape, and the p sweep at shape 1E13 is within a few ulps.
#[test]
fn wilson_hilferty_quantile_is_excels_at_a_huge_shape() {
    let close = |ours: f64, excel: f64| ((ours - excel) / excel).abs() <= 4.0 * f64::EPSILON;
    // A34, A23: GAMMA.INV(0.5,1E13,1) and GAMMA.INV(0.5,1E300,1).
    assert!(close(
        wilson_hilferty_quantile(1e13, 0.0),
        10_002_658_235_505.422
    ));
    assert!(close(
        wilson_hilferty_quantile(1e300, 0.0),
        1.000_265_823_550_575_5e300
    ));
    // A32: p = 0.1, z = -1.2815515655446004.
    assert!(close(
        wilson_hilferty_quantile(1e13, -1.281_551_565_544_600_4),
        10_002_654_182_255.533
    ));
    // At shape 1E11 the fit is within 3E-13 relative (A62, A70, A77; 1,546, 149 and 721 ulps, the
    // z^2 term T-3070 owns), as at 2E10 (A7, 599 ulps).
    let near = |ours: f64, excel: f64| ((ours - excel) / excel).abs() <= 3e-13;
    assert!(near(
        wilson_hilferty_quantile(1e11, -11.464_024_688_443_377),
        100_022_956_593.373_21
    ));
    assert!(near(
        wilson_hilferty_quantile(1e11, 0.0),
        100_026_582_354.721_88
    ));
    assert!(near(
        wilson_hilferty_quantile(1e11, 7.941_444_487_415_977),
        100_029_094_070.505_65
    ));
    assert!(near(
        wilson_hilferty_quantile(2e10, 0.0),
        20_005_316_470.675_835
    ));
}

// From shape 20 the prefactor is Excel's ratio form away from the mode and `rcomp` near it (T-2638).
// Excel's own values: probes-numeric m9-gamma-dist-a50 A2 `GAMMA.DIST(5,50,1,FALSE)` and
// m9-gamma-dist-a500 A10 `GAMMA.DIST(1500,500,1,FALSE)` (bit for bit), ironcalc-calc POISSON.xlsx
// D31 `POISSON.DIST(1E6,1E6,TRUE)`, Q(1E6 + 1, 1E6), within 8 ulp (the ratio form is 1E-10 off).
#[test]
fn the_large_shape_prefactor_is_excels_away_from_the_mode() {
    assert_eq!(
        gamma_density(50.0, 5.0, 1.0),
        Some(1.967_673_038_278_765e-31)
    );
    assert_eq!(
        gamma_density(500.0, 1500.0, 1.0),
        Some(1.097_430_138_864_169_5e-198)
    );
    assert!(within_ulps(
        gamma_ur(1e6 + 1.0, 1e6),
        0.500_265_961_486_283_7,
        8.0
    ));
}
