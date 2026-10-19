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
fn helpers_calling_defined_name_and_eta_lambdas_see_each_element() {
    // The body makes LAMBDAs that hold none of its parameters (a defined name, a function passed
    // by name, a LAMBDA made inside the called one), so it keeps its patch (T-0425).
    let mut model = new_empty_model();
    model
        .new_defined_name("Twice", None, "=LAMBDA(x, 2*x)")
        .unwrap();
    model
        .new_defined_name("AddTo", None, "=LAMBDA(a, LAMBDA(b, a + b)(10))")
        .unwrap();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=MAP(A1:A3, LAMBDA(x, Twice(x) + AddTo(x)))");
    model._set("C1", "=SCAN(0, A1:A3, LAMBDA(acc, v, acc + Twice(v)))");
    model._set(
        "D1",
        "=BYROW(A1:A3, LAMBDA(r, REDUCE(0, r, _xleta.SUM) + Twice(SUM(r))))",
    );
    model._set(
        "E1",
        "=MAP(A1:A3, LAMBDA(x, Twice(x) + LAMBDA(y, x * y)(100)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "13");
    assert_eq!(model._get_text("B2"), "16");
    assert_eq!(model._get_text("B3"), "19");
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("C2"), "6");
    assert_eq!(model._get_text("C3"), "12");
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("D2"), "6");
    assert_eq!(model._get_text("D3"), "9");
    assert_eq!(model._get_text("E1"), "102");
    assert_eq!(model._get_text("E2"), "204");
    assert_eq!(model._get_text("E3"), "306");
}

#[test]
fn defined_name_lambda_calls_reuse_their_body_and_stay_independent() {
    // A defined-name LAMBDA keeps one lambda id and its patched body across calls in one
    // evaluation (T-0440): recursion, a call in its own argument, a wrong argument count, the
    // name passed by value and a changed definition must each still see their own values.
    let mut model = new_empty_model();
    model
        .new_defined_name("Twice", None, "=LAMBDA(x, 2*x)")
        .unwrap();
    model
        .new_defined_name("Fact", None, "=LAMBDA(n, IF(n<=1, 1, n*Fact(n-1)))")
        .unwrap();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set(
        "B1",
        "=MAP(A1:A3, LAMBDA(x, Twice(Twice(x)) + Fact(x + 2)))",
    );
    model._set("C1", "=MAP(A1:A3, Twice)");
    model._set("D1", "=Twice(1, 2)");
    model._set("D2", "=Twice(5) + Twice(A3)");
    model._set("E1", "=Fact(5) + Fact(3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("B2"), "32");
    assert_eq!(model._get_text("B3"), "132");
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("C3"), "6");
    assert_eq!(model._get_text("D1"), "#VALUE!");
    assert_eq!(model._get_text("D2"), "16");
    assert_eq!(model._get_text("E1"), "126");
    model
        .update_defined_name("Twice", None, "Twice", None, "=LAMBDA(x, 3*x)")
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C3"), "9");
    assert_eq!(model._get_text("D2"), "24");
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

// ── LAMBDAs that outlive their call (T-0439) ─────────────────────────────────

#[test]
fn closures_read_the_values_their_outer_names_had_when_made() {
    // probes/lambda-escaping-closures, excel-graph: an accumulator LAMBDA made in each REDUCE
    // step, a LET-bound curried call, and a captured name shadowed or rebound later.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set(
        "B1",
        "=LET(g,REDUCE(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)*10+x))),g(0))",
    );
    model._set("B2", "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),plus,mk(5),plus(1))");
    model._set(
        "B3",
        "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),fa,mk(1),fb,mk(100),fa(0)+fb(0)*1000)",
    );
    model._set("B4", "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f,mk(1),a,50,f(0))");
    model._set("B5", "=LET(a,1,f,LAMBDA(b,a+b),LET(a,2,f(0)))");
    model._set(
        "B6",
        "=LET(g,REDUCE(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)+x))),MAP({10,20},g))",
    );
    model._set(
        "B8",
        "=LET(x,10,mk,LAMBDA(a,LAMBDA(b,a+b+x)),f,mk(1),f(100))",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "123");
    assert_eq!(model._get_text("B2"), "6");
    assert_eq!(model._get_text("B3"), "100001");
    assert_eq!(model._get_text("B4"), "1");
    assert_eq!(model._get_text("B5"), "1");
    assert_eq!(model._get_text("B6"), "16");
    assert_eq!(model._get_text("C6"), "26");
    assert_eq!(model._get_text("B8"), "111");
}

#[test]
fn closures_keep_captured_references_unions_and_errors() {
    // probes/lambda-escaping-closures H11, H12, H19, H23, H24: a captured reference stays one.
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set(
        "B1",
        "=LET(mk,LAMBDA(r,LAMBDA(i,INDEX(r,i))),f,mk(A1:A3),f(2))",
    );
    model._set(
        "B2",
        "=LET(mk,LAMBDA(r,LAMBDA(z,ISREF(r))),f,mk(A1:A3),f(0))",
    );
    model._set("B3", "=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f,mk(1/0),f(1))");
    model._set(
        "B4",
        "=LET(mk,LAMBDA(r,LAMBDA(z,AREAS(r))),f,mk((A1,A2)),f(0))",
    );
    model._set(
        "B5",
        "=LET(mk,LAMBDA(r,LAMBDA(z,SUM(r)+z)),f,mk((A1,A3)),f(0))",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "TRUE");
    assert_eq!(model._get_text("B3"), "#DIV/0!");
    assert_eq!(model._get_text("B4"), "2");
    assert_eq!(model._get_text("B5"), "4");
}

#[test]
fn a_closure_called_many_times_and_recursively_keeps_its_captures() {
    // Each call binds what the closure captured and unbinds it after, so the next call (MAP over
    // 1,000 elements) and a call of the same closure inside one (through REDUCE's chain) see it.
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=LET(mk,LAMBDA(k,LAMBDA(v,v*k)),f,mk(3),SUM(MAP(SEQUENCE(1000),f)))",
    );
    model._set(
        "A2",
        "=LET(g,REDUCE(LAMBDA(z,z),SEQUENCE(50),LAMBDA(f,x,LAMBDA(z,f(z)+x))),g(0)+g(1))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1501500");
    assert_eq!(model._get_text("A2"), "2551");
}

#[test]
fn closures_escape_through_nested_scopes() {
    // A LAMBDA a LET value or body makes inside a call, and one an escaped closure makes, capture
    // the variables of every scope they leave (lexical, as probes/lambda-escaping-closures shows).
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=LET(mk,LAMBDA(a,LET(f,LAMBDA(b,a+b),f)),g,mk(3),g(4))",
    );
    model._set(
        "A2",
        "=LET(mk,LAMBDA(a,LET(x,a*2,LAMBDA(b,x+b))),f,mk(5),f(1))",
    );
    model._set(
        "A3",
        "=LET(mk,LAMBDA(a,LAMBDA(b,LAMBDA(c,a+b+c))),f,mk(1),g,f(10),g(100))",
    );
    model._set("A4", "=LET(x,1,f,LAMBDA(b,x+b),g,LET(y,f,y),g(2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
    assert_eq!(model._get_text("A2"), "11");
    assert_eq!(model._get_text("A3"), "111");
    assert_eq!(model._get_text("A4"), "3");
}

#[test]
fn lambdas_that_do_not_outlive_their_call_capture_nothing() {
    // A LAMBDA made on each of 2,000 REDUCE steps reads the 10,000-row accumulator but never
    // leaves its step, so nothing is copied for it (capturing at creation kept 2,000 copies).
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=ROWS(REDUCE(SEQUENCE(10000),SEQUENCE(2000),LAMBDA(a,x,IF(SUM(MAP({1},LAMBDA(z,ROWS(a))))>0,a,a))))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "10000");
}

#[test]
fn lambda_calls_past_the_stack_budget_are_num_errors() {
    // DEVIATION (decision 0014): Excel evaluates these 300 deep (probes/lambda-call-depth); past
    // the evaluation's stack budget the engine gives #NUM! instead of overflowing its stack.
    let mut model = new_empty_model();
    model._set("A1", "=LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,20000))");
    model._set(
        "A2",
        "=LET(g,REDUCE(LAMBDA(z,z),SEQUENCE(20000),LAMBDA(f,x,LAMBDA(z,f(z)+x))),g(0))",
    );
    model._set("A3", "=LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,10))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "10");
}
