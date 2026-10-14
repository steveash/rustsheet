#![allow(clippy::unwrap_used)]

use crate::test::util::{fixed_clock, new_empty_model};

// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));

    model._set("A1", "=NOW(1, 1)");
    model._set("A2", "=NOW(\"Europe/Berlin\")");
    model._set("A3", "=NOW(\"faketimezone\")");
    model.evaluate();

    assert_eq!(
        model._get_text("A1"),
        "#ERROR!",
        "Wrong number of arguments"
    );
    assert_eq!(model._get_text("A2"), *"3/20/2023, 2:44 PM");
    assert_eq!(
        model._get_text("A3"),
        "#VALUE!",
        "Invalid timezone: faketimezone"
    );
}

#[test]
fn returns_date_time() {
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model._set("A1", "=NOW()");
    model.evaluate();
    let text = model._get_text("A1");
    assert_eq!(text, *"3/20/2023, 1:44 PM");
}

#[test]
fn keeps_sub_second_precision() {
    // 13:44:25.208 UTC: Excel's NOW() is not truncated to whole seconds.
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model._set("A1", "=ROUND((NOW()-INT(NOW()))*86400, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"49465.208");
}
