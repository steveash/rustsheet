#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// `Data!Local` reads the name scoped to Data from another sheet, and keeps reading it after Data
// is renamed (probes/sheet-ops-rename-sheet-scoped-name, excel-graph, T-1608, T-1693).
fn model_with_scoped_name() -> crate::Model {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet1", "Calc").unwrap();
    model.rename_sheet("Sheet2", "Data").unwrap();
    model
        .define_name("Local", Some(1), "=Data!$A$1:$A$2")
        .unwrap();
    model._set("Data!A1", "1");
    model._set("Data!A2", "2");
    model._set("Calc!A1", "=Data!Local");
    model._set("Calc!A2", "=SUM(Data!Local)");
    model.evaluate();
    model
}

#[test]
fn qualified_sheet_scoped_name_evaluates() {
    let model = model_with_scoped_name();
    assert_eq!(model._get_text("Calc!A1"), "#SPILL!");
    assert_eq!(model._get_text("Calc!A2"), "3");
}

#[test]
fn qualified_sheet_scoped_name_follows_a_sheet_rename() {
    let mut model = model_with_scoped_name();
    model.rename_sheet("Data", "Inputs").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Calc!A2"), "=SUM(Inputs!Local)");
    assert_eq!(model._get_text("Calc!A1"), "#SPILL!");
    assert_eq!(model._get_text("Calc!A2"), "3");
}
