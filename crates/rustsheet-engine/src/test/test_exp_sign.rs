#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=EXP()");
    model._set("A2", "=SIGN()");

    model._set("A3", "=EXP(0)");
    model._set("A4", "=SIGN(-10)");

    model._set("A5", "=EXP(1, 2)");
    model._set("A6", "=SIGN(1, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");

    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"-1");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

/// The number in column A of `row`, or NaN (which fails every comparison) when it holds none.
fn number(model: &crate::Model, row: i32) -> f64 {
    match model.get_cell_value_by_index(0, row, 1) {
        Ok(crate::cell::CellValue::Number(v)) => v,
        _ => f64::NAN,
    }
}

/// The worksheet EXP is Excel's x87 model V6, not libm (docs/excel-semantics/numbers.md "EXP is an
/// x87 2^(a*log2 e) evaluation, not correctly rounded"): Excel's own cells, each one ulp from the
/// correctly rounded exp (probes-numeric t1931-exp-random B2, B3, B5, excel-graph).
#[test]
fn exp_is_excels_x87_evaluation() {
    let mut model = new_empty_model();
    // The arguments are stored doubles in column B (a formula literal keeps only 15 digits).
    for (row, a) in [
        (1, -2.060_741_474_999_140_6),
        (2, 34.162_964_267_632_994),
        (3, -12.612_401_543_239_944),
    ] {
        model.update_cell_with_number(0, row, 2, a).unwrap();
        model._set(&format!("A{row}"), &format!("=EXP(B{row})"));
    }
    model.evaluate();
    assert_eq!(number(&model, 1), 0.127_359_500_990_096_19);
    assert_eq!(number(&model, 2), 686_731_339_009_724.8);
    assert_eq!(number(&model, 3), 3.330_455_277_857_461e-6);
}

/// A subnormal EXP result is kept, not flushed (probes/subnormal-function-results C4, C6:
/// `EXP(-745)=0` and `EXP(-720)=0` are FALSE).
#[test]
fn exp_keeps_subnormal_results() {
    let mut model = new_empty_model();
    model._set("A1", "=EXP(-745)=0");
    model._set("A2", "=EXP(-720)=0");
    model._set("A3", "=EXP(-720)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    let tiny = number(&model, 3);
    assert!(tiny > 0.0 && tiny < f64::MIN_POSITIVE);
}
