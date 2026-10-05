//! Parsing of en-US time text for TIMEVALUE (clock text, "date clock" text).
//!
//! Excel: see docs/excel-semantics/dates.md#time-text.

use chrono::{DateTime, NaiveDateTime, NaiveTime, Timelike};

use super::{parse_datevalue_text, SECONDS_PER_DAY_F64};

/// Time of day for TIMEVALUE text: the fractional part of the serial the text denotes, so clock
/// text past 24 hours wraps (`TIMEVALUE("25:00")` is 1.0416666666666667 - 1) and "date time" text
/// keeps the subtraction's float error. Excel: see docs/excel-semantics/dates.md#time-text.
pub(super) fn parse_time_string(text: &str) -> Option<f64> {
    let text = text.trim();
    if let Some(serial) = parse_clock_text(text) {
        return Some(serial - serial.floor());
    }

    // "N PM" / "N AM" (no minutes)
    if let Some((hour_str, is_pm)) = parse_simple_am_pm(text) {
        if let Ok(hour) = hour_str.parse::<u32>() {
            if (1..=12).contains(&hour) {
                let hour_24 = if is_pm {
                    if hour == 12 {
                        12
                    } else {
                        hour + 12
                    }
                } else if hour == 12 {
                    0
                } else {
                    hour
                };
                let time = NaiveTime::from_hms_opt(hour_24, 0, 0)?;
                return Some(time.num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
            }
        }
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
    if let Some((date_part, clock_part)) = text.split_once(char::is_whitespace) {
        if let (Ok(days), Some(clock)) = (
            parse_datevalue_text(date_part),
            parse_clock_text(clock_part),
        ) {
            let serial = f64::from(days) + clock;
            return Some(serial - serial.floor());
        }
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.time().num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
    }
    None
}

/// Largest value of one clock component: Excel takes at most four digits
/// (`TIMEVALUE("9999:59:59")` parses, `"10000:00:00"` is #VALUE!).
const MAX_CLOCK_COMPONENT: u32 = 9_999;

/// One `H`, `M` or `S` component of clock text: 1-4 ASCII digits, surrounding spaces allowed,
/// a leading zero only in a 2-digit component (`"10:05"` parses, `"010:11:12"` and `"10:011:12"`
/// are #VALUE!).
fn parse_clock_component(part: &str) -> Option<u32> {
    let part = part.trim();
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

/// Serial value of clock text in Excel's en-US grammar, before any wrap to one day:
/// `H:M`, `H:M:S[.f]` or `M:S.f` (two components with a fraction are minutes and seconds),
/// spaces allowed around `:` and `.`, components up to 9999, an optional ` AM`/` PM`/` A`/` P`
/// suffix after whitespace (then the hour is at most 12 and minutes and seconds at most 59).
/// The value is `(total_ms / 1000) / 86400`, which matches Excel bit for bit
/// (`"10:11:12.5"` is 0.4244502314814815). Excel: see docs/excel-semantics/dates.md#time-text.
fn parse_clock_text(text: &str) -> Option<f64> {
    let text = text.trim();
    let (body, suffix_is_pm) = match text.rsplit_once(char::is_whitespace) {
        Some((body, word)) if ["am", "a"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(false))
        }
        Some((body, word)) if ["pm", "p"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(true))
        }
        _ => (text, None),
    };
    let (head, fraction) = match body.split_once('.') {
        Some((head, fraction)) => {
            let fraction = fraction.trim();
            if !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (head, Some(fraction))
        }
        None => (body, None),
    };
    let parts = head
        .split(':')
        .map(parse_clock_component)
        .collect::<Option<Vec<u32>>>()?;
    let has_fraction_digits = fraction.is_some_and(|f| !f.is_empty());
    let (hour, minute, second) = match parts.as_slice() {
        [h, m, s] => (*h, *m, *s),
        [m, s] if has_fraction_digits => (0, *m, *s),
        [h, m] => (*h, *m, 0),
        _ => return None,
    };
    let hour = match suffix_is_pm {
        Some(is_pm) => {
            if hour > 12 || minute > 59 || second > 59 {
                return None;
            }
            match (is_pm, hour) {
                (false, 12) => 0,
                (true, h) if h < 12 => h + 12,
                (_, h) => h,
            }
        }
        None => hour,
    };
    let ms = fraction.map_or(0, fraction_digits_to_ms);
    let total_ms = ((u64::from(hour) * 60 + u64::from(minute)) * 60 + u64::from(second)) * 1000
        + u64::from(ms);
    // Clock values stay far below 2^53 ms (at most 9999:9999:9999.999), so the cast is exact.
    let seconds = total_ms as f64 / 1000.0;
    Some(seconds / SECONDS_PER_DAY_F64)
}

// Helper function to parse simple "N PM" / "N AM" formats (case-insensitive)
fn parse_simple_am_pm(text: &str) -> Option<(&str, bool)> {
    let (hour_part, suffix) = text.rsplit_once(char::is_whitespace)?;

    let suffix = suffix.strip_suffix('.').unwrap_or(suffix);

    let is_pm = if suffix.eq_ignore_ascii_case("pm") {
        true
    } else if suffix.eq_ignore_ascii_case("am") {
        false
    } else {
        return None;
    };

    let hour_part = hour_part.trim();

    if !hour_part.is_empty() && hour_part.chars().all(|c| c.is_ascii_digit()) {
        return Some((hour_part, is_pm));
    }
    None
}
