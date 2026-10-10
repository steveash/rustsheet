#![allow(clippy::unwrap_used)]

//! SPEC-0003 rollout E1: Excel's newer error kinds in formulas and typed input.

use crate::test::util::new_empty_model;
use crate::types::CellType;

#[test]
fn getting_data_literal_evaluates_with_its_own_code() {
    // probes/error-literals-new A2, A4, A15 (oracle: excel-graph).
    let mut model = new_empty_model();
    model._set("A1", "=ERROR.TYPE(#GETTING_DATA)");
    model._set("A2", "=ISNA(#getting_data)");
    model._set("A3", "=IFNA(#GETTING_DATA,\"y\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "8");
    assert_eq!(model._get_text("A2"), "FALSE");
    assert_eq!(model._get_text("A3"), "#GETTING_DATA");
}

#[test]
fn blocked_is_not_a_formula_literal() {
    // Excel refuses `=#BLOCKED!` (T-0175); it must not become a #BLOCKED! value.
    let mut model = new_empty_model();
    model._set("A1", "=#BLOCKED!");
    model.evaluate();
    // A formula that does not parse evaluates to the engine's #ERROR!.
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn typed_getting_data_is_an_error_and_typed_blocked_is_text() {
    // SPEC-0003's choice, UNVERIFIED until desktop evidence (T-1016).
    let mut model = new_empty_model();
    model._set("A1", "#getting_data");
    model._set("A2", "#BLOCKED!");
    model.evaluate();
    assert_eq!(model.get_cell_type(0, 1, 1), Ok(CellType::ErrorValue));
    assert_eq!(model._get_text("A1"), "#GETTING_DATA");
    assert_eq!(model.get_cell_type(0, 2, 1), Ok(CellType::Text));
    assert_eq!(model._get_text("A2"), "#BLOCKED!");
}
