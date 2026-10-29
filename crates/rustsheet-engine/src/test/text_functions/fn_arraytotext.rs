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
            // Strict form braces even a scalar (probes/arraytotext-number-text A4).
            let braced = if format == 1 {
                format!("{{{expected}}}")
            } else {
                expected.to_string()
            };
            assert_eq!(model._get_text("A1"), braced);
            assert_eq!(model._get_text("A2"), expected);
            assert_eq!(model._get_text("A3"), braced);
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

/// docs/excel-semantics/text.md "ARRAYTOTEXT error before format": an error value in the first
/// argument wins over format validation, in argument order (probes/arraytotext-error-vs-format).
#[test]
fn test_arraytotext_error_argument_wins_over_format() {
    let mut model = new_empty_model();
    model._set("B1", "=NA()");
    let cases = [
        ("=ARRAYTOTEXT(NA(),2)", "#N/A"),
        ("=ARRAYTOTEXT(1/0,2)", "#DIV/0!"),
        ("=ARRAYTOTEXT(B1,2)", "#N/A"),
        ("=ARRAYTOTEXT(B1:B1,2)", "#N/A"),
        ("=ARRAYTOTEXT(NA(),1/0)", "#N/A"),
        ("=ARRAYTOTEXT(1/0,NA())", "#DIV/0!"),
        ("=ARRAYTOTEXT(NA(),\"x\")", "#N/A"),
        ("=ARRAYTOTEXT(1,NA())", "#N/A"),
        ("=ARRAYTOTEXT(1,2)", "#VALUE!"),
        ("=ARRAYTOTEXT({1,#N/A},2)", "#VALUE!"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}
