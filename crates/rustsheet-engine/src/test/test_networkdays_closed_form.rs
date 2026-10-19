//! NETWORKDAYS and NETWORKDAYS.INTL count their span in closed form (T-1808). The differential test
//! keeps the per-day loops they replaced as the oracle.

use std::collections::HashSet;
use std::fmt::Write;
use std::time::{Duration, Instant};

use chrono::Datelike;

use crate::test::util::{new_empty_model, round_trip_value, Rng};
use crate::types::DateSystem;

const SAT_SUN: [bool; 7] = [false, false, false, false, false, true, true];

/// The weekend codes of NETWORKDAYS.INTL and their masks, Monday first, restated here rather than
/// read from `WeekendPattern` so the oracle stays independent of the code under test.
const WEEKEND_CODES: [(i32, [bool; 7]); 14] = [
    (1, SAT_SUN),
    (2, [true, false, false, false, false, false, true]),
    (3, [true, true, false, false, false, false, false]),
    (4, [false, true, true, false, false, false, false]),
    (5, [false, false, true, true, false, false, false]),
    (6, [false, false, false, true, true, false, false]),
    (7, [false, false, false, false, true, true, false]),
    (11, [false, false, false, false, false, false, true]),
    (12, [true, false, false, false, false, false, false]),
    (13, [false, true, false, false, false, false, false]),
    (14, [false, false, true, false, false, false, false]),
    (15, [false, false, false, true, false, false, false]),
    (16, [false, false, false, false, true, false, false]),
    (17, [false, false, false, false, false, true, false]),
];

/// The first and last serial with a date in `system`.
fn date_range(system: DateSystem) -> (i64, i64) {
    match system {
        DateSystem::Excel1900 => (1, 2_958_465),
        DateSystem::Excel1904 => (0, 2_957_003),
    }
}

fn is_weekend(date: chrono::NaiveDate, mask: &[bool; 7]) -> bool {
    mask[date.weekday().num_days_from_monday() as usize]
}

/// The NETWORKDAYS loop before T-1808: every day of the span is a date (#NUM! otherwise).
fn old_networkdays(
    system: DateSystem,
    start: i64,
    end: i64,
    holidays: &HashSet<i64>,
) -> Option<f64> {
    let (from, to, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    let mut count = 0i64;
    for serial in from..=to {
        let date = system.date_of_serial(serial)?;
        if !is_weekend(date, &SAT_SUN) && !holidays.contains(&serial) {
            count += 1;
        }
    }
    Some(count as f64 * sign)
}

/// The NETWORKDAYS.INTL loop before T-1808: both endpoints are dates (#NUM! otherwise), days
/// between without a date are skipped.
fn old_networkdays_intl(
    system: DateSystem,
    start: i64,
    end: i64,
    mask: &[bool; 7],
    holidays: &HashSet<i64>,
) -> Option<f64> {
    let (from, to, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    system.date_of_serial(from)?;
    system.date_of_serial(to)?;
    let mut count = 0i64;
    for serial in from..=to {
        if holidays.contains(&serial) {
            continue;
        }
        let Some(date) = system.date_of_serial(serial) else {
            continue;
        };
        if !is_weekend(date, mask) {
            count += 1;
        }
    }
    Some(count as f64 * sign)
}

fn expected_text(value: Option<f64>) -> String {
    value.map_or_else(|| "#NUM!".to_string(), |n| format!("{n:?}"))
}

/// A random serial near `around` (within `spread`), clamped to `[lo, hi]`.
fn near(rng: &mut Rng, around: i64, spread: u64, lo: i64, hi: i64) -> i64 {
    (around - spread as i64 + rng.below(2 * spread + 1) as i64).clamp(lo, hi)
}

/// One random case: start, end, holidays (all dates: invalid holidays are an error before the
/// span is read), and the formula's holiday argument.
fn random_case(rng: &mut Rng, system: DateSystem) -> (i64, i64, Vec<i64>) {
    let (first, last) = date_range(system);
    // Starts anywhere, next to either range edge (one to three serials past it included), and
    // around serial 60, the fictitious 1900-02-29.
    let start = match rng.below(4) {
        0 => first + rng.below((last - first + 1) as u64) as i64,
        1 => near(rng, first, 40, first - 3, last),
        2 => near(rng, last, 40, first, last + 3),
        _ => near(rng, 60, 40, first - 3, last),
    };
    let end = if rng.below(1000) == 0 {
        // The whole range, once in a while.
        if rng.below(2) == 0 {
            first
        } else {
            last
        }
    } else {
        start - 400 + rng.below(1201) as i64
    };
    let (lo, hi) = (start.min(end), start.max(end));
    let mut holidays: Vec<i64> = Vec::new();
    for _ in 0..rng.below(6) {
        let holiday = match rng.below(4) {
            0 => lo + rng.below((hi - lo + 1) as u64) as i64,
            1 => near(rng, lo, 10, first, last),
            2 => near(rng, hi, 10, first, last),
            _ => match holidays.last() {
                Some(&previous) => previous,
                None => near(rng, start, 30, first, last),
            },
        };
        holidays.push(holiday.clamp(first, last));
    }
    (start, end, holidays)
}

#[test]
fn closed_forms_match_the_per_day_loops() {
    let mut rng = Rng(0x1808_5eed_d1ff_7e57);
    for system in [DateSystem::Excel1900, DateSystem::Excel1904] {
        let mut model = new_empty_model();
        model.workbook.settings.date_system = system;
        let mut expected = Vec::new();
        for row in 1..=6_000 {
            let (start, end, holidays) = random_case(&mut rng, system);
            let holiday_set: HashSet<i64> = holidays.iter().copied().collect();
            let holiday_arg = if holidays.is_empty() {
                String::new()
            } else {
                let list: Vec<String> = holidays.iter().map(i64::to_string).collect();
                format!(",{{{}}}", list.join(","))
            };
            // Every mask shape: a weekend code or any of the 128 strings.
            let (weekend_arg, mask) = if rng.below(3) == 0 {
                let (code, mask) = WEEKEND_CODES[rng.below(14) as usize];
                (code.to_string(), mask)
            } else {
                let bits = rng.below(128);
                let mask: [bool; 7] = std::array::from_fn(|day| bits >> day & 1 == 1);
                let text = mask.iter().fold(String::new(), |mut text, &weekend| {
                    let _ = write!(text, "{}", u8::from(weekend));
                    text
                });
                (format!("\"{text}\""), mask)
            };
            let networkdays = format!("A{row}");
            let networkdays_intl = format!("B{row}");
            model._set(
                &networkdays,
                &format!("=NETWORKDAYS({start},{end}{holiday_arg})"),
            );
            let intl_holidays = if holidays.is_empty() && rng.below(2) == 0 {
                String::new()
            } else {
                format!(",{weekend_arg}{holiday_arg}")
            };
            let intl_mask = if intl_holidays.is_empty() {
                SAT_SUN
            } else {
                mask
            };
            model._set(
                &networkdays_intl,
                &format!("=NETWORKDAYS.INTL({start},{end}{intl_holidays})"),
            );
            expected.push((
                networkdays,
                expected_text(old_networkdays(system, start, end, &holiday_set)),
            ));
            expected.push((
                networkdays_intl,
                expected_text(old_networkdays_intl(
                    system,
                    start,
                    end,
                    &intl_mask,
                    &holiday_set,
                )),
            ));
        }
        model.evaluate();
        for (cell, value) in expected {
            assert_eq!(
                round_trip_value(&model, &cell),
                value,
                "{system:?} {cell}: {}",
                model._get_formula(&cell)
            );
        }
    }
}

#[test]
fn whole_range_spans_cost_no_per_day_loop() {
    // The per-day loops took about 2.96M steps per call, minutes for these 2,000 calls; the
    // closed forms take well under a millisecond each (about 8 ms in all in a debug build), so
    // the bound leaves room for a loaded runner.
    let mut model = new_empty_model();
    for row in 1..=1_000 {
        model._set(&format!("A{row}"), "=NETWORKDAYS(1,2958465)");
        model._set(
            &format!("B{row}"),
            "=NETWORKDAYS.INTL(1,2958465,\"0000011\")",
        );
    }
    let started = Instant::now();
    model.evaluate();
    assert!(started.elapsed() < Duration::from_secs(5));
    let (first, last) = date_range(DateSystem::Excel1900);
    let whole_range = expected_text(old_networkdays(
        DateSystem::Excel1900,
        first,
        last,
        &HashSet::new(),
    ));
    assert_eq!(round_trip_value(&model, "A1000"), whole_range);
    assert_eq!(round_trip_value(&model, "B1000"), whole_range);
}
