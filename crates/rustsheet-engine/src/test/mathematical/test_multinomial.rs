#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_multinomial_basic() {
    let mut model = new_empty_model();
    // MULTINOMIAL(2,3,4) = 9! / (2! * 3! * 4!) = 362880 / (2 * 6 * 24) = 362880 / 288 = 1260
    model._set("A1", "=MULTINOMIAL(2,3,4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1260");
}

#[test]
fn test_multinomial_single() {
    let mut model = new_empty_model();
    // MULTINOMIAL(5) = 5!/5! = 1
    model._set("A1", "=MULTINOMIAL(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
}

#[test]
fn test_multinomial_zeros() {
    let mut model = new_empty_model();
    // MULTINOMIAL(0,0,1) = 1!/1! = 1
    model._set("A1", "=MULTINOMIAL(0,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
}

#[test]
fn test_multinomial_range() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("B1", "=MULTINOMIAL(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1260");
}

#[test]
fn test_multinomial_negative_error() {
    let mut model = new_empty_model();
    model._set("A1", "=MULTINOMIAL(2,-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
}

#[test]
fn test_multinomial_two_two() {
    let mut model = new_empty_model();
    // MULTINOMIAL(2,2) = 4! / (2! * 2!) = 24 / 4 = 6
    model._set("A1", "=MULTINOMIAL(2,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
}

#[test]
fn test_multinomial_overflow_num_error() {
    // Inputs large enough that (sum)! overflows f64 → must return #NUM!, not Inf.
    let mut model = new_empty_model();
    model._set("A1", "=MULTINOMIAL(100000,100000)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
}

#[test]
fn multinomial_past_170_is_num_at_once() {
    // probes/multinomial-large-arguments (excel-graph, 2026-10-09, T-1197): `#NUM!` once the
    // arguments add up to more than 170. Huge arguments used to loop once per unit (directive 5),
    // and `2^64` and above saturate instead of overflowing the sum.
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=MULTINOMIAL(171)"),
        ("A2", "=MULTINOMIAL(170,1)"),
        ("A3", "=MULTINOMIAL(86,86)"),
        ("A4", "=MULTINOMIAL(1E+15)"),
        ("A5", "=MULTINOMIAL(1E+300)"),
        ("A6", "=MULTINOMIAL(2^64,2^64)"),
        ("A7", "=MULTINOMIAL(3,1E+15)"),
    ];
    for (cell, formula) in cases {
        model._set(cell, formula);
    }
    model._set("B1", "=MULTINOMIAL(85,85)");
    model._set("B2", "=MULTINOMIAL(169,1)");
    let start = std::time::Instant::now();
    model.evaluate();
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
    for (cell, formula) in cases {
        assert_eq!(model._get_text(cell), "#NUM!", "{formula}");
    }
    assert_eq!(model._get_text("B2"), "170");
    assert!(
        model._get_text("B1").starts_with("9.1448"),
        "{}",
        model._get_text("B1")
    );
}

// gen-m6/2c1a6e93a1974f48 Calc!A35: -2.5 first, a boolean later in the range is still `#VALUE!`.
#[test]
fn range_boolean_is_value_even_after_a_negative() {
    let mut model = new_empty_model();
    model._set("A1", "-2.5");
    model._set("A2", "FALSE");
    model._set("B1", "-2.5");
    model._set("B2", "3");
    model._set("C1", "=MULTINOMIAL(A1:A2)");
    model._set("C2", "=MULTINOMIAL(B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#VALUE!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
}
