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
