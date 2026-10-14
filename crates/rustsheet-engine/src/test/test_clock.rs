#![allow(clippy::unwrap_used)]

//! The per-model clock (SPEC-0008 test plan, engine part).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::cell::CellValue;
use crate::clock::Clock;
use crate::model::Model;
use crate::test::util::{fixed_clock, new_empty_model, TEST_CLOCK_MS};

/// 1900-01-01T00:00:00Z and 9999-12-31T23:59:59.999Z, the bounds of a fixed clock.
const MIN_MS: i64 = -2_208_988_800_000;
const MAX_MS: i64 = 253_402_300_799_999;
const DAY_MS: i64 = 86_400_000;

fn model_at(tz: &str, ms: i64) -> Model {
    Model::new_empty_with_clock("model", "en", tz, "en", fixed_clock(ms)).unwrap()
}

/// The value of `Sheet1!A{row}`.
fn value(model: &Model, row: i32) -> CellValue {
    model.get_cell_value_by_index(0, row, 1).unwrap()
}

fn number(model: &Model, row: i32) -> f64 {
    match value(model, row) {
        CellValue::Number(n) => n,
        other => panic!("A{row} is {other:?}, not a number"),
    }
}

#[test]
fn two_models_on_one_thread_keep_their_own_clock() {
    let mut epoch = model_at("UTC", 0);
    let mut next_day = model_at("UTC", DAY_MS);
    for model in [&mut epoch, &mut next_day] {
        model._set("A1", "=NOW()");
        model.evaluate();
    }
    assert_eq!(number(&epoch, 1), 25569.0);
    assert_eq!(number(&next_day, 1), 25570.0);
}

#[test]
fn default_clock_is_the_system_time() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    assert_eq!(model.clock(), Clock::system());
    model._set("A1", "=NOW()");
    model.evaluate();
    let host_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64;
    let host_serial = 25569.0 + host_ms / DAY_MS as f64;
    let seconds_apart = (number(&model, 1) - host_serial).abs() * 86_400.0;
    assert!(seconds_apart < 5.0, "{seconds_apart} s");
}

#[test]
fn set_clock_does_not_evaluate() {
    let mut model = model_at("UTC", 0);
    model._set("A1", "=NOW()");
    model.evaluate();
    model.set_clock(fixed_clock(DAY_MS));
    assert_eq!(number(&model, 1), 25569.0);
    model.evaluate();
    assert_eq!(number(&model, 1), 25570.0);
}

#[test]
fn today_numbers_early_1900_days_as_date_does() {
    // DATE counts Excel's fictitious 1900-02-29 as serial 60 (dates.md).
    for (ms, date, serial) in [
        (MIN_MS, "DATE(1900,1,1)", 1.0),
        (MIN_MS + 58 * DAY_MS, "DATE(1900,2,28)", 59.0),
        (MIN_MS + 59 * DAY_MS, "DATE(1900,3,1)", 61.0),
    ] {
        let mut model = model_at("UTC", ms);
        model._set("A1", "=TODAY()");
        model._set("A2", &format!("={date}"));
        model._set("A3", "=NOW()");
        model.evaluate();
        assert_eq!(number(&model, 1), serial, "{date}");
        assert_eq!(number(&model, 2), serial, "{date}");
        assert_eq!(number(&model, 3), serial, "{date}");
    }
}

#[test]
fn now_and_today_at_the_bounds_in_utc() {
    let mut model = model_at("UTC", MAX_MS);
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();
    assert_eq!(number(&model, 1), 2_958_465.0);
    assert!(number(&model, 2) < 2_958_466.0);
    assert!(number(&model, 2) > 2_958_465.999);

    let mut model = model_at("UTC", MIN_MS);
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();
    assert_eq!(number(&model, 1), 1.0);
    assert_eq!(number(&model, 2), 1.0);
}

/// A timezone offset can move the local date one day outside Excel's calendar. The result is a
/// number or `#ERROR!`, never a panic; Excel cannot reach such a date, so this is no parity
/// claim (SPEC-0008).
#[test]
fn now_and_today_at_the_bounds_in_far_timezones() {
    let cases = [
        // UTC+14: the upper bound is 10000-01-01 locally, one day past DATE's last serial.
        // Excel never returns 2958466; this pins only that there is no panic.
        ("Pacific/Kiritimati", MAX_MS, CellValue::Number(2_958_466.0)),
        // Kiritimati was west of UTC in 1900 (its tzdata LMT is -10:29:20), so 1899-12-31.
        ("Pacific/Kiritimati", MIN_MS, CellValue::Number(0.0)),
        ("Pacific/Pago_Pago", MAX_MS, CellValue::Number(2_958_465.0)),
        // UTC-11: the lower bound is 1899-12-31 locally, serial 0.
        ("Pacific/Pago_Pago", MIN_MS, CellValue::Number(0.0)),
    ];
    for (tz, ms, today) in cases {
        let mut model = model_at(tz, ms);
        model._set("A1", "=TODAY()");
        model._set("A2", "=NOW()");
        model.evaluate();
        assert_eq!(value(&model, 1), today, "TODAY() in {tz} at {ms}");
        let now = number(&model, 2);
        let CellValue::Number(today) = today else {
            unreachable!()
        };
        assert!(
            (today..today + 1.0).contains(&now),
            "NOW() {now} in {tz} at {ms}"
        );
    }
}

#[test]
fn year_less_text_reads_the_clock_year() {
    // 2030-06-15T00:00:00Z.
    let in_2030 = 1_907_712_000_000;
    for (mut model, year) in [
        (new_empty_model(), 2022.0),
        (model_at("UTC", TEST_CLOCK_MS), 2022.0),
        (model_at("UTC", in_2030), 2030.0),
    ] {
        model._set("A1", "=YEAR(\"7/1\"+0)");
        model.evaluate();
        assert_eq!(number(&model, 1), year);
    }
}
