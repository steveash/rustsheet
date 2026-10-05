#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_inverted_range_sum() {
    let mut model = new_empty_model();
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("B4", "3");

    model._set("A1", "=SUM(B2:B4)");
    model._set("A2", "=SUM(B4:B2)"); // inverted

    model.evaluate();

    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("A2"), "6");
}

#[test]
fn test_mixed_absolute_relative_range() {
    let mut model = new_empty_model();
    model._set("B2", "10");
    model._set("B3", "20");
    model._set("B4", "30");
    model._set("B5", "40");

    model._set("A5", "=SUM(B$2:B5)");
    model._set("C5", "=SUM(B5:B$2)"); // inverted

    model.evaluate();

    assert_eq!(model._get_text("A5"), "100");
    assert_eq!(model._get_text("C5"), "100");
}

#[test]
fn test_mixed_absolute_range_after_sheet_rename() {
    let mut model = new_empty_model();
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("B4", "3");
    model._set("B5", "4");

    model._set("A5", "=SUM(B$2:B5)");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "10");

    model.rename_sheet("Sheet1", "RenamedSheet").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("RenamedSheet!A5"), "10");
}

#[test]
fn test_inverted_range_absolute_column() {
    let mut model = new_empty_model();
    model._set("B2", "5");
    model._set("C2", "10");
    model._set("D2", "15");

    model._set("A1", "=SUM($B2:D2)");
    model._set("A2", "=SUM(D2:$B2)"); // inverted

    model.evaluate();

    assert_eq!(model._get_text("A1"), "30");
    assert_eq!(model._get_text("A2"), "30");
}

#[test]
fn test_inverted_2d_range() {
    let mut model = new_empty_model();
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("C2", "3");
    model._set("C3", "4");

    model._set("A1", "=SUM(B2:C3)");
    model._set("A2", "=SUM(C3:B2)"); // fully inverted
    model._set("A3", "=SUM(B3:C2)"); // row inverted
    model._set("A4", "=SUM(C2:B3)"); // column inverted

    model.evaluate();

    assert_eq!(model._get_text("A1"), "10");
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "10");
    assert_eq!(model._get_text("A4"), "10");
}

#[test]
fn test_mixed_absolute_2d_range() {
    let mut model = new_empty_model();
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("B4", "3");
    model._set("C2", "4");
    model._set("C3", "5");
    model._set("C4", "6");
    model._set("D2", "7");
    model._set("D3", "8");
    model._set("D4", "9");

    model._set("A4", "=SUM($B$2:D4)");
    model._set("E4", "=SUM(D4:$B$2)"); // inverted

    model.evaluate();

    assert_eq!(model._get_text("A4"), "45");
    assert_eq!(model._get_text("E4"), "45");

    model.rename_sheet("Sheet1", "Data").unwrap();
    model.evaluate();

    assert_eq!(model._get_text("Data!A4"), "45");
    assert_eq!(model._get_text("Data!E4"), "45");
}

#[test]
fn test_range_with_both_corners_sheet_qualified() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model._set("Data!B2", "1");
    model._set("Data!B3", "2");
    model._set("Data!C2", "3");
    model._set("Data!C3", "4");
    model._set("Data!E1", "=SUM(Data!B2:Data!C3)");
    model._set("Data!E2", "=SUM(Data!B2:C3)");

    model.evaluate();

    assert_eq!(model._get_text("Data!E1"), "10");
    assert_eq!(model._get_text("Data!E2"), "10");
    // The formula round-trips through stringify unchanged.
    assert_eq!(model._get_formula("Data!E1"), "=SUM(Data!B2:Data!C3)");
}

#[test]
fn test_range_with_both_corners_quoted_sheet_qualified() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "It's data").unwrap();
    model.set_user_input(1, 2, 2, "1".to_string()).unwrap();
    model.set_user_input(1, 3, 3, "4".to_string()).unwrap();
    model._set("A1", "=SUM('It''s data'!B2:'It''s data'!C3)");
    model._set("A2", "=SUM('It''s data'!B2:C3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "5");
}

#[test]
fn test_range_with_both_corners_reversed() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model._set("Data!A1", "1");
    model._set("Data!B1", "2");
    model._set("Data!A2", "3");
    model._set("Data!B2", "4");
    model._set("Data!E1", "=SUM(Data!B2:Data!A1)");

    model.evaluate();

    assert_eq!(model._get_text("Data!E1"), "10");
}

#[test]
fn test_array_arithmetic_coerces_text_elements() {
    let mut model = new_empty_model();
    model._set("B1", "' 2 ");
    model._set("B2", "abc");
    model._set("A1", "=B1:B1/1");
    model._set("A2", "=B1:B1*2.5");
    model._set("A3", "=B1:B1/0");
    model._set("A4", "=B2:B2+1");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "#DIV/0!");
    assert_eq!(model._get_text("A4"), "#VALUE!");
}

// gen-m1/4dc8ed1de05a61be Calc!A250: the error in the left range operand wins
// over the right operand's error (docs/excel-semantics/coercion.md, T-0137).
#[test]
fn test_range_operand_error_wins_over_right_scalar_error() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model._set("Data!C9", "=NA()");
    model._set("Data!D5", "=1/0");
    model._set("A1", "=(Data!C1:C10 Data!B9:D9)/Data!D$5");
    model._set("A2", "=Data!C9:C9/Data!D$5");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "#N/A");
    assert_eq!(model._get_text("A2"), "#N/A");
}
