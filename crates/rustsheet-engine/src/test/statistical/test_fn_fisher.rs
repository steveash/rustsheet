#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
#[test]
fn test_fn_fisher_smoke() {
    let mut model = new_empty_model();

    // Valid inputs
    model._set("A1", "=FISHER(0.1)");
    model._set("A2", "=FISHER(-0.5)");
    model._set("A3", "=FISHER(0.8)");

    // Domain errors: x <= -1 or x >= 1 -> #NUM!
    model._set("A4", "=FISHER(1)");
    model._set("A5", "=FISHER(-1)");
    model._set("A6", "=FISHER(2)");

    // Wrong number of arguments -> #ERROR!
    model._set("A7", "=FISHER()");
    model._set("A8", "=FISHER(0.1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.100335348");
    assert_eq!(model._get_text("A2"), *"-0.549306144");
    assert_eq!(model._get_text("A3"), *"1.098612289");

    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");

    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");
}

#[test]
fn test_fn_fisher_inv_smoke() {
    let mut model = new_empty_model();

    // Valid inputs
    model._set("A1", "=FISHERINV(-1.5)");
    model._set("A2", "=FISHERINV(0.5)");
    model._set("A3", "=FISHERINV(2)");

    // Wrong number of arguments -> #ERROR!
    model._set("A4", "=FISHERINV()");
    model._set("A5", "=FISHERINV(0.5, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-0.905148254");
    assert_eq!(model._get_text("A2"), *"0.462117157");
    assert_eq!(model._get_text("A3"), *"0.96402758");

    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
}

#[test]
fn test_fn_fisherinv_bit_exact() {
    let mut model = new_empty_model();
    model._set("A1", "=FISHERINV(1E-7)");
    model._set("A2", "=FISHERINV(1000)");
    model._set("A3", "=FISHERINV(-1000)");
    model._set("A4", "=FISHERINV(0)");
    model.evaluate();

    let value = |cell: &str| match model.get_cell_value_by_ref(cell).unwrap() {
        crate::cell::CellValue::Number(n) => n,
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(value("Sheet1!A1"), 1.000_000_000_503_929_2e-7);
    assert_eq!(value("Sheet1!A2"), 1.0);
    assert_eq!(value("Sheet1!A3"), -1.0);
    assert_eq!(value("Sheet1!A4"), 0.0);
}
