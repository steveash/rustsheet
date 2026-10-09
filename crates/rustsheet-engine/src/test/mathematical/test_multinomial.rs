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
fn test_multinomial_sum_past_170_is_num_error() {
    // Excel (probes/multinomial-huge): 170 is a number; a sum past 170 is #NUM!.
    let mut model = new_empty_model();
    model._set("A1", "=MULTINOMIAL(170)");
    model._set("A2", "=MULTINOMIAL(171)");
    model._set("A3", "=MULTINOMIAL(170,1)");
    model._set("A4", "=MULTINOMIAL(85,85)");
    model._set("A5", "=MULTINOMIAL(100,71)");
    model._set("A6", "=MULTINOMIAL(1E+15,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_ne!(model._get_text("A4"), "#NUM!");
    assert_eq!(model._get_text("A5"), "#NUM!");
    assert_eq!(model._get_text("A6"), "#VALUE!");
}

#[test]
fn test_multinomial_huge_arguments_are_bounded() {
    // These looped once per unit of the argument (or overflowed the u64 sum) before T-1217.
    let start = std::time::Instant::now();
    let mut model = new_empty_model();
    model._set("A1", "=MULTINOMIAL(1E+15)");
    model._set("A2", "=MULTINOMIAL(1E+300)");
    model._set("A3", "=MULTINOMIAL(2^63,2^63)");
    model._set("A4", "=MULTINOMIAL(1E+307,1E+307,1E+307)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), "#NUM!", "{cell}");
    }
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
}
