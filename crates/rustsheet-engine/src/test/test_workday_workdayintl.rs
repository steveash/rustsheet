#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    // Day 9 Dec 2025 = serial number 46000, Tuesday, week 50
    let mut model = new_empty_model();
    model._set("A1", "=WORKDAY()");
    model._set("A2", "=WORKDAY(46000)");
    model._set("A3", "=WORKDAY(46000, 5)");
    model._set("A4", "=WORKDAY(46000, 5, 46001)");
    model._set("A5", "=WORKDAY(46000, 5, 46001, 2)");

    model._set("B1", "=WORKDAY.INTL()");
    model._set("B2", "=WORKDAY.INTL(46000)");
    model._set("B3", "=WORKDAY.INTL(46000, 5)");
    model._set("B4", "=WORKDAY.INTL(46000, 5, 11)");
    model._set("B5", "=WORKDAY.INTL(46000, 5, 11, 46001)");
    model._set("B6", "=WORKDAY.INTL(46000, 5, 11, 46001, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"46007");
    assert_eq!(model._get_text("A4"), *"46008");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"46007");
    assert_eq!(model._get_text("B4"), *"46006");
    assert_eq!(model._get_text("B5"), *"46007");
    assert_eq!(model._get_text("B6"), *"#ERROR!");
}

#[test]
fn whole_week_skipping() {
    // 46000 is a Tuesday; 1000 workdays at 5 per week is exactly 200 weeks.
    let mut model = new_empty_model();
    model._set("A1", "=WORKDAY(46000, 1000)");
    model._set("A2", "=WORKDAY(46000, 1000, 46100)");
    model._set("A3", "=WORKDAY(46000, -1000)");
    model._set("A4", "=WORKDAY(46000, 1000, 46103)");
    model._set("A5", "=WORKDAY.INTL(46000, 2000000, 11)");
    model._set("A6", "=WORKDAY.INTL(46000, 4000000, 11)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"47400");
    // 46100 is a Thursday inside the span: one extra workday.
    assert_eq!(model._get_text("A2"), *"47401");
    assert_eq!(model._get_text("A3"), *"44600");
    // 46103 is a Sunday: no effect.
    assert_eq!(model._get_text("A4"), *"47400");
    assert_eq!(model._get_text("A5"), *"2379333");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

#[test]
fn whole_week_skipping_dense_holidays() {
    // Every workday of a 2000-row holiday column is a holiday; the answer lies past the column.
    let mut model = new_empty_model();
    for row in 1..=2000 {
        model._set(&format!("H{row}"), &format!("={}", 46000 + row));
    }
    model._set("A1", "=WORKDAY.INTL(46000, 7, 1, H1:H2000)");
    model._set("A2", "=WORKDAY.INTL(46000, -7, 1, H1:H2000)");
    model._set("A3", "=WORKDAY.INTL(46000, 7, 11, H1:H2000)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"48009");
    assert_eq!(model._get_text("A2"), *"45989");
    assert_eq!(model._get_text("A3"), *"48008");
}
