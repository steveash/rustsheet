#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── SCAN ──────────────────────────────────────────────────────────────────────

#[test]
fn scan_running_sum_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    // SCAN(0, A1:A3, LAMBDA(acc, x, acc + x)) → [1, 3, 6]
    model._set("B1", "=SCAN(0, A1:A3, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "6");
    assert_eq!(model._get_text("B4"), "");
}

#[test]
fn scan_running_product_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "3");
    model._set("A3", "4");
    // SCAN(1, A1:A3, LAMBDA(acc, x, acc * x)) → [2, 6, 24]
    model._set("B1", "=SCAN(1, A1:A3, LAMBDA(acc, x, acc * x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "6");
    assert_eq!(model._get_text("B3"), "24");
    assert_eq!(model._get_text("B4"), "");
}

#[test]
fn scan_running_max_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "1");
    model._set("A3", "4");
    model._set("A4", "2");
    // Running max starting from 0 → [3, 3, 4, 4]
    model._set("B1", "=SCAN(0, A1:A4, LAMBDA(acc, x, IF(x > acc, x, acc)))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "4");
    assert_eq!(model._get_text("B4"), "4");
    assert_eq!(model._get_text("B5"), "");
}

#[test]
fn scan_without_initial_uses_first_element_as_seed() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    // Without initial: seed = 10 (output[0] = 10), then 10+20=30, 30+30=60 → [10, 30, 60]
    model._set("B1", "=SCAN(A1:A3, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "30");
    assert_eq!(model._get_text("B3"), "60");
    assert_eq!(model._get_text("B4"), "");
}

#[test]
fn scan_2d_array_row_major() {
    let mut model = new_empty_model();
    // 2×2 grid iterated row-major: 1, 2, 3, 4
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    // Running sum from 0: 1, 3, 6, 10
    model._set("D1", "=SCAN(0, A1:B2, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("D2"), "6");
    assert_eq!(model._get_text("E2"), "10");
    assert_eq!(model._get_text("D3"), "");
}

#[test]
fn scan_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SCAN(0, A2:A3, LAMBDA(a, b, a+b), 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn scan_too_few_args() {
    let mut model = new_empty_model();
    model._set("A1", "=SCAN(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn scan_moves_the_accumulator_through_many_steps() {
    // Each step moves the accumulator into the LAMBDA (T-1561); every slot keeps its own copy.
    let mut model = new_empty_model();
    model._set("A1", "=SCAN(0, SEQUENCE(1000), LAMBDA(a, x, a + x))");
    model._set("B1", "=SUM(A1#)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A1000"), "500500");
    assert_eq!(model._get_text("B1"), "167167000");
}

// ── errors reach the LAMBDA (T-2306) ─────────────────────────────────────────

#[test]
fn scan_error_initial_value_is_passed_to_the_lambda() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    // COUNT skips the error accumulator: 1, then 2 (gen-m7/90de8755a2e565e6 A1223)
    model._set("B1", "=SCAN(NA(), A1:A2, LAMBDA(acc, x, COUNT(acc, x)))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
}

#[test]
fn scan_error_step_fills_its_slot_and_the_scan_goes_on() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "0");
    model._set("A3", "2");
    model._set("B1", "=SCAN(5, A1:A3, LAMBDA(acc, x, 1 / x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "#DIV/0!");
    assert_eq!(model._get_text("B3"), "0.5");
}

#[test]
fn scan_multi_cell_array_step_is_one_calc_error() {
    let mut model = new_empty_model();
    for (c, v) in [("C1", "1"), ("D1", "2"), ("C2", "3"), ("D2", "4")] {
        model._set(c, v);
    }
    model._set("F1", "=SCAN(1, A1:B2, LAMBDA(y, acc, TRIMRANGE(C1:D2)))");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#CALC!");
    assert_eq!(model._get_text("G1"), "");
    assert_eq!(model._get_text("F2"), "");
}

// A 1x1 array step or initial_value is its one value (T-2407).
#[test]
fn scan_takes_1x1_arrays_as_one_value() {
    let mut model = new_empty_model();
    model._set("A1", "=SCAN({3},{1;2},LAMBDA(a,x,a+x))");
    model._set("C1", "=SCAN(0,{1;2},LAMBDA(a,x,{5}))");
    model._set("E1", "=SCAN(0,{1;2},LAMBDA(a,x,{5,6}))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "6");
    assert_eq!(model._get_text("C1"), "5");
    assert_eq!(model._get_text("C2"), "5");
    assert_eq!(model._get_text("E1"), "#CALC!");
}
