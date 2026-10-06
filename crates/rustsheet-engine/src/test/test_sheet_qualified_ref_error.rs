#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// docs/excel-semantics/references.md "A sheet-qualified #REF! is #REF!"

#[test]
fn sheet_qualified_ref_error_in_a_formula() {
    let mut model = new_empty_model();
    model._set("A1", "='Sheet1'!#REF!");
    model._set("A2", "=Sheet1!#REF!");
    model._set("A3", "=SUM(Sheet1!#REF!)");
    model._set("A4", "=Sheet1!#REF!+1");
    model._set("A5", "=#REF!");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "A5"] {
        assert_eq!(model._get_text(cell), *"#REF!", "{cell}");
    }
}

#[test]
fn sheet_qualified_non_ref_error_is_not_a_reference() {
    let mut model = new_empty_model();
    model._set("A1", "=Sheet1!#NAME?");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn defined_name_with_a_sheet_qualified_ref_error() {
    let mut model = new_empty_model();
    model
        .new_defined_name("gone", None, "Sheet1!#REF!")
        .unwrap();
    model._set("A1", "=gone*1");
    model._set("A2", "=aaa*1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#REF!");
    assert_eq!(model._get_text("A2"), *"#NAME?");
}
