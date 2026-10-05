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

// Clock text grammar, oracle values from probes/timevalue-*.case.json (excel-graph).
#[test]
fn timevalue_clock_text_grammar() {
    let cases: [(&str, Option<f64>); 13] = [
        ("10:11:12.5", Some(0.424_450_231_481_481_5)),
        ("10:11:12.4995", Some(0.424_450_231_481_481_5)), // half-up on the decimal digits
        ("11:12.5", Some(0.007_783_564_814_814_815)),     // two parts + fraction = M:S
        ("10:11:12.5 P", Some(0.924_450_231_481_481_5)),
        ("10:11:12 . 5", Some(0.424_450_231_481_481_5)),
        ("24:00:00.5", Some(5.787_037_037_130_46e-6)), // frac of 1.0000057870370371
        ("23:59:59.9995", Some(0.0)),
        ("1/2/2020 10:11:12.5", Some(0.424_450_231_483_206_15)),
        ("1/2/2020 25:00", Some(0.041_666_666_664_241_35)), // date + clock past 23 h
        ("10:61 PM", None),
        ("010:11:12", None),
        ("10000:00:00", None),
        ("10:11:12.5PM", None),
    ];
    let mut model = new_empty_model();
    for (i, (text, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), &format!("=TIMEVALUE(\"{text}\")"));
    }
    model.evaluate();
    for (i, (text, expected)) in cases.iter().enumerate() {
        let actual = model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1))
            .unwrap();
        match expected {
            Some(v) => assert_eq!(actual, crate::cell::CellValue::Number(*v), "{text}"),
            None => assert_eq!(model._get_text(&format!("A{}", i + 1)), "#VALUE!", "{text}"),
        }
    }
}
