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

#[test]
fn image_stockhistory_and_fieldvalue_lift_over_arrays() {
    let mut model = new_empty_model();
    model._set("Q1", "a");
    model._set("Q2", "b");
    model._set("A1", r#"=IMAGE({"a","b"})"#);
    model._set("A3", "=_xlfn.STOCKHISTORY(Q1:Q2,DATE(2020,1,2))");
    model._set("A6", r#"=_xlfn.FIELDVALUE(5,{"a","b"})"#);
    model._set("D1", r#"=IMAGE("https://example.invalid/none.png")"#);
    model._set("D2", "=ERROR.TYPE(IMAGE(TRUE))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("B1"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#CONNECT!");
    assert_eq!(model._get_text("A4"), *"#CONNECT!");
    assert_eq!(model._get_text("A6"), *"#FIELD!");
    assert_eq!(model._get_text("B6"), *"#FIELD!");
    assert_eq!(model._get_text("D1"), *"#BLOCKED!");
    assert_eq!(model._get_text("D2"), *"3");
}

#[test]
fn error_type_lifts_over_arrays_and_elements_keep_their_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=ERROR.TYPE({#DIV/0!,5})");
    model._set("A3", r#"=IMAGE({#DIV/0!,"x"})"#);
    model._set("A5", r#"=_xlfn.FIELDVALUE({1;2},"p")"#);
    model._set("C5", r#"=_xlfn.FIELDVALUE(1,1/0)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("B1"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("B3"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#FIELD!");
    assert_eq!(model._get_text("A6"), *"#FIELD!");
    assert_eq!(model._get_text("C5"), *"#DIV/0!");
}

#[test]
fn fieldvalue_broadcast_over_huge_ranges_is_bounded() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=_xlfn.FIELDVALUE(SEQUENCE(1048576),SEQUENCE(1,16384))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
