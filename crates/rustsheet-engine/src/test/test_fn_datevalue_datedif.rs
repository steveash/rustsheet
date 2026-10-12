#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::{Cell, FormulaValue};

// Helper to evaluate a formula and return the formatted text
fn eval_formula(formula: &str) -> String {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

// Helper that evaluates a formula and returns the raw value of A1 as a Result<f64, String>
fn eval_formula_raw_number(formula: &str) -> Result<f64, String> {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    match model._get_cell("A1") {
        Cell::NumberCell { v, .. } => Ok(*v),
        Cell::CellFormula {
            v: FormulaValue::Number(v),
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Number(v),
            ..
        } => Ok(*v),
        Cell::BooleanCell { v, .. } => Ok(if *v { 1.0 } else { 0.0 }),
        Cell::ErrorCell { ei, .. } => Err(format!("{ei}")),
        _ => Err(model._get_text("A1")),
    }
}

#[test]
fn test_datevalue_basic_numeric() {
    // DATEVALUE should return the serial number representing the date, **not** a formatted date
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"2/1/2023\")").unwrap(),
        44958.0
    );
}

#[test]
fn test_datevalue_mmdd_with_leading_zero() {
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"02/01/2023\")").unwrap(),
        44958.0
    ); // 1-Feb-2023
}

#[test]
fn test_datevalue_iso() {
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"2023-01-02\")").unwrap(),
        44928.0
    );
}

#[test]
fn test_datevalue_month_name() {
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"2-Jan-23\")").unwrap(),
        44928.0
    );
}

#[test]
fn test_datevalue_ambiguous_ddmm() {
    // 01/02/2023 interpreted as MM/DD -> 2-Jan-2023
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"01/02/2023\")").unwrap(),
        44928.0
    );
}

#[test]
fn test_datevalue_day_first_is_not_en_us() {
    // en-US reads numeric dates month first only: no month 15 or 29, so #VALUE!
    // (probes/date-text-grammar A32, A57, A58; docs/excel-semantics/dates.md).
    for formula in ["=DATEVALUE(\"15/01/2023\")", "=DATEVALUE(\"29/02/2020\")"] {
        assert_eq!(eval_formula(formula), *"#VALUE!", "{formula}");
    }
}

#[test]
fn test_datevalue_leap_day() {
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"2/29/2020\")").unwrap(),
        43890.0
    );
}

#[test]
fn test_datevalue_year_first_text_month() {
    // Year-first dates are numeric only (probes/date-text-grammar A20, A56).
    assert_eq!(eval_formula("=DATEVALUE(\"2023/Jan/15\")"), *"#VALUE!");
}

#[test]
fn test_datevalue_month_name_and_clock_text() {
    // probes/date-text-grammar B1, B30, B55, B53.
    for (formula, expected) in [
        ("=DATEVALUE(\"July 1, 2026\")", 46204.0),
        ("=DATEVALUE(\"July 1, 2026 10:00\")", 46204.0),
        ("=DATEVALUE(\"Jul 1, 2026 4:35 PM\")", 46204.0),
        ("=DATEVALUE(\"1/2/2020 25:00\")", 43833.0),
    ] {
        assert_eq!(eval_formula_raw_number(formula), Ok(expected), "{formula}");
    }
    for formula in [
        "=DATEVALUE(\"2026-01-01 garbage\")",
        "=DATEVALUE(\"2026-07-01T10:00\")",
    ] {
        assert_eq!(eval_formula(formula), *"#VALUE!", "{formula}");
    }
}

#[test]
fn test_datevalue_mmdd_with_day_gt_12() {
    assert_eq!(
        eval_formula_raw_number("=DATEVALUE(\"6/15/2021\")").unwrap(),
        44362.0
    );
}

#[test]
fn test_datevalue_error_conditions() {
    let cases = [
        "=DATEVALUE(\"31/04/2023\")", // invalid day (Apr has 30 days)
        "=DATEVALUE(\"13/13/2023\")", // invalid month
        "=DATEVALUE(\"not a date\")", // non-date text
    ];
    for formula in cases {
        let result = eval_formula(formula);
        assert_eq!(result, *"#VALUE!", "Expected #VALUE! for {formula}");
    }
}

#[test]
fn test_datevalue_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE()");
    model._set("A2", "=DATEVALUE(\"2023-01-02\")");
    model._set("A3", "=DATEVALUE(\"2023-01-02\", \"2023-01-03\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"44928");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

// Helper to set and evaluate a single DATEDIF call
fn eval_datedif(unit: &str) -> String {
    let mut model = new_empty_model();
    let formula = format!("=DATEDIF(\"2020-01-01\", \"2021-06-15\", \"{unit}\")");
    model._set("A1", &formula);
    model.evaluate();
    model._get_text("A1")
}

#[test]
fn test_datedif_y() {
    assert_eq!(eval_datedif("Y"), *"1");
}

#[test]
fn test_datedif_m() {
    assert_eq!(eval_datedif("M"), *"17");
}

#[test]
fn test_datedif_d() {
    assert_eq!(eval_datedif("D"), *"531");
}

#[test]
fn test_datedif_ym() {
    assert_eq!(eval_datedif("YM"), *"5");
}

#[test]
fn test_datedif_yd() {
    assert_eq!(eval_datedif("YD"), *"165");
}

#[test]
fn test_datedif_md() {
    assert_eq!(eval_datedif("MD"), *"14");
}

#[test]
fn test_datedif_edge_and_error_cases() {
    let mut model = new_empty_model();
    // Leap-year spanning
    model._set("A1", "=DATEDIF(\"2/28/2020\", \"3/1/2020\", \"D\")");
    // End date before start date => #NUM!
    model._set("A2", "=DATEDIF(\"2/1/2021\", \"1/1/2021\", \"D\")");
    // Invalid unit => #VALUE!
    model._set("A3", "=DATEDIF(\"1/1/2020\", \"1/1/2021\", \"Z\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
}

#[test]
fn test_datedif_mixed_case_unit() {
    assert_eq!(eval_datedif("yD"), *"165"); // mixed-case should work
}

#[test]
fn test_datedif_error_propagation() {
    // Invalid date in arguments should propagate #VALUE!
    let mut model = new_empty_model();
    model._set("A1", "=DATEDIF(\"bad\", \"bad\", \"Y\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
}
