use crate::{cell::CellValue, test::util::new_empty_model};

/// Fuzz crash T-1419 (nightly `formula_eval`, crash-4b2019ed): a month near `i32::MAX` panicked in
/// chrono's `NaiveDate + Months`. Excel's values: probes/date-huge-month-day (excel-graph).
#[test]
fn date_with_a_huge_month_or_day_does_not_panic() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2000,1000000000000000,1)");
    model._set("A2", "=DATE(0,1000000000000000,1)");
    model._set("A3", "=DATE(2000,-1000000000000000,1)");
    model._set("A4", "=DATE(2000,1,-1000000000000000)");
    model._set(
        "A5",
        "=(((0.2+B1)+DATE(B2,1000000000000000.0,1))+DAYS360(B3,0.2)%)",
    );
    model._set("B1", "1.5");
    model._set("B2", "ABC");
    model._set("B3", "1");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    // A day outside -32768..=32767 counts as 32767.
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A4"),
        Ok(CellValue::Number(69292.0))
    );
    assert_eq!(model._get_text("A5"), *"#VALUE!");
}

#[test]
fn date_month_bounds_are_exclusive_of_the_16_bit_ends() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2000,32766,1)");
    model._set("A2", "=DATE(2000,32767,1)");
    model._set("A3", "=DATE(9999,-32767,1)");
    model._set("A4", "=DATE(9999,-32768,1)");
    model._set("A5", "=DATE(2000,1,-32768)");
    model._set("A6", "=DATE(2000,1,-32769)");
    model._set("A7", "=DATE(0,13,-1)");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(1033789.0))
    );
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A3"),
        Ok(CellValue::Number(1960745.0))
    );
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A5"),
        Ok(CellValue::Number(3757.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A6"),
        Ok(CellValue::Number(69292.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A7"),
        Ok(CellValue::Number(365.0))
    );
}

/// probes/date-argument-window A16:A21, A48:A55, A83 (excel-graph, T-2268): a year, month or day
/// within the near-integer window below an integer reads as that integer, half the window when
/// negative, before the bounds apply.
#[test]
fn date_arguments_just_below_an_integer_read_as_that_integer() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,3,2-(2^-22+1E-12))");
    model._set("A2", "=DATE(2020,3-(2^-22+1E-12),2)");
    model._set("A3", "=DATE(2021-(2^-22+1E-12),3,2)");
    model._set("A4", "=DATE(2020,3,2-2.4E-7)");
    model._set("A5", "=DATE(2020,3,-(2^-23+1E-12))");
    model._set("A6", "=DATE(2020,3,-1.2E-7)");
    model._set("A7", "=DATE(-1.2E-7,3,2)");
    model._set("A8", "=DATE(9999,12,31+1-1E-7)");
    model.evaluate();

    let number = |c: &str| model.get_cell_value_by_ref(&format!("Sheet1!{c}"));
    assert_eq!(number("A1"), Ok(CellValue::Number(43892.0)));
    assert_eq!(number("A2"), Ok(CellValue::Number(43892.0)));
    assert_eq!(number("A3"), Ok(CellValue::Number(44257.0)));
    assert_eq!(number("A4"), Ok(CellValue::Number(43891.0)));
    assert_eq!(number("A5"), Ok(CellValue::Number(43890.0)));
    assert_eq!(number("A6"), Ok(CellValue::Number(43889.0)));
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
}
