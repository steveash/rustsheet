//! HSTACK and VSTACK check the size of their padded result before building it, as `&` and the
//! element-wise text functions do (T-1780, CLAUDE.md directive 5). DEVIATION (decision 0014):
//! Excel (expected, not measured) builds the array; we refuse one above a whole column
//! (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells").
#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn column_next_to_row_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // A filled column, so the result does not depend on how much of the sheet is used.
    for row in 1..=2000 {
        model._set(&format!("A{row}"), "1");
    }
    // HSTACK pads 2,000 x 1,001 = 2,002,000 cells and VSTACK 2,001 x 1,000 = 2,001,000.
    // The cells are below row 2000, outside both referenced ranges (no circularity).
    model._set("C3001", "=ROWS(HSTACK(A1:A2000,A1:ALL1))");
    model._set("C3002", "=ROWS(VSTACK(A1:A2000,A1:ALL1))");
    model._set("C3003", "=COLUMNS(HSTACK(A1:A2000,A1:ALL1))");

    model.evaluate();

    for cell in ["C3001", "C3002", "C3003"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    // Out of column A and row 1 (where they would be circular). The stacked shape is one past
    // the grid (2^20 + 1 rows, or 2^14 + 1 columns), so `check_array_size` answers `#VALUE!`
    // (past the grid), not `#ERROR!` (a shape inside the grid but above 2^20 cells).
    model._set("C3", "=ROWS(VSTACK(A:A,1:1))");
    model._set("C4", "=ROWS(HSTACK(A:A,1:1))");
    model._set("C5", "=HSTACK(A:A,1:1)");

    model.evaluate();

    for cell in ["C3", "C4", "C5"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
}

#[test]
fn stacks_under_the_limit_are_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "3");
    model._set("B2", "4");
    model._set("B3", "5");
    // Padding with #N/A below the limit still applies; a whole column fits exactly (2^20).
    model._set("D1", "=HSTACK(A1:A2,B1:B3)");
    model._set("H1", "=VSTACK(A1:A2,B1:C1)");
    model._set("K1", "=ROWS(HSTACK(A:A))");
    // Two whole columns are 2^21 cells: above the limit, inside the grid.
    model._set("K2", "=ROWS(HSTACK(A:A,B:B))");

    model.evaluate();

    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("E1"), "3");
    assert_eq!(model._get_text("E3"), "5");
    assert_eq!(model._get_text("D3"), "#N/A");
    assert_eq!(model._get_text("H1"), "1");
    assert_eq!(model._get_text("I1"), "#N/A");
    assert_eq!(model._get_text("H3"), "3");
    assert_eq!(model._get_text("I3"), "0");
    assert_eq!(model._get_text("K1"), "1048576");
    assert_eq!(model._get_text("K2"), "#ERROR!");
}
