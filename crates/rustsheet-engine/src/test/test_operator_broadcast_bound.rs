#![allow(clippy::unwrap_used, reason = "tests")]

//! A binary operator over two arrays checks the size of its broadcast result before building it
//! (T-1420, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel tries to build the array; we
//! refuse one above a whole column (docs/excel-semantics/arrays.md "Array results above 1,048,576
//! cells").

use crate::test::util::new_empty_model;

#[test]
fn whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    // The formula_eval fuzz reproducer (nightly run 38109900859), verbatim: the space before `>`
    // and the row range `$4:44` are the fuzzer's. 2^20 x 2^14 cells.
    model._set("C3", "=Data!$D:$D >Data!$4:44");
    // The smaller forms, out of column D and row 4 (where they would be circular).
    model._set("G1", "=D:D+4:4");
    model._set("G2", "=D:D>4:4");
    model._set("G3", "=D1:D100000>4:4");
    model._set("G5", "=SUM(D:D>4:4)");
    model._set("G6", "=D:D&4:4");
    model._set("G7", "=4:4^D:D");

    model.evaluate();

    for cell in ["C3", "G1", "G2", "G3", "G5", "G6", "G7"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn broadcast_under_the_limit_is_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "10");
    model._set("C1", "20");
    // A column of 2 against a row of 2 (and a column of 3 padded with #N/A).
    model._set("E1", "=A1:A2+B1:C1");
    model._set("E4", "=A1:A2&B1:C1");
    model._set("E6", "=A1:A2<B1:C1");
    model._set("E8", "=SUM(A1:A1048576*B1:C1)");
    model._set("H1", "=A1:A3+{1,2}");

    model.evaluate();

    assert_eq!(model._get_text("E1"), *"11");
    assert_eq!(model._get_text("F1"), *"21");
    assert_eq!(model._get_text("E2"), *"12");
    assert_eq!(model._get_text("F2"), *"22");
    assert_eq!(model._get_text("E4"), *"110");
    assert_eq!(model._get_text("F5"), *"220");
    assert_eq!(model._get_text("E6"), *"TRUE");
    // A whole column against two columns is 2^21 cells: above the limit.
    assert_eq!(model._get_text("E8"), *"#ERROR!");
    assert_eq!(model._get_text("H1"), *"2");
    assert_eq!(model._get_text("I3"), *"2");
}

#[test]
fn if_iferror_ifna_whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    // Out of column D and row 4 (where they would be circular). T-1448.
    model._set("G1", "=IF(D:D,4:4)");
    model._set("G2", "=IF(D:D,1,4:4)");
    model._set("G6", "=IFERROR(D:D,4:4)");
    model._set("G5", "=IFNA(D:D,4:4)");

    model.evaluate();

    for cell in ["G1", "G2", "G6", "G5"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}
