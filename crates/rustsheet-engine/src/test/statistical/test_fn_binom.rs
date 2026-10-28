#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, round_trip_value};

#[test]
fn test_fn_binom_dist_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST(6, 10, 0.5, TRUE)");
    model._set("A2", "=BINOM.DIST(6, 10, 0.5, FALSE)");
    model._set("A3", "=BINOM.DIST(6, 10, 0.5)"); // wrong args
    model._set("A4", "=BINOM.DIST(6, 10, 0.5, TRUE, FALSE)"); // too many args
    model.evaluate();

    // P(X <= 6) for X ~ Bin(10, 0.5) = 0.828125
    assert_eq!(model._get_text("A1"), *"0.828125");

    // P(X = 6) for X ~ Bin(10, 0.5) = 0.205078125
    assert_eq!(model._get_text("A2"), *"0.205078125");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_binom_dist_range_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST.RANGE(60, 0.75, 48)");
    model._set("A2", "=BINOM.DIST.RANGE(60, 0.75, 45, 50)");
    model._set("A3", "=BINOM.DIST.RANGE(60, 1.2, 45, 50)"); // p > 1 -> #NUM!
    model._set("A4", "=BINOM.DIST.RANGE(60, 0.75, 50, 45)"); // lower > upper -> #NUM!");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.083974967");

    assert_eq!(model._get_text("A2"), *"0.523629793");

    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn test_fn_binom_inv_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.INV(6, 0.5, 0.75)");
    model._set("A2", "=BINOM.INV(6, 0.5, -0.1)"); // alpha < 0 -> #NUM!
    model._set("A3", "=BINOM.INV(6, 1.2, 0.75)"); // p > 1 -> #NUM!
    model._set("A4", "=BINOM.INV(6, 0.5)"); // args error
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_binom_inv_small_trials_and_probabilities() {
    // These used to panic inside statrs' `inverse_cdf`.
    // Expected values: smallest k with CDF(k) >= alpha (scipy.stats.binom.ppf).
    let cases = [
        ("=BINOM.INV(1, 0.001, 0.1)", "0"),
        ("=BINOM.INV(1, 0.001, 0.9999)", "1"),
        ("=BINOM.INV(1, 0.5, 0.6)", "1"),
        ("=BINOM.INV(10, 0.01, 0.5)", "0"),
        ("=BINOM.INV(10, 0.01, 0.999)", "2"),
        ("=BINOM.INV(50, 0.001, 0.999)", "2"),
        ("=BINOM.INV(20, 0.3, 0.05)", "3"),
        ("=BINOM.INV(100, 0.99, 0.01)", "96"),
        ("=BINOM.INV(1000, 0.5, 0.5)", "500"),
        ("=BINOM.INV(1000000, 0.3, 0.9)", "300587"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_fn_binom_inv_probability_bounds_are_num_errors() {
    let cases = [
        ("=BINOM.INV(10, 0, 0.5)", "#NUM!"),
        ("=CRITBINOM(10, 0, 0.5)", "#NUM!"),
        ("=BINOM.INV(10, 1, 0.5)", "#NUM!"),
        ("=BINOM.INV(50, 1E-06, 0.9)", "0"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_fn_negbinom_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: PMF (non-cumulative) and CDF (cumulative)
    model._set("A1", "=NEGBINOM.DIST(10, 5, 0.25, FALSE)");
    model._set("A2", "=NEGBINOM.DIST(10, 5, 0.25, TRUE)");

    // Wrong number of arguments -> #ERROR!
    model._set("A3", "=NEGBINOM.DIST(10, 5, 0.25)");
    model._set("A4", "=NEGBINOM.DIST(10, 5, 0.25, TRUE, FALSE)");

    // Domain errors:
    // p < 0 or p > 1 -> #NUM!
    model._set("A5", "=NEGBINOM.DIST(10, 5, 1.5, TRUE)");
    // number_f < 0 -> #NUM!
    model._set("A6", "=NEGBINOM.DIST(-1, 5, 0.25, TRUE)");
    // number_s < 1 -> #NUM!
    model._set("A7", "=NEGBINOM.DIST(10, 0, 0.25, TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.05504866");
    assert_eq!(model._get_text("A2"), *"0.313514058");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn test_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST()");
    model._set("A2", "=BINOM.DIST(3)");
    model._set("A3", "=BINOM.DIST(3, 10)");
    model._set("A4", "=BINOM.DIST(3, 10, 0.3)");
    model._set("A5", "=BINOM.DIST(3, 10, 0.3, FALSE)");
    model._set("A6", "=BINOM.DIST(3, 10, 0.3, FALSE, TRUE)");

    model._set("B1", "=BINOM.DIST.RANGE()");
    model._set("B2", "=BINOM.DIST.RANGE(10)");
    model._set("B3", "=BINOM.DIST.RANGE(10, 0.3)");
    model._set("B4", "=BINOM.DIST.RANGE(10, 0.3, 3)");
    model._set("B5", "=BINOM.DIST.RANGE(10, 0.3, 3, 4)");
    model._set("B6", "=BINOM.DIST.RANGE(10, 0.3, 3, 4, TRUE)");

    model._set("C1", "=BINOM.INV()");
    model._set("C2", "=BINOM.INV(10)");
    model._set("C3", "=BINOM.INV(10, 0.5)");
    model._set("C4", "=BINOM.INV(10, 0.5, 0.5)");
    model._set("C5", "=BINOM.INV(10, 0.5, 0.5, TRUE)");

    model._set("D1", "=NEGBINOM.DIST()");
    model._set("D2", "=NEGBINOM.DIST(5)");
    model._set("D3", "=NEGBINOM.DIST(5, 10)");
    model._set("D4", "=NEGBINOM.DIST(5, 10, 0.3)");
    model._set("D5", "=NEGBINOM.DIST(5, 10, 0.3, TRUE)");
    model._set("D6", "=NEGBINOM.DIST(5, 10, 0.3, TRUE, FALSE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"0.266827932");
    assert_eq!(model._get_text("A6"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("B4"), *"0.266827932");
    assert_eq!(model._get_text("B5"), *"0.466948881");
    assert_eq!(model._get_text("B6"), *"#ERROR!");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
    assert_eq!(model._get_text("C3"), *"#ERROR!");
    assert_eq!(model._get_text("C4"), *"5");
    assert_eq!(model._get_text("C5"), *"#ERROR!");

    assert_eq!(model._get_text("D1"), *"#ERROR!");
    assert_eq!(model._get_text("D2"), *"#ERROR!");
    assert_eq!(model._get_text("D3"), *"#ERROR!");
    assert_eq!(model._get_text("D4"), *"#ERROR!");
    assert_eq!(model._get_text("D5"), *"0.003652521");
    assert_eq!(model._get_text("D6"), *"#ERROR!");
}

#[test]
fn test_booleans() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST(1,TRUE,0.7,TRUE)");
    model._set("A2", "=BINOM.DIST.RANGE(TRUE,0.3,0,1)");
    model._set("A3", "=BINOM.INV(TRUE,0.5,0.8)");
    model._set("A4", "=NEGBINOM.DIST(0,TRUE,0.7,TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"0.7");
}

/// No trials and a certain outcome (p = 0 or 1) take the shortcut before the double-double walk
/// (T-0927). Excel values: ironcalc-calc STATISTICAL/BINOM.xlsx BINOM.DIST E17, E25, E26 and
/// BINOM.DIST.RANGE E14, E24, E25; the rest follow from P(X = n p) = 1.
#[test]
fn no_trials_and_certain_outcomes() {
    let mut model = new_empty_model();
    let cells = [
        ("=BINOM.DIST(0,0,0,TRUE)", "1"),
        ("=BINOM.DIST(0,0,0.3,TRUE)", "1"),
        ("=BINOM.DIST(0,0,0.3,FALSE)", "1"),
        ("=BINOM.DIST(2,10,0,TRUE)", "1"),
        ("=BINOM.DIST(0,10,0,FALSE)", "1"),
        ("=BINOM.DIST(2,10,0,FALSE)", "0"),
        ("=BINOM.DIST(2,10,1,TRUE)", "0"),
        ("=BINOM.DIST(10,10,1,TRUE)", "1"),
        ("=BINOM.DIST(10,10,1,FALSE)", "1"),
        ("=BINOM.DIST(9,10,1,FALSE)", "0"),
        ("=BINOM.DIST.RANGE(0,0,0,0)", "1"),
        ("=BINOM.DIST.RANGE(10,0,2,3)", "0"),
        ("=BINOM.DIST.RANGE(10,0,0,3)", "1"),
        ("=BINOM.DIST.RANGE(10,1,2,3)", "0"),
        ("=BINOM.DIST.RANGE(10,1,9,10)", "1"),
        ("=NEGBINOM.DIST(2,3,1,TRUE)", "#NUM!"),
        ("=NEGBINOM.DIST(2,3,0,FALSE)", "#NUM!"),
    ];
    for (row, (formula, _)) in (1..).zip(&cells) {
        model._set(&format!("A{row}"), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in (1..).zip(&cells) {
        assert_eq!(model._get_text(&format!("A{row}")), *expected, "{formula}");
    }
}

// BINOM.DIST's closed forms (T-2645; statistics.md "BINOM.DIST pmf: closed forms at k = 0 and
// k = n"): EXP(fl(n·l)) at k = 0, cumulative or not, with l Excel's own log1p, and EXP(fl(n·LN(p)))
// for the pmf at k = n. Excel's bits from probes-numeric/t2145-binomial-sweep (A1541, A65, A1235,
// A22, A429, A3259, A363, A93, A2923, A139) and t2645-binom-trials-limit A6, A17.
#[test]
fn binom_dist_end_closed_forms_are_excel_bits() {
    let cases: [(&str, f64); 12] = [
        ("=BINOM.DIST(0,12,0.5,FALSE)", 0.000_244_140_625_000_000_16),
        ("=BINOM.DIST(0,2,0.2,FALSE)", 0.64),
        ("=BINOM.DIST(0,10,0.9,FALSE)", 9.999_999_999_999_96e-11),
        ("=BINOM.DIST(0,1,0.3,TRUE)", 0.7),
        ("=BINOMDIST(0,6,0.125,FALSE)", 0.448_795_318_603_515_6),
        ("=BINOM.DIST(100,100,0.999,FALSE)", 0.904_792_147_113_708_9),
        ("=BINOM.DIST(5,5,0.7,FALSE)", 0.168_069_999_999_999_94),
        ("=BINOM.DIST(2,2,0.99,FALSE)", 0.9801),
        ("=BINOM.DIST(50,50,0.3,FALSE)", 7.178_979_876_918_459e-27),
        ("=BINOM.DIST(3,3,0.2,FALSE)", 0.008_000_000_000_000_004),
        ("=BINOM.DIST(0,2^31-2,1E-12,FALSE)", 0.997_854_820_547_302_9),
        ("=BINOM.DIST(0,2^31-2,1E-12,TRUE)", 0.997_854_820_547_302_9),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, excel)) in cases.iter().enumerate() {
        let value = round_trip_value(&model, &format!("A{}", row + 1));
        assert_eq!(
            value.parse::<f64>().unwrap().to_bits(),
            excel.to_bits(),
            "{formula}"
        );
    }
}

// From 2^31 - 1 trials BINOM.DIST and BINOMDIST are #NUM! for every k and p, and so is
// BINOM.DIST.RANGE at number_s = 0 (probes-numeric/t2645-binom-trials-limit A7-A45); NEGBINOM.DIST
// is not (A46).
#[test]
fn binom_dist_trials_limit() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST(0,2^31-1,1E-12,FALSE)");
    model._set("A2", "=BINOM.DIST(0,2^31-0.5,1E-12,TRUE)");
    model._set("A3", "=BINOMDIST(0,2^31,1E-12,FALSE)");
    model._set("A4", "=BINOM.DIST(3,2^31-1,1E-9,TRUE)");
    model._set("A5", "=BINOM.DIST(0,2^31-1,1,FALSE)");
    model._set("A6", "=BINOM.DIST.RANGE(2^31-1,1E-12,0)");
    model._set("A7", "=NEGBINOM.DIST(2^31,1,1-1E-12,FALSE)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "A5", "A6"] {
        assert_eq!(model._get_text(cell), "#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A7"), "0");
}

/// BINOM.DIST's cdf at k = n - 1 is the upper tail of `I_p(n, 1)`, Excel's closed form on bratio's
/// oriented arguments (T-2502): probes-numeric t2145-binomial-sweep A374 and A1894, Excel's values.
#[test]
fn binom_dist_at_k_n_minus_one_takes_the_closed_form() {
    crate::test::util::check_cells(
        &[],
        &[
            ("A1", "=BINOM.DIST(4,5,0.9,TRUE)", "0.40950999999999993"),
            ("A2", "=BINOM.DIST(14,15,0.125,TRUE)", "0.9999999999999716"),
        ],
    );
}

/// NEGBINOM.DIST's cdf is `I_p(r, f+1)`, which at r = 1 or f = 0 is Excel's closed form (T-2502):
/// probes-numeric t2145-negbinom-sweep A34 (r = 1) and A728 (f = 0), Excel's values.
#[test]
fn negbinom_dist_at_one_success_or_no_failures_takes_the_closed_form() {
    crate::test::util::check_cells(
        &[],
        &[
            ("A1", "=NEGBINOM.DIST(5,1,0.25,TRUE)", "0.822021484375"),
            (
                "A2",
                "=NEGBINOM.DIST(0,5,0.1,TRUE)",
                "1.0000000000000016E-05",
            ),
        ],
    );
}

#[test]
fn test_negative_fractional_count_is_num_before_truncation() {
    // gen-m9 85f48d5cdfb24555 A35, d208bbafc3bc1997 A231: the sign is checked before truncation (T-2700).
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST(-0.0125, 10, 1E-16, FALSE)");
    model._set("A2", "=NEGBINOMDIST(-0.5, 8, 0.5)");
    model._set("A3", "=BINOM.DIST(2.9, 3.9, 0.5, FALSE)"); // a positive fraction still truncates
    model._set("A4", "=BINOM.DIST(2, 3, 0.5, FALSE)");
    model._set("A5", "=BINOM.DIST(-0, 3, 0.5, FALSE)"); // negative zero is no negative count
    model._set("A6", "=BINOM.DIST(0, 3, 0.5, FALSE)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), model._get_text("A4"));
    assert_eq!(model._get_text("A5"), model._get_text("A6"));
}
