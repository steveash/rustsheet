#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn datevalue_timevalue_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE()");
    model._set("A2", "=TIMEVALUE()");
    model._set("A3", "=DATEVALUE(\"2000-01-01\")");
    model._set("A4", "=TIMEVALUE(\"12:00:00\")");
    model._set("A5", "=DATEVALUE(1,2)");
    model._set("A6", "=TIMEVALUE(1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"36526");
    assert_eq!(model._get_text("A4"), *"0.5");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

// Excel keeps the float error of serial - floor(serial) for "date time" text
// (TIMEVALUE.xlsx B18 cached 0.69097222221898846).
#[test]
fn timevalue_of_datetime_text_is_serial_fraction() {
    let mut model = new_empty_model();
    model._set("A1", "=TIMEVALUE(\"7/7/2026 4:35 PM\")");
    model._set("A2", "=TIMEVALUE(\"4:35 PM\")");
    model.evaluate();

    let value = |cell: &str| match model.get_cell_value_by_ref(cell).unwrap() {
        crate::cell::CellValue::Number(n) => n,
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(value("Sheet1!A1"), 0.690_972_222_218_988_5);
    assert_eq!(value("Sheet1!A2"), 16.583_333_333_333_332 / 24.0);
}
