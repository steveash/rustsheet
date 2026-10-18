#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel does not snap a grouped defined-name body to zero (docs/excel-semantics/numbers.md
// "Near-zero snapping", T-1164), so the parentheses must survive every re-stringify of the body.
#[test]
fn grouped_name_body_keeps_its_parentheses_through_copy_and_rename() {
    let mut model = new_empty_model();
    model
        .define_name("grouped", Some(0), "=(Sheet1!$A$1-0.3)")
        .unwrap();
    assert_eq!(model.get_defined_name_list()[0].2, "=(Sheet1!$A$1-0.3)");
    let (_, new_index) = model.duplicate_sheet(0).unwrap();
    let names = model.get_defined_name_list();
    let copy = names
        .iter()
        .find(|(name, scope, _)| name == "grouped" && *scope == Some(new_index))
        .expect("local name should be duplicated on the copy");
    assert_eq!(copy.2, "=('Sheet1 (1)'!$A$1-0.3)");

    model.rename_sheet("Sheet1", "Renamed").unwrap();
    let names = model.get_defined_name_list();
    assert!(names.iter().any(|(name, scope, formula)| name == "grouped"
        && *scope == Some(0)
        && formula == "=(Renamed!$A$1-0.3)"));
}

// `:` joins a grouped reference name with a cell into their bounding box, and `ig` (a column
// spelling) before `:` is still a name (probes/defined-name-grouped-reference A10, A11,
// excel-graph, T-1173).
#[test]
fn grouped_reference_name_joins_with_a_cell_in_the_range_operator() {
    let mut model = new_empty_model();
    model
        .define_name("rg", Some(0), "=(Sheet1!$B$1:$B$2)")
        .unwrap();
    model
        .define_name("ig", Some(0), "=(INDEX(Sheet1!$B$1:$B$2,2))")
        .unwrap();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("A1", "=SUM(ig:B3)");
    model._set("A2", "=ROWS(rg:B3)");
    model._set("A3", "=SUM(B3:ig)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "5");
}

// A call of a cell-shaped name is stored in R1C1 as `R[-7]C[27](1)`, which must parse back after
// a structural edit (T-1758).
#[test]
fn cell_shaped_call_survives_a_sheet_rename() {
    let mut model = new_empty_model();
    model._set("A8", "=Ab1(1)");
    model.evaluate();
    assert_eq!(model._get_formula("A8"), "=AB1(1)");
    model.rename_sheet("Sheet1", "Other").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("A8"), "=AB1(1)");
    assert_eq!(model._get_text("A8"), "#REF!");
}

// A formula that does not parse is stored as its A1 text; a structural edit must not read `Ab1`
// there as the R1C1 offset `R[1]C[28]` (T-1758) or as a call of a name (T-1805).
#[test]
fn unparsed_cell_shaped_call_keeps_its_text_after_a_sheet_rename() {
    let mut model = new_empty_model();
    model._set("A8", "=Ab1 (1)");
    model.evaluate();
    assert_eq!(model._get_formula("A8"), "=Ab1 (1)");
    model.rename_sheet("Sheet1", "Other").unwrap();
    model.evaluate();
    // Not `=AC9(1)`, nor `=Ab1(1)`: the stored parse error is not re-parsed (T-1805).
    assert_eq!(model._get_formula("A8"), "=Ab1 (1)");
    assert_eq!(model._get_text("A8"), "#ERROR!");
}
