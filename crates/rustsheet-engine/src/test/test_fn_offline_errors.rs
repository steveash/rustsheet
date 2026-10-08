#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// SPEC-0003 F1; expected values are probes/rich-errors-args and rich-errors-{image,stockhistory,field}.
#[test]
fn offline_functions_return_their_errors() {
    let mut model = new_empty_model();
    model._set("A1", r#"=IMAGE("https://example.invalid/none.png")"#);
    model._set("A2", r#"=_xlfn.STOCKHISTORY("QZXQZXQ",DATE(2020,1,2))"#);
    model._set("A3", r#"=_xlfn.FIELDVALUE(B1,"Price")"#);
    model._set("B1", "5");
    model._set("A4", "=ERROR.TYPE(A1)");
    model._set("A5", "=ERROR.TYPE(A2)");
    model._set("A6", "=ERROR.TYPE(A3)");
    model.evaluate();
    // The stored names are not formula literals, so VALUETOTEXT spells the error.
    model._set("C1", "=VALUETOTEXT(A1)");
    model._set("C2", "=VALUETOTEXT(A2)");
    model._set("C3", "=VALUETOTEXT(A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#BLOCKED!");
    assert_eq!(model._get_text("C2"), *"#CONNECT!");
    assert_eq!(model._get_text("C3"), *"#FIELD!");
    assert_eq!(model._get_text("A4"), *"11");
    assert_eq!(model._get_text("A5"), *"10");
    assert_eq!(model._get_text("A6"), *"13");
}

#[test]
fn argument_errors_win_except_for_fieldvalue_value() {
    let mut model = new_empty_model();
    model._set("A1", "=IMAGE(1/0)");
    model._set("A2", r#"=IMAGE("x",1/0)"#);
    model._set("A3", "=_xlfn.STOCKHISTORY(1/0,DATE(2020,1,2))");
    model._set("A4", r#"=_xlfn.FIELDVALUE(1/0,"Price")"#);
    model._set("A5", "=_xlfn.FIELDVALUE(5,1/0)");
    model._set("A6", "=ERROR.TYPE(A4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"13");
}

#[test]
fn image_of_empty_or_number_is_value_error() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("A1", "=IMAGE(B1)");
    model._set("A2", "=IMAGE(B2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}

#[test]
fn fieldvalue_of_an_array_is_an_array_of_field_errors() {
    let mut model = new_empty_model();
    model._set("A1", r#"=_xlfn.FIELDVALUE({1,2},"Price")"#);
    model._set("C1", "=VALUETOTEXT(A1)");
    model._set("C2", "=VALUETOTEXT(B1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#FIELD!");
    assert_eq!(model._get_text("C2"), *"#FIELD!");
}

#[test]
fn wrong_argument_counts_are_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=IMAGE()");
    model._set("A2", r#"=IMAGE("a","b","c","d","e","f")"#);
    model._set("A3", r#"=_xlfn.STOCKHISTORY("A")"#);
    model._set("A4", r#"=_xlfn.FIELDVALUE("A")"#);
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}
