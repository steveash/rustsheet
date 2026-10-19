#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Ported-From: ironcalc@f4953b1d, c4641f0f (cases Excel-cached in
// upstream ISREF.xlsx), 29ee0601.
#[test]
fn references_and_values() {
    let mut model = new_empty_model();
    model._set("K1", "text");
    model._set("K2", "7");
    model._set("A1", "=ISREF(K1)");
    model._set("A2", "=ISREF(K1:K2)");
    model._set("A3", "=ISREF(K:K)");
    model._set("A4", "=ISREF(OFFSET(K1, 1, 0))");
    model._set("A5", "=ISREF(INDIRECT(\"K1\"))");
    model._set("A6", "=ISREF(5)");
    model._set("A7", "=ISREF(\"K1\")");
    model._set("A8", "=ISREF(K1+1)");
    model._set("A9", "=ISREF(INDEX(K1:K2, 1))");
    model._set("A10", "=ISREF(INDEX({1,2}, 1))");
    model._set("A11", "=ISREF(INDEX(K1:K2, 3))");
    model.evaluate();

    for (cell, expected) in [
        ("A1", "TRUE"),
        ("A2", "TRUE"),
        ("A3", "TRUE"),
        ("A4", "TRUE"),
        ("A5", "TRUE"),
        ("A6", "FALSE"),
        ("A7", "FALSE"),
        ("A8", "FALSE"),
        ("A9", "TRUE"),
        ("A10", "FALSE"),
        ("A11", "FALSE"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

// A defined name is a reference if what it stands for is one (upstream ISREF.xlsx A14-A18; the constant name `Seven` is in probes/isref-names-filter-if-empty, since the API rejects a constant body).
#[test]
fn defined_names() {
    let mut model = new_empty_model();
    model._set("K1", "text");
    model._set("K2", "7");
    model
        .new_defined_name("TextCell", None, "Sheet1!$K$1")
        .unwrap();
    model
        .new_defined_name("Cells", None, "Sheet1!$K$1:$K$2")
        .unwrap();
    model
        .new_defined_name("Twice", None, "=LAMBDA(x, 2*x)")
        .unwrap();
    model._set("A1", "=ISREF(TextCell)");
    model._set("A2", "=ISREF(Cells)");
    model._set("A3", "=ISREF(Twice)");
    model._set("A4", "=ISREF(NoSuchName)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"FALSE");
    assert_eq!(model._get_text("A4"), *"FALSE");
}

// FILTER's if_empty range is its values, not a reference (upstream ISREF.xlsx D11, D14 values; D17, D20 ISREF FALSE).
#[test]
fn filter_if_empty_range_is_its_values() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "FALSE");
    model._set("D1", "10");
    model._set("D2", "20");
    model._set("F1", "=FILTER(A1:A1,B1:B1,D1:D2)");
    model._set("G1", "=SUM(FILTER(A1:A1,B1:B1,D1:D2))");
    model._set("G2", "=ISREF(FILTER(A1:A1,B1:B1,D1:D2))");
    model.evaluate();

    assert_eq!(model._get_text("F1"), *"10");
    assert_eq!(model._get_text("F2"), *"20");
    assert_eq!(model._get_text("G1"), *"30");
    assert_eq!(model._get_text("G2"), *"FALSE");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=ISREF()");
    model._set("A2", "=ISREF(K1, K2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

// A range corner may be an error, a value, or an IF or CHOOSE that picks a reference
// (probes/isref-range-operands, excel-graph; T-1177).
#[test]
fn range_operand_errors_values_and_if() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model._set(cell, value);
    }
    for (cell, value) in [("B1", "10"), ("B2", "20"), ("B3", "30")] {
        model._set(cell, value);
    }
    model._set("D1", "=ISREF(A1:INDEX(A1:A3,5))");
    model._set("D2", "=ISREF(A1:(A1 B2))");
    model._set("D3", "=ISREF(A1:IF(TRUE,1))");
    model._set("D4", "=ISREF(A1:IF(TRUE,B2))");
    model._set("D5", "=ISREF(IF(TRUE,B2):A1)");
    model._set("E1", "=SUM(A1:IF(FALSE,B2,B3))");
    model._set("E2", "=A1:IF(TRUE,A1)");
    model._set("E3", "=ROWS(A1:IF(TRUE,B3))");
    model._set("E4", "=ERROR.TYPE(A1:IF(TRUE,1))");
    model._set("E5", "=SUM(A1:CHOOSE(1,B2))");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"FALSE");
    assert_eq!(model._get_text("D2"), *"FALSE");
    assert_eq!(model._get_text("D3"), *"FALSE");
    assert_eq!(model._get_text("D4"), *"TRUE");
    assert_eq!(model._get_text("D5"), *"TRUE");
    assert_eq!(model._get_text("E1"), *"66");
    assert_eq!(model._get_text("E2"), *"1");
    assert_eq!(model._get_text("E3"), *"3");
    assert_eq!(model._get_text("E4"), *"3");
    assert_eq!(model._get_text("E5"), *"33");
}

// An INDEX of a reference is a reference wherever one is wanted, ISFORMULA's argument included
// (probes/ifna-isformula-edges Calc!B12, excel-graph; T-1453).
#[test]
fn isformula_of_index_reads_the_picked_cell() {
    let mut model = new_empty_model();
    model._set("K1", "=1+1");
    model._set("K2", "5");
    model._set("A1", "=ISFORMULA(INDEX(K1:K2,1))");
    model._set("A2", "=ISFORMULA(INDEX(K1:K2,2))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
}
