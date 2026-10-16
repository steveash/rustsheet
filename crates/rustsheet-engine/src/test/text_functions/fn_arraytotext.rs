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

/// docs/excel-semantics/text.md "ARRAYTOTEXT number text": numbers are General text, the same
/// text VALUETOTEXT gives (gen-m7/d38b52bfa1a9f9e6 Calc!A2).
#[test]
fn test_arraytotext_numbers_match_valuetotext() {
    for (value, literal, expected) in [
        ("1/3", "0.3333333333333333", "0.333333333333333"),
        ("1E+20", "1E+20", "1E+20"),
    ] {
        for format in [0, 1] {
            let mut model = new_empty_model();
            model._set("A1", &format!("=ARRAYTOTEXT({value}, {format})"));
            model._set("A2", &format!("=VALUETOTEXT({value}, {format})"));
            model._set("A3", &format!("=ARRAYTOTEXT({{{literal}}}, {format})"));
            model.evaluate();
            assert_eq!(model._get_text("A1"), expected);
            assert_eq!(model._get_text("A1"), model._get_text("A2"));
            let braces = if format == 1 {
                format!("{{{expected}}}")
            } else {
                expected.to_string()
            };
            assert_eq!(model._get_text("A3"), braces);
        }
    }
    let mut model = new_empty_model();
    model._set("A1", "=ARRAYTOTEXT({0.3333333333333333,1E+20})");
    model._set("A2", "=ARRAYTOTEXT({0.3333333333333333,1E+20}, 1)");
    model._set("A3", "=ARRAYTOTEXT(YEARFRAC(1911,31142))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "0.333333333333333, 1E+20");
    assert_eq!(model._get_text("A2"), "{0.333333333333333,1E+20}");
    assert_eq!(model._get_text("A3"), "80.0277777777778");
}
