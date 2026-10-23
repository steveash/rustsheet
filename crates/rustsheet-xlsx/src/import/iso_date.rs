//! ISO 8601 text of `t="d"` cells (Strict OOXML) → Excel serial.

use chrono::{NaiveDate, NaiveTime, Timelike};
use rustsheet_engine::formatter::dates::real_date_to_serial;
use rustsheet_engine::types::DateSystem;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The serial Excel gives a `t="d"` cell holding `text` (`YYYY-MM-DD`, optionally followed by
/// `Thh:mm:ss[.fff]` and a trailing `Z`), or `hh:mm:ss[.fff]` alone (a fraction of a day); `None` if
/// it is not ISO 8601, before 1900-01-01 or after 9999-12-31 (only the first date bound is probed).
/// Evidence: fixtures/t-d-iso-date-cells (1990-01-01 = 32874, 1990-01-01T12:00:00 = 32874.5,
/// 1900-02-28 = 59, 1900-03-01 = 61), t-d-z-suffix, t-d-fractional-seconds, t-d-1900-02-29 (60),
/// t-d-time-only (0.5); docs/excel-semantics/xlsx-storage.md. A date before 1900-01-01 reads as
/// `#VALUE!` (t-d-pre-1900), so the importer turns `None` into that error. Excel rejects a
/// workbook holding other unparsable `t="d"` text, so those have no Excel behavior to match.
#[must_use]
pub fn iso8601_to_serial(text: &str) -> Option<f64> {
    let text = text.trim();
    if !text.contains('-') {
        return time_fraction(text);
    }
    let (date_text, time_text) = match text.split_once('T') {
        Some((d, t)) => (d, Some(t.strip_suffix('Z').unwrap_or(t))),
        None => (text, None),
    };
    // Excel's 1900 leap-year bug: serial 60 is the nonexistent 1900-02-29 (t-d-1900-02-29).
    let serial = if date_text == "1900-02-29" {
        60.0
    } else {
        let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok()?;
        f64::from(real_date_to_serial(date)?)
    };
    let Some(time_text) = time_text else {
        return Some(serial);
    };
    Some(serial + time_fraction(time_text)?)
}

/// The serial a `t="d"` cell or a pivot cache `<d>` item holding `text` stands for in a workbook
/// of `date_system`: `iso8601_to_serial` moved to that system. In a 1904 workbook `2024-01-01` is
/// 43830, a time of day alone keeps its day fraction, and a date before 1904-01-01 has no serial,
/// which a cell reads as `#VALUE!` (fixtures/t-d-1904, t-d-1904-epoch, t-d-1904-time-only,
/// t-d-1904-pre-epoch, t-d-1904-1900-01-01; docs/excel-semantics/xlsx-storage.md).
#[must_use]
pub fn iso8601_to_date_system_serial(text: &str, date_system: DateSystem) -> Option<f64> {
    iso8601_to_serial(text).and_then(|s| date_system.serial_from_imported_1900_serial(s))
}

/// `hh:mm:ss[.fff]` as a fraction of a day.
fn time_fraction(text: &str) -> Option<f64> {
    let time = NaiveTime::parse_from_str(text, "%H:%M:%S%.f").ok()?;
    let seconds = f64::from(time.num_seconds_from_midnight()) + f64::from(time.nanosecond()) / 1e9;
    Some(seconds / SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::iso8601_to_serial;

    #[test]
    fn matches_excel_serials() {
        assert_eq!(iso8601_to_serial("1990-01-01"), Some(32874.0));
        assert_eq!(iso8601_to_serial("1990-01-01T12:00:00"), Some(32874.5));
        assert_eq!(iso8601_to_serial("1990-01-01T12:00:00Z"), Some(32874.5));
        assert_eq!(iso8601_to_serial("1900-02-28"), Some(59.0));
        assert_eq!(iso8601_to_serial("1900-03-01"), Some(61.0));
        assert_eq!(iso8601_to_serial("1900-02-29"), Some(60.0));
        assert_eq!(iso8601_to_serial("1900-02-29T12:00:00"), Some(60.5));
        assert_eq!(iso8601_to_serial("12:00:00"), Some(0.5));
        assert_eq!(
            iso8601_to_serial("1990-01-01T12:00:00.5"),
            Some(32_874.500_005_787_035)
        );
    }

    #[test]
    fn rejects_unparsable_text() {
        assert_eq!(iso8601_to_serial("abc"), None);
        assert_eq!(iso8601_to_serial("1990-13-01"), None);
        assert_eq!(iso8601_to_serial("1899-12-30"), None);
        assert_eq!(iso8601_to_serial("1899-12-31"), None);
        assert_eq!(iso8601_to_serial("1990-01-01T25:00:00"), None);
    }
}
