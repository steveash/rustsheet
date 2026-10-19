#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_if_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=IF()");
    model._set("A2", "=IF(1, 2, 3, 4)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn fn_if_2_args() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(2 > 3, TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
}

#[test]
fn fn_if_missing_args() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(2 > 3, TRUE, )");
    model._set("A2", "=IF(2 > 3, , 5)");
    model._set("A3", "=IF(2 < 3, , 5)");

    model.evaluate();

    // assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"5");
    assert_eq!(model._get_text("A3"), *"0");
}

// Excel: IF and INDEX hand an aggregate a reference, so its text and booleans
// are skipped like a range cell's; elsewhere they are values (T-0216;
// docs/excel-semantics/references.md).
#[test]
fn test_if_and_index_pass_references_to_aggregates() {
    let mut model = new_empty_model();
    model._set("A1", "abc");
    model._set("A2", "'1");
    model._set("A3", "TRUE");
    model._set("A4", "5");
    model._set("B1", "=SUM(IF(FALSE,-2.5,A1),A4)");
    model._set("B2", "=COUNT(IF(TRUE,A2),A4)");
    model._set("B3", "=SUM(A4,INDEX(A1:A2,1))");
    model._set("B4", "=MAX(IF(TRUE,A1),INDEX(A1:A4,3),A4)");
    model._set("B5", "=AVERAGE(IF(TRUE,A3),A4)");
    model._set("B6", "=ISREF(INDEX(A1:A2,2))");
    model._set("B7", "=IF(TRUE,A1)&\"x\"");
    model._set("B8", "=SUM(IF(TRUE,\"abc\"))");
    model._set("B9", "=MIN(IF(TRUE,A1),INDEX(A1:A4,3),A4)");
    model._set("B10", "=ISREF(INDEX(A1,1,2,2))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"5");
    assert_eq!(model._get_text("B2"), *"1");
    assert_eq!(model._get_text("B3"), *"5");
    assert_eq!(model._get_text("B4"), *"5");
    assert_eq!(model._get_text("B5"), *"5");
    assert_eq!(model._get_text("B6"), *"TRUE");
    assert_eq!(model._get_text("B7"), *"abcx");
    assert_eq!(model._get_text("B8"), *"#VALUE!");
    assert_eq!(model._get_text("B9"), *"5");
    assert_eq!(model._get_text("B10"), *"FALSE");
}

// An eta-reduced IF called with one argument is `#VALUE!` per element that is not an error
// (probes/eta-if-one-argument, T-1947).
#[test]
fn eta_if_with_one_argument() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "0");
    model._set("C1", "=LET(f,_xleta.IF,f(TRUE))");
    model._set("C2", "=LET(f,_xleta.IF,f(NA()))");
    model._set("C3", "=LET(f,_xleta.IF,f({1,#DIV/0!}))");
    model._set("C5", "=BYROW(A1:B1,_xleta.IF)");
    model._set("C6", "=LET(f,_xleta.IF,f(FALSE,5))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
    assert_eq!(model._get_text("C2"), "#N/A");
    assert_eq!(model._get_text("C3"), "#VALUE!");
    assert_eq!(model._get_text("D3"), "#DIV/0!");
    assert_eq!(model._get_text("C5"), "#CALC!");
    assert_eq!(model._get_text("C6"), "FALSE");
}
