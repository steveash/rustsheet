#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::{get_locale, Locale},
    types::DateSystem,
};

fn get_default_locale() -> &'static Locale {
    get_locale("de").unwrap()
}

#[test]
fn simple_test() {
    let locale = get_default_locale();
    let b = format_number(46015.0, "m.d.yy", &FormatContext::new(locale));
    assert_eq!(b.text, "12.24.25");
}

/// `format_number` of `value` with `format` in the en locale and the given date system.
fn format_in(value: f64, format: &str, date_system: DateSystem) -> String {
    let ctx = FormatContext {
        locale: get_locale("en").unwrap(),
        date_system,
    };
    format_number(value, format, &ctx).text
}

// probes/negative-date-sections-1900 and probes/date-system-1900-negatives (T-1105, T-1268).
#[test]
fn negative_serials_1900() {
    let cases = [
        (-1.5, "[h]", "-36"),
        (-1.5, "[h]\" h\"", "-36 h"),
        (-1.5, "\"x\"[h]", "-x36"),
        (-1.5, "[Red][h]", "-36"),
        (-1.5, "[h];[h]", "36"),
        (-1.5, "0;[h]", "36"),
        (-1E-10, "[s]", "-0"),
        (-0.0, "[h]:mm", "0:00"),
        (-1.5, "[h]:mm", "#VALUE!"),
        (-1.5, "[h] d", "#VALUE!"),
        (-1.5, "[s].00", "#VALUE!"),
        (-1.5, "[h]:mm;-[h]:mm", "#VALUE!"),
        (-1.0, "d;d", "#VALUE!"),
        (-1.5, "[h];d", "#VALUE!"),
        (-1.5, "0.0;d", "#VALUE!"),
        (-0.5, "h:mm", "#VALUE!"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1900),
            expected,
            "{value} {format}"
        );
    }
}

// probes/date-system-1904-negatives, negative-date-sections-1904 and poi ElapsedFormatTests A40.
#[test]
fn negative_serials_1904() {
    let cases = [
        (-1.5, "[h]:mm", "-36:00"),
        (-1.5, "[mm]:ss", "-2160:00"),
        (-1E-10, "[h]:mm", "-0:00"),
        (-0.5, "h:mm", "-12:00"),
        (-1.5, "[h]:mm;-[h]:mm", "-36:00"),
        (-1.5, "\"x\"[h]", "-x36"),
        (-1.5, "[h];[h]", "36"),
        // Date tokens read the 1904 epoch (T-1330).
        (-1.5, "[h] d", "-36 2"),
        (-1.5, "yyyy", "-1904"),
        (-1.0, "d", "-2"),
        (-1.0, "d;d", "2"),
        (0.0, "yyyy-mm-dd", "1904-01-01"),
        (1.0, "yyyy-mm-dd", "1904-01-02"),
        (59.0, "yyyy-mm-dd", "1904-02-29"),
        // 1904-01-01 was a Friday (poi DateFormatTests.xlsx, a 1904 workbook, caches
        // "Saturday-October-1952" through the same weekday rule).
        (0.0, "dddd ddd", "Friday Fri"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1904),
            expected,
            "{value} {format}"
        );
    }
}

// The 1904 epoch shift is checked: no serial overflows, and no 1904 day precedes 1904-01-01.
#[test]
fn date_system_serial_conversion() {
    use DateSystem::{Excel1900, Excel1904};
    assert_eq!(Excel1904.to_1900_serial(0), Some(1462));
    assert_eq!(Excel1904.to_1900_serial(-1), None);
    assert_eq!(Excel1904.to_1900_serial(i64::MAX), None);
    assert_eq!(Excel1904.serial_from_1900_serial(1462), Some(0));
    assert_eq!(Excel1904.serial_from_1900_serial(1461), None);
    assert_eq!(Excel1904.serial_from_1900_serial(i64::MIN), None);
    assert_eq!(Excel1900.to_1900_serial(-5), Some(-5));
    assert_eq!(Excel1900.serial_from_1900_serial(-5), Some(-5));
    // EDATE/EOMONTH keep a day before the 1904 epoch as a negative serial.
    assert_eq!(Excel1904.signed_serial_from_1900_serial(1431), -31);
    assert_eq!(Excel1900.signed_serial_from_1900_serial(1431), 1431);
    // 1904 serial 0 is Friday 1904-01-01; a negative 1904 serial is no date.
    assert_eq!(
        Excel1904.date_of_serial(0),
        chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
    );
    assert_eq!(Excel1904.date_of_serial(-1), None);
    // The last day Excel accepts, 9999-12-31, is 1900 serial 2958465 and 1904 serial 2957003.
    assert_eq!(
        format_in(2_957_003.0, "yyyy-mm-dd", Excel1904),
        "9999-12-31"
    );
    assert_eq!(format_in(2_957_004.0, "yyyy-mm-dd", Excel1904), "#VALUE!");
}

// The financial day counts read a 1904 serial 2 * 1462 days on and return their dates the same
// number of days earlier (T-1492); a negative 1904 serial stays negative (no date).
#[test]
fn financial_1900_serial_reads_two_epochs_on() {
    use DateSystem::{Excel1900, Excel1904};
    assert_eq!(Excel1904.financial_1900_serial(0.0), 2924.0);
    assert_eq!(Excel1904.financial_1900_serial(10.5), 2934.5);
    assert_eq!(Excel1904.financial_1900_serial(-1.0), -1.0);
    assert_eq!(Excel1900.financial_1900_serial(45000.0), 45000.0);
    assert_eq!(Excel1904.serial_from_financial_1900_serial(2923), -1);
    assert_eq!(Excel1900.serial_from_financial_1900_serial(45000), 45000);
}

// ACCRINT takes a settlement up to the last 1904 date, 2957003, though it reads past 9999-12-31
// (probes/date-system-1904-financial-settlement-bound, T-1492).
#[test]
fn accrint_settlement_limit_is_the_last_1904_date_read() {
    use DateSystem::{Excel1900, Excel1904};
    assert_eq!(Excel1900.accrint_settlement_limit(), 2_958_465);
    assert_eq!(
        Excel1904.accrint_settlement_limit(),
        Excel1904.financial_1900_serial(2_957_003.0) as i64
    );
}

// The odd-period functions read a 1904 serial 2 * 1462 days on, so the last 1904 serial they read
// as a date is 2958465 - 2924; a negative 1904 serial stays negative (T-1457).
#[test]
fn odd_period_1900_serial_reads_two_epochs_on() {
    use DateSystem::{Excel1900, Excel1904};
    assert_eq!(Excel1904.odd_period_1900_serial(0.5), Some(2924));
    assert_eq!(
        Excel1904.odd_period_1900_serial(2_955_541.0),
        Some(2_958_465)
    );
    assert_eq!(Excel1904.odd_period_1900_serial(2_955_542.0), None);
    assert_eq!(Excel1904.odd_period_1900_serial(1e300), None);
    assert_eq!(Excel1904.odd_period_1900_serial(-1.0), Some(-1));
    assert_eq!(Excel1900.odd_period_1900_serial(45000.7), Some(45000));
}
