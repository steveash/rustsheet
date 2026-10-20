//! Parsing of en-US time text for TIMEVALUE (clock text, "date clock" text).
//!
//! Excel: see docs/excel-semantics/dates.md#time-text.

use chrono::{DateTime, NaiveDateTime, Timelike};

use crate::{
    cast::number_text_too_long,
    formatter::date_clock::{
        parse_clock_text, parse_date_clock_serial_in, parse_hour_only_am_pm, trim_spaces,
        SECONDS_PER_DAY_F64,
    },
    formatter::date_text::{parse_date, CurrentYear},
    locale::Locale,
    model::Model,
    types::DateSystem,
};

impl Model {
    /// [`parse_time_string`] with the model's locale, year and date system.
    pub(super) fn parse_time_text(&self, text: &str) -> Option<f64> {
        parse_time_string(
            text,
            self.locale,
            &|| self.current_year(),
            self.workbook.settings.date_system,
        )
    }
}

/// Time of day for TIMEVALUE text: the fractional part of the serial the text denotes, so clock
/// text past 24 hours wraps (`TIMEVALUE("25:00")` is 1.0416666666666667 - 1) and "date time" text
/// keeps the subtraction's float error; the date part is read in `date_system`, a pre-epoch 1904 day being no date (probes/date-system-1904-timevalue). Excel: see docs/excel-semantics/dates.md#time-text.
pub(super) fn parse_time_string(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<f64> {
    let raw = text;
    let text = trim_spaces(text);
    if let Some(serial) = parse_limited_clock_text(raw, text) {
        return Some(serial - serial.floor());
    }

    // "N PM" / "N AM" (no minutes)
    if let Some(serial) = parse_hour_only_am_pm(text) {
        return Some(serial);
    }

    let patterns_dt = [
        // ISO formats
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        // Excel-style date formats with AM/PM
        "%d-%b-%Y %I:%M:%S %p", // "22-Aug-2011 6:35:00 AM"
        "%d-%b-%Y %I:%M %p",    // "22-Aug-2011 6:35 AM"
        "%d-%b-%Y %H:%M:%S",    // "22-Aug-2011 06:35:00"
        "%d-%b-%Y %H:%M",       // "22-Aug-2011 06:35"
        // US date formats with AM/PM
        "%m/%d/%Y %I:%M:%S %p", // "8/22/2011 6:35:00 AM"
        "%m/%d/%Y %I:%M %p",    // "8/22/2011 6:35 AM"
        "%m/%d/%Y %H:%M:%S",    // "8/22/2011 06:35:00"
        "%m/%d/%Y %H:%M",       // "8/22/2011 06:35"
        // European date formats with AM/PM
        "%d/%m/%Y %I:%M:%S %p", // "22/8/2011 6:35:00 AM"
        "%d/%m/%Y %I:%M %p",    // "22/8/2011 6:35 AM"
        "%d/%m/%Y %H:%M:%S",    // "22/8/2011 06:35:00"
        "%d/%m/%Y %H:%M",       // "22/8/2011 06:35"
    ];
    for p in patterns_dt {
        if let Ok(dt) = NaiveDateTime::parse_from_str(text, p) {
            return crate::utils::time_of_day_from_datetime(dt, date_system);
        }
    }
    // "date clock" text the patterns above do not cover (fractional seconds, hours past 23,
    // M:S.f): date serial plus clock serial, then the fractional part (probe
    // timevalue-ampm-limits: TIMEVALUE("1/2/2020 25:00") is 0.04166666666424135).
    if let Some(serial) = parse_date_clock_serial_in(text, locale, current_year, date_system) {
        return Some(serial - serial.floor());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.time().num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
    }
    // Date-only text has no clock, so TIMEVALUE is 0, not #VALUE! (probes/date-text-glued-month-2
    // column C rows 6-8, 10-11, 13-15: `"Jul1 , 2026"`, `"Jun31"`). The date must be in range for
    // the date system, as in the date-clock path above.
    if let Ok((days, _)) = parse_date(text, locale, current_year) {
        return date_system
            .serial_from_1900_serial(i64::from(days))
            .map(|_| 0.0);
    }
    None
}

/// The serial a time or date-time text coerces to where `cast_number` reads text as a number
/// (`0+"10:11"`, `--"12:00"`): clock text, hour-only AM/PM and "date clock" text, unwrapped (`0+"25:00"`
/// is 1.0416666666666667, unlike TIMEVALUE which keeps the fraction). A leading space makes
/// the text invalid here although TIMEVALUE trims it (probes/timevalue-fractional-seconds2
/// C11, value-clock-text C14); trailing spaces are allowed (value-clock-text C34-C37).
/// Excel: see docs/excel-semantics/dates.md#time-text.
pub(crate) fn parse_time_serial(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<f64> {
    if text.starts_with(' ') {
        return None;
    }
    let raw = text;
    let text = text.trim_end_matches(' ');
    if let Some(serial) = parse_limited_clock_text(raw, text) {
        return Some(serial);
    }
    parse_hour_only_am_pm(text)
        .or_else(|| parse_date_clock_serial_in(text, locale, current_year, date_system))
}

/// [`parse_clock_text`] of `trimmed`, which is `raw` without its surrounding spaces: Excel rejects
/// clock text of more than 255 characters counted with those spaces
/// (probes/clock-text-length-limit).
fn parse_limited_clock_text(raw: &str, trimmed: &str) -> Option<f64> {
    if number_text_too_long(raw) {
        return None;
    }
    parse_clock_text(trimmed)
}
