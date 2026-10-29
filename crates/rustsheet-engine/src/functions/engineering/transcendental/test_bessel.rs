use crate::functions::engineering::transcendental::bessel_k;

use super::bessel_i::bessel_i;

const EPS_LOW: f64 = 1e-6;

// Known values computed with Arb via Nemo.jl in Julia
// You can also use Mathematica
// But please do not use Excel or any other software without arbitrary precision

fn numbers_are_somewhat_close(a: f64, b: f64) -> bool {
    if a == b {
        // avoid underflow if a = b = 0.0
        return true;
    }
    (a - b).abs() / ((a * a + b * b).sqrt()) < EPS_LOW
}

#[test]
fn bessel_in_known_values() {
    let cases = [
        (1, 0.5, 0.2578943053908963),
        (3, 0.5, 0.002645111968990286),
        (7, 0.2, 1.986608521182497e-11),
        (7, 0.0, 0.0),
        (0, -0.5, 1.0634833707413236),
        // worse case scenario
        (0, 3.7499, 9.118167894541882),
        (0, 3.7501, 9.119723897590003),
    ];
    for (n, value, known) in cases {
        let f = bessel_i(n, value);
        assert!(
            numbers_are_somewhat_close(f, known),
            "Got: {f}, expected: {known} for in({n}, {value})"
        );
    }
}

#[test]
fn bessel_kn_known_values() {
    let cases = [
        (1, 0.5, 1.656441120003301),
        (0, 0.5, 0.9244190712276659),
        (3, 0.5, 62.05790952993026),
    ];
    for (n, value, known) in cases {
        let f = bessel_k(n, value);
        assert!(
            numbers_are_somewhat_close(f, known),
            "Got: {f}, expected: {known} for kn({n}, {value})"
        );
    }
}

// Expected values are Excel-cached cells from BESSEL.xlsx (ironcalc-calc suite); they must match
// bit-for-bit, not within a tolerance, because Excel's algorithm (Numerical Recipes) is only
// ~1e-8 accurate.
#[test]
fn excel_bessel_small_args_are_bit_exact() {
    use super::bessel_excel::{excel_bessel_j, excel_bessel_y};
    assert_eq!(excel_bessel_j(0, 0.0), 1.000_000_002_831_41);
    assert_eq!(excel_bessel_y(0, 0.8), -0.086_802_273_855_431_45);
    assert_eq!(excel_bessel_y(10, 2.9), -3_564.032_241_223_704);
    assert_eq!(excel_bessel_y(7, 3.5), -7.848_865_602_729_761);
    assert_eq!(excel_bessel_y(1, 0.2), -3.323_824_986_221_588_5);
    assert_eq!(excel_bessel_y(7, 0.2), -2_295_654_720.157_33);
}

// Excel values from probes-numeric/m9-bessel (T-1004, excel-graph), bit-for-bit: the
// large-argument asymptotic branch with Excel's own coefficients (T-0908).
#[test]
fn excel_bessel_large_args_are_bit_exact() {
    use super::bessel_excel::{excel_bessel_j, excel_bessel_y};
    assert_eq!(excel_bessel_j(0, 8.0), 0.171_650_807_317_693_55);
    assert_eq!(excel_bessel_j(1, 8.0), 0.234_628_386_507_074_4);
    assert_eq!(excel_bessel_j(1, 30.0), -0.118_751_060_911_231_17);
    assert_eq!(excel_bessel_j(4, 8.5), -0.207_695_125_000_947_22);
    assert_eq!(excel_bessel_y(0, 8.0), 0.223_521_489_249_680_93);
    assert_eq!(excel_bessel_y(1, 8.5), -0.026_168_679_623_921_92);
    assert_eq!(excel_bessel_y(8, 10.0), 0.001_075_473_508_846_236_4);
    // BESSEL.xlsx (Excel-cached) has x = -10 and -32: J of an odd order is odd in x, even is even.
    assert_eq!(excel_bessel_j(1, -30.0), 0.118_751_060_911_231_17);
    assert_eq!(excel_bessel_j(0, -8.0), excel_bessel_j(0, 8.0));
    assert_eq!(excel_bessel_j(4, -8.5), excel_bessel_j(4, 8.5));
    assert_eq!(excel_bessel_j(3, 0.0), 0.0);
    assert!(excel_bessel_y(0, 0.0).is_nan());
    assert!(excel_bessel_y(2, -10.0).is_nan());
}

#[test]
fn excel_bessel_huge_order_is_nan_not_panic() {
    use super::bessel_excel::{excel_bessel_j, excel_bessel_y};
    assert!(excel_bessel_j(i32::MAX, 1.0).is_nan());
    assert!(excel_bessel_y(i32::MAX, 1.0).is_nan());
}

#[test]
fn excel_bessel_j_odd_order_is_odd_in_x() {
    use super::bessel_excel::excel_bessel_j;
    // Covers the forward (n < x) and Miller (n >= x) branches.
    for (n, x) in [(1, 2.5), (7, 3.5), (7, 6.0), (3, 1.2)] {
        assert_eq!(excel_bessel_j(n, -x), -excel_bessel_j(n, x));
        assert_eq!(excel_bessel_j(n + 1, -x), excel_bessel_j(n + 1, x));
    }
}

/// Past `MAX_ORDER` the O(1) rules give 0 only where the uncapped recurrence (Excel's algorithm)
/// computes exactly 0, over a log grid of x that spans the tiny-x overflow, the underflow and the
/// turning point x = n (T-2546).
#[test]
fn past_cap_rules_agree_with_the_recurrence() {
    use super::bessel_excel::{excel_bessel_j, jn_miller};
    use super::bessel_i::in_miller;
    let grid = |lo: f64, hi: f64, points: u32| {
        (0..=points).map(move |i| lo * (hi / lo).powf(f64::from(i) / f64::from(points)))
    };
    let (mut j_zeros, mut i_zeros) = (0, 0);
    for n in [100_001, 160_001] {
        let order = f64::from(n);
        for x in grid(1e-7, order, 120).chain(grid(1.5e-5, 4e-5, 12)) {
            if excel_bessel_j(n, x) == 0.0 {
                j_zeros += 1;
                assert_eq!(jn_miller(n, x), 0.0, "BESSELJ({x},{n})");
            }
        }
        for x in grid(1e-7, 720.0, 120).chain(grid(1.5e-5, 6e-5, 12)) {
            if bessel_i(n, x) == 0.0 {
                i_zeros += 1;
                assert_eq!(in_miller(n, x), 0.0, "BESSELI({x},{n})");
            }
        }
    }
    // Most of each grid is 0, so the rules are not vacuous.
    assert!(j_zeros > 150 && i_zeros > 150, "{j_zeros} {i_zeros}");
}

// probes/t2546-bessel-past-cap (excel-graph) and the gen-m9 cells of T-2546.
#[test]
fn past_cap_values_match_excel() {
    use super::bessel_excel::excel_bessel_j;
    let zero_j = [
        (100_001, 5e4),
        (100_001, 9e4),
        (100_001, 1e-3),
        (100_001, 2e-5),
        (100_001, -10.0),
        (100_002, -10.0),
        (1_000_000, 0.0),
        (1_000_000, 1e-3),
        (1_000_000, 3e-4),
        (1_000_000, 1e5),
        (5_000_000, 10.0),
        (10_000_000, 0.0),
    ];
    for (n, x) in zero_j {
        assert_eq!(excel_bessel_j(n, x).to_bits(), 0, "BESSELJ({x},{n})");
    }
    // Excel's tiny-x overflow and its own cap are `#NUM!`; x near or past n has a value there
    // (the remaining deviation).
    for (n, x) in [
        (100_001, 1e-5),
        (100_001, 1e-300),
        (1_000_000, 3e-5),
        (10_000_001, 0.0),
    ] {
        assert!(excel_bessel_j(n, x).is_nan(), "BESSELJ({x},{n})");
    }
    let zero_i = [
        (100_001, 100.0),
        (100_001, 700.0),
        (100_001, 1e-3),
        (100_001, 5e-5),
        (100_001, 0.0),
        (100_001, -2.5),
        (100_002, -2.5),
        (1_000_000, 2.5),
        (1_000_000, 8.0),
        (1_000_000, 1e-3),
        (10_000_000, 100.0),
    ];
    for (n, x) in zero_i {
        assert_eq!(bessel_i(n, x).to_bits(), 0, "BESSELI({x},{n})");
    }
    for (n, x) in [
        (100_001, 1e4),
        (100_001, 713.0),
        (100_001, 2e-5),
        (10_000_001, 100.0),
    ] {
        assert!(bessel_i(n, x).is_nan(), "BESSELI({x},{n})");
    }
    for (n, x) in [
        (100_001, 706.0),
        (100_001, 25019.0),
        (1_000, 720.0),
        (10_000_000, 800.0),
    ] {
        assert_eq!(bessel_k(n, x).to_bits(), 0, "BESSELK({x},{n})");
    }
    for (n, x) in [
        (100_001, 705.0),
        (100_001, 2.5),
        (100_001, 1e-300),
        (10_000_001, 800.0),
    ] {
        assert!(!bessel_k(n, x).is_finite(), "BESSELK({x},{n})");
    }
}

/// Both recurrences round each step twice, as Excel's x87 code does: plain double steps miss these
/// near the turning point n = x (probes/t2881-besselj-turning-point, excel-graph, T-2548).
#[test]
fn turning_point_recurrences_round_twice() {
    use super::bessel_excel::excel_bessel_j;
    for (x, n, excel) in [
        // Miller's downward recurrence (x <= n).
        (5000.0, 5000, 0.026_158_686_649_286_743),
        (990.0, 1000, 0.012_361_942_456_230_18),
        (100_000.0, 100_000, 0.009_636_944_011_336_267),
        // The upward recurrence (x > n).
        (2001.0, 2000, 0.038_080_655_425_271_136),
        (100_100.0, 100_000, -0.006_729_033_731_473_101),
    ] {
        assert_eq!(excel_bessel_j(n, x), excel, "BESSELJ({x},{n})");
    }
}

#[test]
fn bessel_i_is_nan_past_the_overflow_cut() {
    // probes/besseli-overflow-cut-finest: 709.7 is finite and 709.70001 is #NUM! (T-2613).
    assert!(bessel_i(0, 709.7).is_finite());
    for n in [0, 1, 5] {
        assert!(bessel_i(n, 709.70001).is_nan(), "n = {n}");
    }
}
