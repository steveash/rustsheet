//! ISO 8601 text of `t="d"` cells (Strict OOXML) → Excel serial.

use chrono::{NaiveDate, NaiveTime, Timelike};
use rustsheet_engine::formatter::dates::real_date_to_serial;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The serial Excel gives a `t="d"` cell holding `text` (`YYYY-MM-DD`, optionally followed by
/// `Thh:mm:ss[.fff]` and a trailing `Z`); `None` if it is not ISO 8601 or outside Excel's date range.
/// Evidence: fixtures/t-d-iso-date-cells (1990-01-01 = 32874, 1990-01-01T12:00:00 = 32874.5,
/// 1900-02-28 = 59, 1900-03-01 = 61); docs/excel-semantics/xlsx-storage.md. Excel rejects a
/// workbook holding unparsable `t="d"` text, so `None` has no Excel behavior to match.
#[must_use]
pub fn iso8601_to_serial(text: &str) -> Option<f64> {
    let text = text.trim();
    let (date_text, time_text) = match text.split_once('T') {
        Some((d, t)) => (d, Some(t.strip_suffix('Z').unwrap_or(t))),
        None => (text, None),
    };
    let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok()?;
    let serial = f64::from(real_date_to_serial(date)?);
    let Some(time_text) = time_text else {
        return Some(serial);
    };
    let time = NaiveTime::parse_from_str(time_text, "%H:%M:%S%.f").ok()?;
    let seconds = f64::from(time.num_seconds_from_midnight()) + f64::from(time.nanosecond()) / 1e9;
    Some(serial + seconds / SECONDS_PER_DAY)
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
    }

    #[test]
    fn rejects_unparsable_text() {
        assert_eq!(iso8601_to_serial("abc"), None);
        assert_eq!(iso8601_to_serial("1990-13-01"), None);
        assert_eq!(iso8601_to_serial("1899-12-30"), None);
        assert_eq!(iso8601_to_serial("1990-01-01T25:00:00"), None);
    }
}
