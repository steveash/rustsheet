use crate::formatter::dates::convert_to_serial_number;

#[derive(Clone)]
pub(crate) struct Tz(chrono_tz::Tz);

impl Tz {
    pub(crate) fn parse(s: &str) -> Result<Tz, String> {
        s.parse::<chrono_tz::Tz>()
            .map(Tz)
            .map_err(|_| format!("Invalid timezone: {s}"))
    }
}

pub fn get_all_timezone_names() -> Vec<String> {
    chrono_tz::TZ_VARIANTS
        .iter()
        .map(|t| t.name().to_string())
        .collect()
}

/// The Excel serial of the instant `millis` (ms since the Unix epoch) in `tz`.
pub(crate) fn excel_serial_for_now(tz: &Tz, millis: i64) -> Option<f64> {
    use chrono::{DateTime, Timelike};
    DateTime::from_timestamp(millis.div_euclid(1000), 0).map(|dt| {
        let local = dt.with_timezone(&tz.0);
        // Days are numbered as DATE numbers them, fictitious 1900-02-29 included
        // (docs/excel-semantics/dates.md "Serial 60 is the fictitious 1900-02-29").
        let days = convert_to_serial_number(local.date_naive());
        // Excel's NOW() keeps sub-second precision (see the cached NOW cells in
        // eval/suites/ironcalc-calc DATE_AND_TIME/NOW_TODAY.xlsx), so the fraction carries milliseconds.
        let seconds =
            f64::from(local.num_seconds_from_midnight()) + millis.rem_euclid(1000) as f64 / 1000.0;
        let fraction = seconds / (60.0 * 60.0 * 24.0);
        days as f64 + fraction
    })
}

/// The calendar year in `tz` of the instant `millis` (ms since the Unix epoch).
pub(crate) fn current_year(tz: &Tz, millis: i64) -> Option<i32> {
    use chrono::{DateTime, Datelike};
    DateTime::from_timestamp(millis.div_euclid(1000), 0).map(|dt| dt.with_timezone(&tz.0).year())
}
