#![allow(clippy::unwrap_used)]

// Functions that walk every cell of a range read only the used area of a
// whole-row or whole-column argument (`clip_to_used_area`); the cells past it
// are empty, so the value is the one the used-area range gives (T-0048).

use std::time::{Duration, Instant};

use crate::test::util::new_empty_model;

//         ║   A   |   B   |   C   |
// ════════╬═══════╪═══════╪═══════╪
//    1    ║   2   |   1   |       |
//    2    ║   3   |       |   4   |
//    3    ║       |   5   | "x"   |
fn sample() -> crate::Model {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("B1", "1");
    model._set("A2", "3");
    model._set("C2", "4");
    model._set("B3", "5");
    model._set("C3", "x");
    // The formulas sit on the second sheet, outside the whole-sheet ranges.
    model.new_sheet();
    model
}

fn same_as_used_area(function: &str, whole: &str, used: &str) {
    let mut model = sample();
    model._set("Sheet2!E1", &format!("={function}(Sheet1!{whole})"));
    model._set("Sheet2!E2", &format!("={function}(Sheet1!{used})"));
    let start = Instant::now();
    model.evaluate();
    assert!(
        start.elapsed() < Duration::from_secs(10),
        "{function}({whole}) was not clipped"
    );
    assert_eq!(
        model._get_text("Sheet2!E1"),
        model._get_text("Sheet2!E2"),
        "{function}({whole})"
    );
}

#[test]
fn multinomial_over_whole_rows() {
    same_as_used_area("MULTINOMIAL", "1:1", "A1:C1");
}

#[test]
fn networkdays_holidays_over_whole_column() {
    let mut model = new_empty_model();
    model._set("A1", "45300");
    model._set("A2", "45301");
    model.new_sheet();
    model._set("Sheet2!C1", "=NETWORKDAYS(45292,45330,Sheet1!A:A)");
    model._set("Sheet2!C2", "=NETWORKDAYS(45292,45330,Sheet1!A1:A2)");
    let start = Instant::now();
    model.evaluate();
    assert!(start.elapsed() < Duration::from_secs(10));
    assert_eq!(model._get_text("Sheet2!C1"), "27");
    assert_eq!(model._get_text("Sheet2!C2"), "27");
}

#[test]
fn count_family_over_whole_sheet_ranges() {
    same_as_used_area("COUNT", "A:XFD", "A1:C3");
    same_as_used_area("COUNTA", "A:XFD", "A1:C3");
    same_as_used_area("AVERAGEA", "A:XFD", "A1:C3");
    same_as_used_area("MAXA", "1:1048576", "A1:C3");
}

#[test]
fn countblank_counts_the_clipped_cells() {
    let mut model = sample();
    model._set("Sheet2!E1", "=COUNTBLANK(Sheet1!A:A)");
    model._set("Sheet2!E2", "=COUNTBLANK(Sheet1!1:1)");
    model._set("Sheet2!E3", "=COUNTBLANK(Sheet1!A1:C3)");
    model.evaluate();
    // Column A holds two values; row 1 holds two values; A1:C3 holds six.
    assert_eq!(model._get_text("Sheet2!E1"), "1048574");
    assert_eq!(model._get_text("Sheet2!E2"), "16382");
    assert_eq!(model._get_text("Sheet2!E3"), "3");
}
