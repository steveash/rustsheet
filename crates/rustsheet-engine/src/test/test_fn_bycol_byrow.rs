#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── BYCOL ─────────────────────────────────────────────────────────────────────

#[test]
fn bycol_sum_each_column() {
    let mut model = new_empty_model();
    // 2×3 grid
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    // SUM of each column: 5, 7, 9
    model._set("A4", "=BYCOL(A1:C2, LAMBDA(col, SUM(col)))");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "5");
    assert_eq!(model._get_text("B4"), "7");
    assert_eq!(model._get_text("C4"), "9");
    assert_eq!(model._get_text("D4"), "");
}

#[test]
fn bycol_max_each_column() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("B1", "1");
    model._set("A2", "7");
    model._set("B2", "4");
    model._set("A4", "=BYCOL(A1:B2, LAMBDA(col, MAX(col)))");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "7");
    assert_eq!(model._get_text("B4"), "4");
    assert_eq!(model._get_text("C4"), "");
}

#[test]
fn bycol_single_column() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "=BYCOL(A1:A3, LAMBDA(col, SUM(col)))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "60");
    assert_eq!(model._get_text("C1"), "");
}

#[test]
fn bycol_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=BYCOL(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── BYROW ─────────────────────────────────────────────────────────────────────

#[test]
fn byrow_sum_each_row() {
    let mut model = new_empty_model();
    // 3×2 grid
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("A3", "5");
    model._set("B3", "6");
    // SUM of each row: 3, 7, 11
    model._set("D1", "=BYROW(A1:B3, LAMBDA(row, SUM(row)))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("D2"), "7");
    assert_eq!(model._get_text("D3"), "11");
    assert_eq!(model._get_text("D4"), "");
}

#[test]
fn byrow_min_each_row() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("B1", "2");
    model._set("A2", "8");
    model._set("B2", "3");
    model._set("D1", "=BYROW(A1:B2, LAMBDA(row, MIN(row)))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("D2"), "3");
    assert_eq!(model._get_text("D3"), "");
}

#[test]
fn byrow_single_row() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A3", "=BYROW(A1:C1, LAMBDA(row, SUM(row)))");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "60");
    assert_eq!(model._get_text("A4"), "");
}

// Regression: a single-character LAMBDA parameter named `c` (or `r`) must survive
// the round-trip through the internal R1C1 storage. Formulas are stored in R1C1
// format, and `c`/`r` used to be rejected as invalid R1C1 references on reload,
// turning the formula into a parse error that displayed the raw R1C1 string.
#[test]
fn byrow_single_char_param_displays_a1() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    // Placed in B22 so the stored R1C1 form is BYROW(R[-21]C[-1]:R[-21]C[1],...).
    model._set("B22", "=BYROW(A1:C1, LAMBDA(c, MAX(c)))");
    model.evaluate();
    assert_eq!(model._get_text("B22"), "30");
    // The displayed formula must remain in A1 notation, not the raw R1C1 string.
    assert_eq!(model._get_formula("B22"), "=BYROW(A1:C1,LAMBDA(c,MAX(c)))");
}

#[test]
fn byrow_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=BYROW(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── MAKEARRAY ─────────────────────────────────────────────────────────────────

#[test]
fn makearray_multiplication_table() {
    let mut model = new_empty_model();
    model._set("A1", "=MAKEARRAY(3, 3, LAMBDA(x, y, x * y))");
    model.evaluate();
    // Row 1: 1*1, 1*2, 1*3
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("C1"), "3");
    // Row 2: 2*1, 2*2, 2*3
    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("B2"), "4");
    assert_eq!(model._get_text("C2"), "6");
    // Row 3: 3*1, 3*2, 3*3
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("B3"), "6");
    assert_eq!(model._get_text("C3"), "9");
    assert_eq!(model._get_text("D1"), "");
}

#[test]
fn makearray_row_indices() {
    let mut model = new_empty_model();
    // 4×1 column of row indices
    model._set("A1", "=MAKEARRAY(4, 1, LAMBDA(x, y, x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("A4"), "4");
    assert_eq!(model._get_text("A5"), "");
}

#[test]
fn makearray_single_cell() {
    let mut model = new_empty_model();
    model._set("A1", "=MAKEARRAY(1, 1, LAMBDA(x, y, 42))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "42");
    assert_eq!(model._get_text("A2"), "");
    assert_eq!(model._get_text("B1"), "");
}

#[test]
fn makearray_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MAKEARRAY(3, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn makearray_invalid_rows() {
    let mut model = new_empty_model();
    model._set("A1", "=MAKEARRAY(0, 3, LAMBDA(x, y, x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn makearray_invalid_cols() {
    let mut model = new_empty_model();
    model._set("A1", "=MAKEARRAY(3, 0, LAMBDA(x, y, y))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn bycol_lambda_counta() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "z");
    model._set("B1", "x");
    model._set("C1", "y");
    model._set("C2", "z");
    model._set("G1", "=BYCOL(A1:C3, LAMBDA(col, COUNTA(col)))");
    model.evaluate();
    assert_eq!(model._get_text("G1"), "3");
    assert_eq!(model._get_text("H1"), "1");
    assert_eq!(model._get_text("I1"), "2");
}

#[test]
fn bycol_range_slices_are_references() {
    // probes/lambda-helper-range-slices (excel-graph): blank cells of a column slice are skipped.
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A3", "z");
    model._set("B3", "y");
    model._set("E1", "=BYCOL(A1:B3,LAMBDA(c,COUNTA(c)))");
    model._set("E3", "=BYROW(A1:B3,LAMBDA(r,ROW(r)))");
    model._set("G1", "=BYCOL(IFERROR(A1:B3,1),LAMBDA(c,COUNTA(c)))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("F1"), "1");
    assert_eq!(model._get_text("E5"), "3");
    assert_eq!(model._get_text("G1"), "3");
    assert_eq!(model._get_text("H1"), "3");
}

#[test]
fn byrow_bycol_whole_column_keeps_full_size() {
    // A whole column keeps its 1,048,576 rows, not the used area (arrays.md "Array results
    // above 1,048,576 cells").
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("D1", "=BYCOL(A:A,LAMBDA(c,ROWS(c)))");
    model._set("D2", "=ROWS(BYROW(C:C,LAMBDA(r,1)))");
    // Two whole columns are above the 2^20-cell array bound: the documented DEVIATION.
    model._set("D3", "=BYCOL(A:B,LAMBDA(c,ROWS(c)))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1048576");
    assert_eq!(model._get_text("D2"), "1048576");
    assert_eq!(model._get_text("D3"), "#ERROR!");
}

#[test]
fn byrow_over_a_range_above_the_array_bound_is_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "=BYROW(B1:XFD1048575,LAMBDA(r,COUNTA(r)))");
    model._set("A2", "=BYCOL(B1:XFD1048575,LAMBDA(c,COUNTA(c)))");
    model.evaluate();
    assert!(model._get_text("A1").starts_with('#'));
    assert!(model._get_text("A2").starts_with('#'));
}

#[test]
fn byrow_bycol_need_a_lambda() {
    // Anything but a LAMBDA is `#VALUE!`, an error and an unknown name too
    // (probes/eta-reduced-names E5:E21).
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("D1", "=BYCOL(A1:B3,NA())");
    model._set("D2", "=BYCOL(A1:B3,1)");
    model._set("D3", "=BYCOL(A1:B3,COUNTA)");
    model._set("D4", "=BYROW(A1:B3,_xleta.NONSUCH)");
    model.evaluate();
    for cell in ["D1", "D2", "D3", "D4"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}

#[test]
fn functions_passed_by_name() {
    // `_xleta.NAME` takes as many arguments as the helper passes (probes/eta-reduced-names,
    // probes/m7-argument-modes F28:H51).
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A3", "z");
    model._set("B3", "y");
    model._set("D1", "=BYCOL(A1:B3,_xleta.COUNTA)");
    model._set("D3", "=BYROW(A1:B3,_xleta.countblank)");
    model._set("D7", "=MAKEARRAY(2,2,_xleta.POWER)");
    model._set("D9", "=LET(f,_xleta.SUM,f(1,2))");
    model._set("D10", "=MAP(A1:A3,_xleta.ISREF)");
    model._set("D13", "=BYROW({1,2;3,4},_xleta._xlfn.STDEV.S)");
    model._set("D15", "=SCAN(0,A1:A3,_xleta.COUNTA)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("D3"), "1");
    assert_eq!(model._get_text("D4"), "2");
    assert_eq!(model._get_text("D5"), "0");
    assert_eq!(model._get_text("E7"), "1");
    assert_eq!(model._get_text("E8"), "4");
    assert_eq!(model._get_text("D9"), "3");
    assert_eq!(model._get_text("D10"), "TRUE");
    assert_eq!(model._get_text("D12"), "TRUE");
    assert_eq!(model._get_text("D13"), "0.707106781");
    assert_eq!(model._get_text("D15"), "2");
    assert_eq!(model._get_text("D16"), "1");
}

#[test]
fn functions_passed_by_name_with_special_forms_do_not_panic() {
    // Functions the parser or evaluator treats specially still evaluate to some value.
    let mut model = new_empty_model();
    model._set("A1", "1");
    let formulas = [
        "=BYROW(A1:B3,_xleta.LET)",
        "=MAKEARRAY(2,2,_xleta.LET)",
        "=BYROW(A1:B3,_xleta.LAMBDA)",
        "=MAKEARRAY(2,2,_xleta.LAMBDA)",
        "=MAP(A1:A3,_xleta.SUBTOTAL)",
        "=MAKEARRAY(2,2,_xleta.SUBTOTAL)",
        "=BYROW(A1:B3,_xleta.AREAS)",
        "=MAKEARRAY(2,2,_xleta.OFFSET)",
        "=MAP(A1:A3,_xleta.NOW)",
        "=REDUCE(0,A1:A3,_xleta.INDIRECT)",
        "=_xleta.SUM",
        "=_xleta.",
    ];
    for (row, formula) in formulas.iter().enumerate() {
        model._set(&format!("D{}", 4 * row + 1), formula);
    }
    model.evaluate();
    for row in 0..formulas.len() {
        let _ = model._get_text(&format!("D{}", 4 * row + 1));
    }
}
