//! EXPAND, WRAPCOLS, WRAPROWS and MMULT check the size of their result before building it, as
//! HSTACK and VSTACK do (T-1786, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel
//! (expected, not measured) builds an in-grid array above a whole column, which we refuse with
//! `#ERROR!`; rows past the grid are `#NUM!` and columns past it are allowed, as for VSTACK and HSTACK
//! (probes/expand-wrap-past-grid; docs/excel-semantics/arrays.md "Stacked results past the grid").

use crate::test::util::new_empty_model;

#[test]
fn expand_rows_past_the_grid_are_num_columns_are_allowed_and_a_huge_wrap_count_clamps() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model._set(&format!("A{row}"), &row.to_string());
    }
    // 10^18 cells: rows past the grid.
    model._set("C1", "=EXPAND(1,1E9,1E9)");
    model._set("C8", "=COLUMNS(EXPAND(1,1,16385))");
    // A wrap_count above the vector's length is the length, as in Excel (arrays.md "WRAPROWS
    // and WRAPCOLS"), so a billion pads nothing: a 10x1 column and a 1x10 row.
    model._set("C2", "=ROWS(WRAPCOLS(A1:A10,1E9))");
    model._set("C3", "=COLUMNS(WRAPROWS(A1:A10,1E9))");
    // Inside the grid but above 2^20 cells: 2,000 x 1,000.
    model._set("C4", "=ROWS(EXPAND(1,2000,1000))");
    // Valid sizes keep their values.
    model._set("C5", "=ROWS(EXPAND(A1:A10,12,2))");
    model._set("C6", "=COLUMNS(WRAPROWS(A1:A10,4))");
    model._set("C7", "=ROWS(WRAPCOLS(A1:A10,4))");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C8"), *"16385");
    assert_eq!(model._get_text("C2"), *"10");
    assert_eq!(model._get_text("C3"), *"10");
    assert_eq!(model._get_text("C4"), *"#ERROR!");
    assert_eq!(model._get_text("C5"), *"12");
    assert_eq!(model._get_text("C6"), *"4");
    assert_eq!(model._get_text("C7"), *"4");
}

#[test]
fn mmult_product_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // A filled 2,000-row column and a filled 1,000-column row, so the product is numeric and
    // 2,000 x 1,000 = 2,000,000 cells. The formulas sit outside both ranges.
    for row in 1..=2000 {
        model._set(&format!("A{row}"), "1");
    }
    model._set("B1", "=SEQUENCE(1,1000)");
    model._set("C3001", "=ROWS(MMULT(A1:A2000,B1#))");
    // A blank cell in an input is `#VALUE!` first (arrays.md, MMULT reads no blanks).
    model._set("C3002", "=ROWS(MMULT(A1:A1048576,A1:B1))");
    model._set("C3003", "=SUM(MMULT(A1:A2,TRANSPOSE(A1:A2)))");

    model.evaluate();

    assert_eq!(model._get_text("C3001"), *"#ERROR!");
    assert_eq!(model._get_text("C3002"), *"#VALUE!");
    assert_eq!(model._get_text("C3003"), *"4");
}
