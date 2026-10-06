//! Parsing of en-US time text for TIMEVALUE (clock text, "date clock" text).
//!
//! Excel: see docs/excel-semantics/dates.md#time-text.

use chrono::{DateTime, NaiveDateTime, NaiveTime, Timelike};

use super::SECONDS_PER_DAY_F64;
use crate::{formatter::date_text::parse_date, locale::Locale};

/// Time of day for TIMEVALUE text: the fractional part of the serial the text denotes, so clock
/// text past 24 hours wraps (`TIMEVALUE("25:00")` is 1.0416666666666667 - 1) and "date time" text
/// keeps the subtraction's float error. Excel: see docs/excel-semantics/dates.md#time-text.
pub(super) fn parse_time_string(text: &str, locale: &Locale) -> Option<f64> {
    let text = trim_spaces(text);
    if let Some(serial) = parse_clock_text(text) {
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
            return Some(crate::utils::time_of_day_from_datetime(dt));
        }
    }
    // "date clock" text the patterns above do not cover (fractional seconds, hours past 23,
    // M:S.f): date serial plus clock serial, then the fractional part (probe
    // timevalue-ampm-limits: TIMEVALUE("1/2/2020 25:00") is 0.04166666666424135).
    if let Some(serial) = parse_date_clock_serial(text, locale) {
        return Some(serial - serial.floor());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.time().num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
    }
    None
}

/// The serial a time or date-time text coerces to where `cast_number` reads text as a number
/// (`0+"10:11"`, `--"12:00"`): clock text, hour-only AM/PM and "date clock" text, unwrapped (`0+"25:00"`
/// is 1.0416666666666667, unlike TIMEVALUE which keeps the fraction). A leading space makes
/// the text invalid here although TIMEVALUE trims it (probes/timevalue-fractional-seconds2
/// C11, value-clock-text C14); trailing spaces are allowed (value-clock-text C34-C37).
/// Excel: see docs/excel-semantics/dates.md#time-text.
pub(crate) fn parse_time_serial(text: &str, locale: &Locale) -> Option<f64> {
    if text.starts_with(' ') {
        return None;
    }
    let text = text.trim_end_matches(' ');
    if let Some(serial) = parse_clock_text(text) {
        return Some(serial);
    }
    parse_hour_only_am_pm(text).or_else(|| parse_date_clock_serial(text, locale))
}

/// Hour-only "N AM" / "N PM" text (hour 0-12, no minutes) as a time-of-day serial: `"0 PM"` is
/// noon and `"0 AM"` midnight, `"13 PM"` is #VALUE! (probes/value-clock-text rows 9-11, 28).
fn parse_hour_only_am_pm(text: &str) -> Option<f64> {
    let (hour_str, is_pm) = parse_simple_am_pm(text)?;
    let hour = hour_str.parse::<u32>().ok()?;
    if hour > 12 {
        return None;
    }
    let time = NaiveTime::from_hms_opt(hour_from_12h(hour, is_pm), 0, 0)?;
    Some(time.num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64)
}

/// "Date clock" text (`"1/2/2020 10:11:12.5"`, `"July 1, 2026 6 PM"`) as the unwrapped serial:
/// date serial plus clock serial. The clock is the last word, or the last two when they end in
/// AM/PM, since date text has spaces of its own (probes/date-text-grammar C30, C31, C55).
pub(crate) fn parse_date_clock_serial(text: &str, locale: &Locale) -> Option<f64> {
    let mut date_end = text.len();
    for _ in 0..2 {
        date_end = text[..date_end].trim_end().rfind(char::is_whitespace)?;
        let clock_part = text[date_end..].trim();
        let clock = parse_clock_text(clock_part).or_else(|| parse_hour_only_am_pm(clock_part));
        if let (Some(clock), Ok((days, _))) =
            (clock, parse_date(text[..date_end].trim_end(), locale))
        {
            return Some(f64::from(days) + clock);
        }
    }
    None
}

/// Time text is trimmed and split on the space character only: a tab, a no-break space or an
/// ideographic space anywhere is #VALUE! (`"10:00\t"`, `"10:00\u{a0}AM"`, `"\u{3000}10:00"`:
/// probes/value-clock-text rows 38, 42-45).
fn trim_spaces(text: &str) -> &str {
    text.trim_matches(' ')
}

/// Largest value of one clock component: Excel takes at most four digits
/// (`TIMEVALUE("9999:59:59")` parses, `"10000:00:00"` is #VALUE!).
const MAX_CLOCK_COMPONENT: u32 = 9_999;

/// One `H`, `M` or `S` component of clock text: 1-4 ASCII digits, surrounding spaces allowed,
/// a leading zero only in a 2-digit component (`"10:05"` parses, `"010:11:12"` and `"10:011:12"`
/// are #VALUE!).
fn parse_clock_component(part: &str) -> Option<u32> {
    let part = trim_spaces(part);
    let digits_ok = (1..=4).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit());
    if !digits_ok || (part.len() > 2 && part.starts_with('0')) {
        return None;
    }
    part.parse::<u32>()
        .ok()
        .filter(|v| *v <= MAX_CLOCK_COMPONENT)
}

/// Milliseconds of a fractional-seconds digit string (ASCII digits only), rounded half-up on the decimal digits:
/// `.1234` is 123 ms, `.4995` is 500 ms, `.9999` is 1000 ms (probes timevalue-fractional-seconds*).
fn fraction_digits_to_ms(digits: &str) -> u32 {
    let digit = |i: usize| digits.as_bytes().get(i).map_or(0, |b| u32::from(b - b'0'));
    let ms = digit(0) * 100 + digit(1) * 10 + digit(2);
    if digit(3) >= 5 {
        ms + 1
    } else {
        ms
    }
}

/// Whole seconds plus milliseconds of the `f` in `H:M.f:S` clock text: the digits are read twice,
/// once as whole seconds and once as the fraction (`"10:11.25:12"` is 10:11 plus 25.25 s,
/// `"10:11.0123:12"` plus 123.012 s; probes/clock-text-oddities*). `None` past seven
/// significant digits, which no clock Excel accepts reaches (see [`minutes_with_seconds_fit`]).
fn fraction_as_seconds(digits: &str) -> Option<(u64, u32)> {
    let significant = digits.trim_start_matches('0');
    if significant.len() > 7 {
        return None;
    }
    let whole = if significant.is_empty() {
        0
    } else {
        significant.parse::<u64>().ok()?
    };
    Some((whole, fraction_digits_to_ms(digits)))
}

/// Whether the minutes plus the whole minutes in the (rounded) seconds stay within
/// [`MAX_CLOCK_COMPONENT`]: `"0:11.9999:1"` (11 min, 10000 s) parses but `"0:9999.9998:1"`
/// (9999 min, 9999 s), `"0:9998.9999:1"` and `"0:9999:9999.9999"` are #VALUE!, while
/// `"0:9999.5:1"` (9999 min, 5.5 s) parses (probes/clock-text-oddities2 A20,
/// clock-text-oddities3 A1-A9, A16).
fn minutes_with_seconds_fit(minute: u32, second_ms: u64) -> bool {
    u64::from(minute) + second_ms / 60_000 <= u64::from(MAX_CLOCK_COMPONENT)
}

/// Serial value of clock text in Excel's en-US grammar, before any wrap to one day:
/// `H:M`, `H:M:S[.f]`, `M:S.f` (two components with a fraction are minutes and seconds) or the
/// odd `H:M.f:S` (see [`fraction_as_seconds`]; `S` is checked and then ignored), spaces allowed
/// around `:` and `.`, one trailing `:` ignored, components up to 9999 (minutes plus the
/// seconds' whole minutes too), an optional ` AM`/` PM`/` A`/` P` suffix after whitespace
/// (then the hour is at most 12 and minutes and whole seconds at most 59). The value is
/// `(total_ms / 1000) / 86400`, which matches Excel bit for bit (`"10:11:12.5"` is
/// 0.4244502314814815). Excel: see docs/excel-semantics/dates.md#time-text.
fn parse_clock_text(text: &str) -> Option<f64> {
    let text = trim_spaces(text);
    let (body, suffix_is_pm) = match text.rsplit_once(' ') {
        Some((body, word)) if ["am", "a"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(false))
        }
        Some((body, word)) if ["pm", "p"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(true))
        }
        _ => (text, None),
    };
    // `"10:11:"` is 10:11 and `"10:11:12.5:"` is 10:11:12.5, but `"10:11.5::"` is #VALUE!
    // (probes/clock-text-oddities3 A11-A14).
    let body = trim_spaces(body);
    let body = body.strip_suffix(':').unwrap_or(body);
    let (head, fraction, odd_tail) = match body.split_once('.') {
        Some((head, rest)) => {
            let (fraction, odd_tail) = match rest.split_once(':') {
                Some((fraction, tail)) => (fraction, Some(tail)),
                None => (rest, None),
            };
            let fraction = fraction.trim();
            if !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (head, Some(fraction), odd_tail)
        }
        None => (body, None, None),
    };
    let parts = head
        .split(':')
        .map(parse_clock_component)
        .collect::<Option<Vec<u32>>>()?;
    let has_fraction_digits = fraction.is_some_and(|f| !f.is_empty());
    let ((hour, minute), (second, ms)) = match (parts.as_slice(), fraction, odd_tail) {
        ([h, m], Some(f), Some(tail)) if has_fraction_digits => {
            parse_clock_component(tail)?;
            ((*h, *m), fraction_as_seconds(f)?)
        }
        (_, _, Some(_)) => return None,
        ([h, m, s], ..) => (
            (*h, *m),
            (u64::from(*s), fraction.map_or(0, fraction_digits_to_ms)),
        ),
        ([m, s], Some(f), None) if has_fraction_digits => {
            ((0, *m), (u64::from(*s), fraction_digits_to_ms(f)))
        }
        ([h, m], ..) => ((*h, *m), (0, 0)),
        _ => return None,
    };
    let hour = match suffix_is_pm {
        Some(is_pm) => {
            if hour > 12 || minute > 59 || second > 59 {
                return None;
            }
            hour_from_12h(hour, is_pm)
        }
        None => hour,
    };
    let second_ms = second * 1000 + u64::from(ms);
    if !minutes_with_seconds_fit(minute, second_ms) {
        return None;
    }
    let total_ms = (u64::from(hour) * 60 + u64::from(minute)) * 60_000 + second_ms;
    // Clock values stay far below 2^53 ms (at most 9999 h plus 9999 min plus 59 s), so the cast is
    // exact.
    let seconds = total_ms as f64 / 1000.0;
    Some(seconds / SECONDS_PER_DAY_F64)
}

/// 24-hour value of a 12-hour clock hour (0-12): PM adds 12 to hours below 12 (`"0:11:12.5 PM"`
/// is 12:11:12.5, probes/timevalue-component-limits B16), AM maps 12 to 0.
fn hour_from_12h(hour: u32, is_pm: bool) -> u32 {
    match (is_pm, hour) {
        (false, 12) => 0,
        (true, h) if h < 12 => h + 12,
        (_, h) => h,
    }
}

/// Splits hour-only "N PM" / "N AM" text into the hour digits and whether it is PM. The suffix is
/// `AM`, `PM`, `A` or `P` in any case with an optional trailing `.` (`"6 P."`, `"6 a"`:
/// probes/value-clock-text rows 7, 25, 29-33).
fn parse_simple_am_pm(text: &str) -> Option<(&str, bool)> {
    let (hour_part, suffix) = text.rsplit_once(' ')?;

    let suffix = suffix.strip_suffix('.').unwrap_or(suffix);

    let is_pm = if suffix.eq_ignore_ascii_case("pm") || suffix.eq_ignore_ascii_case("p") {
        true
    } else if suffix.eq_ignore_ascii_case("am") || suffix.eq_ignore_ascii_case("a") {
        false
    } else {
        return None;
    };

    let hour_part = trim_spaces(hour_part);

    if !hour_part.is_empty() && hour_part.chars().all(|c| c.is_ascii_digit()) {
        return Some((hour_part, is_pm));
    }
    None
}
