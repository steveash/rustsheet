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
fn map_mismatched_array_sizes_pad_with_na() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "10");
    // A1:A2 is 2×1, B1:B1 is 1×1: the shorter array pads with #N/A, nothing broadcasts
    // (probes/map-unequal-sizes, arrays.md "MAP over arrays of different sizes").
    model._set("C1", "=MAP(A1:A2, B1:B1, LAMBDA(a, b, a + b))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "11");
    assert_eq!(model._get_text("C2"), "#N/A");
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
fn by_slices_and_makearray_inner_lambda_sees_each_call() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=BYROW(A1:A3, LAMBDA(r, LAMBDA(y, SUM(r) + y)(10)))");
    model._set("C1", "=BYCOL({1,2,3}, LAMBDA(c, LAMBDA(y, SUM(c) * y)(2)))");
    model._set(
        "C3",
        "=MAKEARRAY(2, 2, LAMBDA(i, j, LAMBDA(k, i * 10 + j + k)(0)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "11");
    assert_eq!(model._get_text("B2"), "12");
    assert_eq!(model._get_text("B3"), "13");
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("D1"), "4");
    assert_eq!(model._get_text("E1"), "6");
    assert_eq!(model._get_text("C3"), "11");
    assert_eq!(model._get_text("D3"), "12");
    assert_eq!(model._get_text("C4"), "21");
    assert_eq!(model._get_text("D4"), "22");
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

#[test]
fn reduce_with_an_array_accumulator() {
    // Each step moves the growing accumulator into the LAMBDA (T-1561).
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=REDUCE(0, SEQUENCE(3), LAMBDA(a, x, VSTACK(a, x * 10)))",
    );
    model._set(
        "B1",
        "=ROWS(REDUCE(0, SEQUENCE(500), LAMBDA(a, x, VSTACK(a, x))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "0");
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "20");
    assert_eq!(model._get_text("A4"), "30");
    assert_eq!(model._get_text("A5"), "");
    assert_eq!(model._get_text("B1"), "501");
}

#[test]
fn a_parameter_read_once_is_moved_and_one_read_more_often_is_copied() {
    // T-1588: a parameter the body reads once, as a VSTACK/HSTACK argument or as the body
    // itself, is moved into that read; every other read copies it, so each case below still
    // sees the value on every read.
    let mut model = new_empty_model();
    // Read twice: the accumulator doubles each step, 1 -> 2 -> 4 -> 8 rows.
    model._set(
        "A1",
        "=ROWS(REDUCE(0, SEQUENCE(3), LAMBDA(a, x, VSTACK(a, a))))",
    );
    // Read once in the text, but inside a nested LAMBDA that MAP calls three times.
    model._set(
        "B1",
        "=SUM(MAP(SEQUENCE(2), LAMBDA(a, SUM(MAP(SEQUENCE(3), LAMBDA(y, a))))))",
    );
    // Read once, through groups and nested stacks.
    model._set(
        "C1",
        "=SUM(LAMBDA(a, (VSTACK(HSTACK((a), 1), {2,3})))({5}))",
    );
    // Read once as the whole body; and a parameter called twice.
    model._set("D1", "=SUM(LAMBDA(a, a)({1;2;3}))");
    model._set(
        "E1",
        "=SUM(LAMBDA(f, VSTACK(f(2), f(3)))(LAMBDA(x, x * 3)))",
    );
    // Read once by a function that is not a stack, and once more by VSTACK.
    model._set("F1", "=SUM(LAMBDA(a, VSTACK(a, SUM(a)))({1;2}))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "8");
    assert_eq!(model._get_text("B1"), "9");
    assert_eq!(model._get_text("C1"), "11");
    assert_eq!(model._get_text("D1"), "6");
    assert_eq!(model._get_text("E1"), "15");
    assert_eq!(model._get_text("F1"), "6");
}

#[test]
fn a_parameter_read_once_through_if_let_sort_take_drop_is_moved_and_one_read_twice_is_copied() {
    // T-1593: IF (its condition, and one read in each exclusive branch), LET (values and body)
    // and the array of SORT, TAKE and DROP read a parameter once; each case below that reads it
    // a second time still sees the value on every read.
    let mut model = new_empty_model();
    // IF: a read in each branch; a read in the condition and a branch; a branch and outside.
    model._set(
        "A1",
        "=ROWS(REDUCE(0, SEQUENCE(5), LAMBDA(a, x, IF(x > 2, VSTACK(a, x), a))))",
    );
    model._set("A2", "=SUM(LAMBDA(a, IF(SUM(a) > 0, a, 0))({1;2}))");
    model._set("A3", "=SUM(LAMBDA(a, VSTACK(IF(TRUE, a, 0), a))({1;2}))");
    // IF over an array condition evaluates both branches, each reading the parameter.
    model._set(
        "A4",
        "=SUM(LAMBDA(a, IF({TRUE,FALSE}, a, VSTACK(a)))({1;2}))",
    );
    model._set(
        "A5",
        "=SUM(LAMBDA(a, IF({TRUE,FALSE}, VSTACK(a, 1), IF(TRUE, VSTACK(a, 0), 0)))({1;2}))",
    );
    // LET: the body; a value; a value and the body.
    model._set(
        "B1",
        "=ROWS(REDUCE(0, SEQUENCE(4), LAMBDA(a, x, LET(y, x * 2, VSTACK(a, y)))))",
    );
    model._set("B2", "=SUM(LAMBDA(a, LET(b, a, SUM(b) + 1))({1;2}))");
    model._set("B3", "=SUM(LAMBDA(a, LET(b, a, VSTACK(a, b)))({1;2}))");
    // SORT, TAKE and DROP: the array once; the array and another argument.
    model._set(
        "C1",
        "=INDEX(REDUCE(0, SEQUENCE(4), LAMBDA(a, x, SORT(VSTACK(a, x), 1, -1))), 1)",
    );
    model._set(
        "C2",
        "=INDEX(LAMBDA(a, SORT(a, COLUMNS(a)))({3,1;2,4}), 1, 1)",
    );
    model._set("C3", "=SUM(LAMBDA(a, TAKE(a, 1))({1;2}))");
    model._set("C4", "=SUM(LAMBDA(a, TAKE(a, ROWS(a) - 1))({1;2;3}))");
    model._set("C5", "=SUM(LAMBDA(a, DROP(a, 1))({1;2;3}))");
    model._set("C6", "=SUM(LAMBDA(a, DROP(a, ROWS(a) - 1))({1;2;3}))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "6");
    assert_eq!(model._get_text("A4"), "6");
    assert_eq!(model._get_text("A5"), "7");
    assert_eq!(model._get_text("B1"), "5");
    assert_eq!(model._get_text("B2"), "4");
    assert_eq!(model._get_text("B3"), "6");
    assert_eq!(model._get_text("C1"), "4");
    assert_eq!(model._get_text("C2"), "3");
    assert_eq!(model._get_text("C3"), "1");
    assert_eq!(model._get_text("C4"), "3");
    assert_eq!(model._get_text("C5"), "5");
    assert_eq!(model._get_text("C6"), "3");
}

// ── A LAMBDA returning a multi-cell range (T-1769) ───────────────────────────

#[test]
fn map_whose_lambda_returns_a_tall_range_reads_only_the_used_area() {
    // Nightly fuzz timeout (formula_eval, T-1769), on the fuzz target's sheet: every element was
    // read as a 100,000-cell array before it became an error, so 1,000 calls took seconds.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=A1*2");
    model._set("C3", "=SUM(MAP(A1:A1000,LAMBDA(rSMU,((B1:B100000)))))");
    model._set("D3", "=ROWS(MAP(A1:A1000,LAMBDA(r,B1:B1048576)))");
    model._set("E3", "=SUM(SCAN(0,A1:A1000,LAMBDA(a,v,B2:C500000)))");
    model.evaluate();
    assert_eq!(model._get_text("C3"), "#CALC!");
    assert_eq!(model._get_text("D3"), "#CALC!");
    assert_eq!(model._get_text("E3"), "#CALC!");
}

#[test]
fn map_and_scan_elements_that_are_ranges_still_read_their_cells() {
    // The whole call is #CALC! whatever the range holds (probes/map-scan-range-result), but its
    // cells are still read: a range through the formula's own cell closes a cycle, and an error cell inside the range does not change the result.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "=1/0");
    model._set("C1", "=MAP(A1:A2,LAMBDA(r,A1:A2))");
    model._set("E1", "=SCAN(0,A1:A1,LAMBDA(a,v,A1:A2))");
    model._set("G1", "=MAP(A1:A2,LAMBDA(r,G1:G5))");
    model._set("H1", "=SCAN(0,A1:A2,LAMBDA(a,v,H1:H5))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#CALC!");
    assert_eq!(model._get_text("E1"), "#CALC!");
    assert_eq!(model._get_text("G1"), "#CIRC!");
    assert_eq!(model._get_text("H1"), "#CIRC!");
}
