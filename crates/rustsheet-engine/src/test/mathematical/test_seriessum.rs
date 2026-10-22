#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_seriessum_basic() {
    let mut model = new_empty_model();
    // SERIESSUM(2, 1, 1, {1, 2, 3})
    // = 1*2^1 + 2*2^2 + 3*2^3 = 2 + 8 + 24 = 34
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=SERIESSUM(2,1,1,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "34");
}

#[test]
fn test_seriessum_geometric_series() {
    // Geometric series: SERIESSUM(0.5, 0, 1, {1,1,1,1}) = 1 + 0.5 + 0.25 + 0.125 = 1.875
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "1");
    model._set("A3", "1");
    model._set("A4", "1");
    model._set("B1", "=SERIESSUM(0.5,0,1,A1:A4)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1.875");
}

#[test]
fn test_seriessum_single_coeff() {
    let mut model = new_empty_model();
    // SERIESSUM(3, 2, 0, {5}) = 5 * 3^2 = 45
    model._set("A1", "5");
    model._set("B1", "=SERIESSUM(3,2,0,A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "45");
}

#[test]
fn test_seriessum_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SERIESSUM(1,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_seriessum_x1_n0_m1() {
    let mut model = new_empty_model();
    // SERIESSUM(1, 0, 1, {1, 1, 1}) = 1*1^0 + 1*1^1 + 1*1^2 = 1+1+1 = 3
    model._set("A1", "1");
    model._set("A2", "1");
    model._set("A3", "1");
    model._set("B1", "=SERIESSUM(1,0,1,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}

#[test]
fn test_seriessum_negative_base_fractional_exponent_is_a_zero_term() {
    // x=-2 with fractional exponent 0.5: the term is 0, never NaN (Excel: `SERIESSUM(-2,0.5,1,
    // Data!A2:B2)` is 0, probes/seriessum-negative-x-fractional-power A12, T-2380).
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=SERIESSUM(-2,0.5,1,A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "0");
}

#[test]
fn test_seriessum_overflow_num_error() {
    // Very large base and exponent → overflow to Inf → must return #NUM!, not Inf.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=SERIESSUM(1E300,1000,1,A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#NUM!");
}

#[test]
fn test_seriessum_drops_blank_coefficient_cells() {
    // Excel: B2 is skipped and B4 takes power 1, so 42 + 2.5*97^0.5 + 2*97 (gen-m5/34f69d858a39f497
    // Calc!A170, 260.6221445044903); reading the blank as a 0 term would give 2*97^1.5.
    let mut model = new_empty_model();
    model._set("B1", "42");
    model._set("B3", "2.5");
    model._set("B4", "2");
    model._set("C1", "=SERIESSUM(97,0,0.5,B1:B4)");
    model._set("C2", "=SERIESSUM(97,0,0.5,{42,2.5,2})");
    model.evaluate();
    assert_eq!(model._get_text("C1"), model._get_text("C2"));
    assert_eq!(model._get_text("C1"), "260.6221445");
}

#[test]
fn test_seriessum_overflowing_term_beats_a_later_text_cell() {
    // Excel: the first term overflows, so #NUM! before the text in A2 is read
    // (gen-m5/6b831f5e2a20bc34 Calc!A130).
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "x");
    model._set("B1", "=SERIESSUM(64,1E+15,0,A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#NUM!");
}

#[test]
fn negative_x_to_a_non_integer_power_is_a_zero_term() {
    // probes/seriessum-negative-x-fractional-power (excel-graph, T-2380).
    let mut model = new_empty_model();
    model._set("A1", "=SERIESSUM(-2,0.5,1,{1,2})");
    model._set("A2", "=SERIESSUM(-2,1,0.5,{1,1,1})");
    model._set("A3", "=SERIESSUM(-2,0.5,1,{1,\"a\"})");
    model._set("A4", "=SERIESSUM(-10,0.5,399.5,{1,1,\"a\"})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}
