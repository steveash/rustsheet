#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A workbook name read through a sheet prefix shows the workbook's file name as its prefix and
// keeps its value when that sheet is renamed (probes/sheet-ops/sheet-ops-rename-workbook-name-
// via-prefix, sheet-ops-workbook-name-via-prefix-no-rename, workbook-name-own-book-prefix;
// excel-graph, T-1701, T-2834).
fn model_with_workbook_name(workbook: &str) -> crate::Model {
    let mut model = new_empty_model();
    model.workbook.name = workbook.to_string();
    model.new_sheet();
    model.rename_sheet("Sheet1", "Calc").unwrap();
    model.rename_sheet("Sheet2", "Feb").unwrap();
    model.define_name("loc", None, "=Feb!$A$1").unwrap();
    model._set("Feb!A1", "5");
    model._set("Calc!A1", "=Feb!loc");
    model._set("Calc!A2", "=SUM(Feb!loc)+1");
    model._set("Calc!A3", "=[0]!loc");
    model._set("Calc!A4", "=Calc!loc");
    model.evaluate();
    model
}

#[test]
fn workbook_name_through_a_sheet_shows_the_file_name() {
    let model = model_with_workbook_name("ebfe7ed");
    assert_eq!(model._get_formula("Calc!A1"), "=ebfe7ed.xlsx!loc");
    assert_eq!(model._get_formula("Calc!A2"), "=SUM(ebfe7ed.xlsx!loc)+1");
    assert_eq!(model._get_formula("Calc!A3"), "=ebfe7ed.xlsx!loc");
    assert_eq!(model._get_formula("Calc!A4"), "=ebfe7ed.xlsx!loc");
    assert_eq!(model._get_text("Calc!A1"), "5");
    assert_eq!(model._get_text("Calc!A2"), "6");
    assert_eq!(model._get_text("Calc!A3"), "5");
}

#[test]
fn a_digit_led_file_name_is_quoted() {
    let model = model_with_workbook_name("53aa");
    assert_eq!(model._get_formula("Calc!A1"), "='53aa.xlsx'!loc");
    assert_eq!(model._get_text("Calc!A1"), "5");
}

#[test]
fn the_file_prefixed_name_survives_a_sheet_rename() {
    let mut model = model_with_workbook_name("53aa");
    model.rename_sheet("Feb", "Mar").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Calc!A1"), "='53aa.xlsx'!loc");
    assert_eq!(model._get_formula("Calc!A2"), "=SUM('53aa.xlsx'!loc)+1");
    assert_eq!(model._get_text("Calc!A1"), "5");
    assert_eq!(model._get_text("Calc!A2"), "6");
}

#[test]
fn the_file_prefix_is_read_back_as_the_workbook_name() {
    let mut model = model_with_workbook_name("Budget");
    model._set("Calc!B1", "='budget.XLSX'!loc*2");
    model.evaluate();
    assert_eq!(model._get_formula("Calc!B1"), "=Budget.xlsx!loc*2");
    assert_eq!(model._get_text("Calc!B1"), "10");
}

#[test]
fn a_sheet_with_its_own_name_reads_the_workbook_one_through_the_file_prefix() {
    let mut model = model_with_workbook_name("Budget");
    model.define_name("loc", Some(0), "=Feb!$A$2").unwrap();
    model._set("Feb!A2", "7");
    model._set("Calc!B1", "=[0]!loc");
    model._set("Calc!B2", "=loc");
    model.evaluate();
    assert_eq!(model._get_formula("Calc!B1"), "=Budget.xlsx!loc");
    assert_eq!(model._get_text("Calc!B1"), "5");
    assert_eq!(model._get_text("Calc!B2"), "7");
}
