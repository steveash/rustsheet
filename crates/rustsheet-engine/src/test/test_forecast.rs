#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_forecast_basic() {
    let mut model = new_empty_model();
    // known_y: 2, 3, 9, 1, 8  known_x: 6, 5, 11, 7, 5  predict at x=8
    model._set("A1", "2");
    model._set("A2", "3");
    model._set("A3", "9");
    model._set("A4", "1");
    model._set("A5", "8");
    model._set("B1", "6");
    model._set("B2", "5");
    model._set("B3", "11");
    model._set("B4", "7");
    model._set("B5", "5");
    model._set("C1", "=FORECAST(8, A1:A5, B1:B5)");
    model.evaluate();
    // slope = (5*173 - 34*23) / (5*256 - 34^2) = 83/124
    // intercept = (23 - (83/124)*34) / 5 ≈ 0.04839
    // FORECAST(8) ≈ 0.04839 + (83/124)*8 ≈ 5.40323
    let result = model._get_text("C1");
    let v: f64 = result.parse().unwrap();
    assert!(
        (v - 5.40323).abs() < 0.001,
        "FORECAST result {v} not close to expected 5.40323"
    );
}

#[test]
fn test_forecast_perfect_line() {
    let mut model = new_empty_model();
    // y = 2x + 1
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "7");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "=FORECAST(4, A1:A3, B1:B3)");
    model.evaluate();
    let result = model._get_text("C1");
    let v: f64 = result.parse().unwrap();
    assert!(
        (v - 9.0).abs() < 1e-10,
        "FORECAST(4) on y=2x+1 should be 9, got {v}"
    );
}

#[test]
fn test_forecast_linear_identical_to_forecast() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "5");
    model._set("A3", "7");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "=FORECAST(4, A1:A3, B1:B3)");
    model._set("C2", "=FORECAST.LINEAR(4, A1:A3, B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), model._get_text("C2"));
}

#[test]
fn test_forecast_constant_x_returns_div_error() {
    let mut model = new_empty_model();
    // All x values the same -> division by zero
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "5");
    model._set("B2", "5");
    model._set("B3", "5");
    model._set("C1", "=FORECAST(5, A1:A3, B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#DIV/0!");
}

#[test]
fn test_forecast_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=FORECAST(1, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

/// T-2028: the FORECAST.ETS family stays `#N/IMPL!`, but `target_date` and `statistic_type`
/// read as numbers first (text is `#VALUE!`) and lift over a two-cell range.
#[test]
fn forecast_ets_family_reads_lifted_argument_then_is_not_implemented() {
    let mut model = new_empty_model();
    for row in 1..=8 {
        model._set(&format!("A{row}"), &format!("{}", row * 2));
        model._set(&format!("B{row}"), &format!("{row}"));
    }
    model._set("D1", "=FORECAST.ETS(9,A1:A8,B1:B8)");
    model._set("D2", "=FORECAST.ETS(\"x\",A1:A8,B1:B8)");
    model._set("D3", "=FORECAST.ETS.CONFINT(9,A1:A8,B1:B8)");
    model._set("D4", "=FORECAST.ETS.CONFINT(\"x\",A1:A8,B1:B8)");
    model._set("D5", "=FORECAST.ETS.STAT(A1:A8,B1:B8,1)");
    model._set("D6", "=FORECAST.ETS.STAT(A1:A8,B1:B8,\"x\")");
    model._set("D7", "=ROWS(FORECAST.ETS({9;9},A1:A8,B1:B8))");
    model._set("D8", "=ROWS(FORECAST.ETS.STAT(A1:A8,B1:B8,B1:B2))");
    model.evaluate();
    let got: Vec<String> = (1..=8).map(|r| model._get_text(&format!("D{r}"))).collect();
    assert_eq!(
        got,
        ["#N/IMPL!", "#VALUE!", "#N/IMPL!", "#VALUE!", "#N/IMPL!", "#VALUE!", "2", "2"]
    );
}

/// T-2424: before it gives up, the family reads every argument in order and the first that fails
/// decides: a scalar argument's error or non-numeric text, or an error given directly as `values`
/// or `timeline` (statistics.md "FORECAST.ETS family on excel-graph: argument errors, text and
/// lifting decide before the #N/A (T-2255)"; gen-m9/22656c225ff4f22a A1, A28, A32, A69).
#[test]
fn forecast_ets_family_first_failing_argument_decides() {
    let mut model = new_empty_model();
    for row in 1..=8 {
        model._set(&format!("A{row}"), &format!("{}", row * 2));
        model._set(&format!("B{row}"), &format!("{row}"));
    }
    model._set("C1", "ABC");
    let cells = [
        // An error in each position: target_date, values, timeline, then the optional ones.
        ("=FORECAST.ETS(NA(),1/0,B1:B8)", "#N/A"),
        ("=FORECAST.ETS(9,1/0,SQRT(-1))", "#DIV/0!"),
        ("=FORECAST.ETS(9,A1:A8,SQRT(-1),NA())", "#NUM!"),
        ("=FORECAST.ETS(9,A1:A8,B1:B8,1/0,NA())", "#DIV/0!"),
        ("=FORECAST.ETS(9,A1:A8,B1:B8,1,1,NA())", "#N/A"),
        (
            "=FORECAST.ETS.CONFINT(9,A1:A8,B1:B8,0.95,1,1,SQRT(-1))",
            "#NUM!",
        ),
        ("=FORECAST.ETS.SEASONALITY(A1:A8,SQRT(-1),NA())", "#NUM!"),
        ("=FORECAST.ETS.SEASONALITY(A1:A8,B1:B8,1,NA())", "#N/A"),
        ("=FORECAST.ETS.STAT(A1:A8,B1:B8,1,1,1,1/0)", "#DIV/0!"),
        // Text is #VALUE! in its turn, before a later argument's error.
        ("=FORECAST.ETS(C1,1/0,B1:B8)", "#VALUE!"),
        ("=FORECAST.ETS(9,A1:A8,B1:B8,\"x\",NA())", "#VALUE!"),
        ("=FORECAST.ETS.STAT(A1:A8,B1:B8,C1,NA())", "#VALUE!"),
        ("=FORECAST.ETS.SEASONALITY(A1:A8,B1:B8,\"x\")", "#VALUE!"),
        // A call with no error or text stays unimplemented.
        ("=FORECAST.ETS(9,A1:A8,B1:B8,1,1,1)", "#N/IMPL!"),
        ("=FORECAST.ETS.SEASONALITY(A1:A8,B1:B8,1,1)", "#N/IMPL!"),
        // A two-row scalar argument lifts.
        ("=ROWS(FORECAST.ETS(9,A1:A8,B1:B8,{1;1}))", "2"),
        ("=ROWS(FORECAST.ETS.SEASONALITY(A1:A8,B1:B8,B1:B2))", "2"),
    ];
    for (row, (formula, _)) in cells.iter().enumerate() {
        model._set(&format!("E{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cells.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("E{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}
