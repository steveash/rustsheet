//! The element-wise text functions check the size of their broadcast result before building it,
//! as `&` and ROUND do (T-1771, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel
//! (expected, not measured) tries to build the array; we refuse one above a whole column
//! (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells").

use crate::test::util::new_empty_model;

#[test]
fn column_against_row_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // A filled column, so the result does not depend on how much of the sheet is used.
    for row in 1..=2000 {
        model._set(&format!("A{row}"), "abc");
    }
    // 2,000 x 1,000 = 2,000,000 cells. LEFT's text fits the 64 MiB text budget and FIND's
    // numbers do not count against it, so only the array limit stops them.
    // The cells are below row 2000 so they sit outside both referenced ranges (no circularity).
    model._set("C3001", "=ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))");
    model._set("C3002", "=ROWS(FIND(\"a\",A1:A2000,COLUMN(A1:ALL1)))");
    model._set("C3003", "=ROWS(MID(A1:A2000,COLUMN(A1:ALL1),1))");
    model._set("C3004", "=ROWS(CONCATENATE(A1:A2000,COLUMN(A1:ALL1)))");
    model._set("C3005", "=ROWS(REPT(A1:A2000,COLUMN(A1:ALL1)))");

    model.evaluate();

    for cell in ["C3001", "C3002", "C3003", "C3004", "C3005"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    // 2^20 x 2^14 = 2^34 cells; out of column A and row 1 (where they would be circular).
    model._set("C3", "=ROWS(LEFT(A:A,1:1))");
    model._set("C4", "=ROWS(FIND(\"a\",A:A,1:1))");
    model._set("C5", "=LEFT(A:A,1:1)");

    model.evaluate();

    for cell in ["C3", "C4", "C5"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn broadcast_under_the_limit_is_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "abc");
    model._set("A2", "xyz");
    // A column of 2 against a row of 3 (and a whole column against one column, 2^20 cells).
    model._set("E1", "=LEFT(A1:A2,COLUMN(A1:C1))");
    model._set("E4", "=ROWS(LEN(A:A))");

    model.evaluate();

    assert_eq!(model._get_text("E1"), *"a");
    assert_eq!(model._get_text("G1"), *"abc");
    assert_eq!(model._get_text("F2"), *"xy");
    assert_eq!(model._get_text("E4"), *"1048576");
}

#[test]
fn text_budget_below_the_limit_stays_value() {
    let mut model = new_empty_model();
    // 3,000 cells is under the array limit, but 3,000 x 32,767 bytes is over the 64 MiB text
    // budget (T-0232), so the budget's own `#VALUE!` still applies: the size check only adds the
    // `#ERROR!` above 1,048,576 cells.
    model._set("C1", "=REPT(\"x\",32767+0*SEQUENCE(3000))");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#VALUE!");
}
