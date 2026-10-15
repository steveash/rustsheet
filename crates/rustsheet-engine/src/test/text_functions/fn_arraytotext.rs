#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_arraytotext_concise() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "hello");
    model._set("A2", "2");
    model._set("B2", "world");
    model._set("C1", "=ARRAYTOTEXT(A1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1, hello, 2, world");
}

#[test]
fn test_arraytotext_strict() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "hello");
    model._set("C1", "=ARRAYTOTEXT(A1:B1, 1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "{1,\"hello\"}");
}

#[test]
fn test_arraytotext_numbers_use_general_text_like_valuetotext() {
    let mut model = new_empty_model();
    model._set("A1", "=ARRAYTOTEXT(1/3)");
    model._set("A2", "=VALUETOTEXT(1/3)");
    model._set("B1", "=ARRAYTOTEXT({1E+20})");
    model._set("B2", "=VALUETOTEXT(1E+20)");
    model._set("E1", "=1/3");
    model._set("F1", "1E+20");
    model._set("C1", "=ARRAYTOTEXT(E1:F1, 0)");
    model._set("C2", "=ARRAYTOTEXT(E1:F1, 1)");
    model._set("D1", "=ARRAYTOTEXT(YEARFRAC(1911,31142))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), model._get_text("A2"));
    assert_eq!(model._get_text("A1"), "0.333333333333333");
    assert_eq!(model._get_text("B1"), model._get_text("B2"));
    assert_eq!(model._get_text("B1"), "1E+20");
    assert_eq!(model._get_text("C1"), "0.333333333333333, 1E+20");
    assert_eq!(model._get_text("C2"), "{0.333333333333333,1E+20}");
    assert_eq!(model._get_text("D1"), "80.0277777777778");
}
