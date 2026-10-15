#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── MAP ───────────────────────────────────────────────────────────────────────

#[test]
fn map_double_each_element() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=MAP(A1:A3, LAMBDA(x, x * 2))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "4");
    assert_eq!(model._get_text("B3"), "6");
    assert_eq!(model._get_text("B4"), "");
}

#[test]
fn map_add_two_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");
    model._set("C1", "=MAP(A1:A3, B1:B3, LAMBDA(a, b, a + b))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "11");
    assert_eq!(model._get_text("C2"), "22");
    assert_eq!(model._get_text("C3"), "33");
    assert_eq!(model._get_text("C4"), "");
}

#[test]
fn map_2d_array() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "=MAP(A1:B2, LAMBDA(x, x * x))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("E1"), "4");
    assert_eq!(model._get_text("D2"), "9");
    assert_eq!(model._get_text("E2"), "16");
    assert_eq!(model._get_text("D3"), "");
}

#[test]
fn map_mismatched_array_sizes_is_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "10");
    // A1:A2 is 2×1, B1:B1 is 1×1 — mismatch
    model._set("C1", "=MAP(A1:A2, B1:B1, LAMBDA(a, b, a + b))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
}

#[test]
fn map_too_few_args() {
    let mut model = new_empty_model();
    model._set("A1", "=MAP(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── REDUCE ────────────────────────────────────────────────────────────────────

#[test]
fn reduce_sum_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=REDUCE(0, A1:A3, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "6");
}

#[test]
fn reduce_product_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("B1", "=REDUCE(1, A1:A3, LAMBDA(acc, x, acc * x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "24");
}

#[test]
fn reduce_without_initial_uses_first_element() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    // No initial value: starts at 10, adds 20, adds 30 → 60
    model._set("B1", "=REDUCE(A1:A3, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "60");
}

#[test]
fn reduce_max_with_initial() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A2", "2");
    model._set("A3", "8");
    model._set("A4", "1");
    model._set(
        "B1",
        "=REDUCE(0, A1:A4, LAMBDA(acc, x, IF(x > acc, x, acc)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "8");
}

#[test]
fn reduce_2d_array_row_major() {
    let mut model = new_empty_model();
    // 2×2 grid iterated row-major: 1, 2, 3, 4
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "=REDUCE(0, A1:B2, LAMBDA(acc, x, acc + x))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "10");
}

#[test]
fn reduce_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=REDUCE(0, A2:A3, LAMBDA(a,b,a+b), 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn map_scan_reduce_read_blank_and_error_cells_of_a_range_without_a_grid() {
    // T-1465: a one-cell reference is built on demand and read directly; a returned blank cell
    // is 0 and a cell error stays that error, as when the reference was read as an array.
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A3", "=1/0");
    model._set("C1", "=MAP(A1:A3,LAMBDA(v,v))");
    model._set("D1", "=SCAN(0,A1:A2,LAMBDA(a,v,a+v))");
    model._set("E1", "=REDUCE(0,A1:A2,LAMBDA(a,v,a+v))");
    model._set("F1", "=MAP(A1:A2,LAMBDA(v,ISREF(v)))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "5");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("C3"), "#DIV/0!");
    assert_eq!(model._get_text("D1"), "5");
    assert_eq!(model._get_text("D2"), "5");
    assert_eq!(model._get_text("E1"), "5");
    assert_eq!(model._get_text("F1"), "TRUE");
    assert_eq!(model._get_text("F2"), "TRUE");
}

#[test]
fn reduce_and_scan_walk_a_two_dimensional_range_row_by_row() {
    // T-1465: the element index is split into row and column, so a 2x2 range pins the order.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B1", "2"), ("A2", "3"), ("B2", "4")] {
        model._set(cell, value);
    }
    model._set("D1", "=REDUCE(0,A1:B2,LAMBDA(a,v,a*10+v))");
    model._set("E1", "=SCAN(0,A1:B2,LAMBDA(a,v,a*10+v))");
    model._set("H1", "=REDUCE(0,{1,2;3,4},LAMBDA(a,v,a*10+v))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1234");
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("F1"), "12");
    assert_eq!(model._get_text("E2"), "123");
    assert_eq!(model._get_text("F2"), "1234");
    assert_eq!(model._get_text("H1"), "1234");
}

// ── One patched body per call (T-1560) ───────────────────────────────────────

#[test]
fn map_inner_lambda_sees_each_element() {
    // The body makes a LAMBDA that holds the parameter, so each element patches a fresh body.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=MAP(A1:A3, LAMBDA(x, LAMBDA(y, x + y)(10)))");
    model._set(
        "C1",
        "=REDUCE(0, A1:A3, LAMBDA(acc, v, acc + LAMBDA(z, z * v)(2)))",
    );
    model._set("D1", "=SCAN(0, A1:A3, LAMBDA(acc, v, acc + v))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "11");
    assert_eq!(model._get_text("B2"), "12");
    assert_eq!(model._get_text("B3"), "13");
    assert_eq!(model._get_text("C1"), "12");
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("D2"), "3");
    assert_eq!(model._get_text("D3"), "6");
}

#[test]
fn map_wrong_lambda_arity_is_value_in_every_element() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "=MAP(A1:A2, LAMBDA(x, y, x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
}
