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
    // The imported (fractional) form keeps the time of day and drops a moment before the epoch.
    assert_eq!(
        Excel1904.serial_from_imported_1900_serial(1462.75),
        Some(0.75)
    );
    assert_eq!(Excel1904.serial_from_imported_1900_serial(1461.5), None);
    assert_eq!(Excel1904.serial_from_imported_1900_serial(0.5), Some(0.5));
    assert_eq!(Excel1904.serial_from_imported_1900_serial(f64::NAN), None);
    assert_eq!(
        Excel1900.serial_from_imported_1900_serial(45292.5),
        Some(45292.5)
    );
    // EDATE/EOMONTH keep a day before the 1904 epoch as a negative serial.
    assert_eq!(Excel1904.signed_serial_from_1900_serial(1431), -31);
    assert_eq!(Excel1900.signed_serial_from_1900_serial(1431), 1431);
    // 1904 serial 0 is Friday 1904-01-01; a negative 1904 serial is no date.
    assert_eq!(
        Excel1904.date_of_serial_from_zero(0),
        chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
    );
    assert_eq!(Excel1904.date_of_serial_from_zero(-1), None);
    // WEEKDAY, WORKDAY and NETWORKDAYS also read 1900 serial 0, Saturday 1899-12-30 on the
    // weekday-faithful calendar (T-1254).
    assert_eq!(
        Excel1900.date_of_serial_from_zero(0),
        chrono::NaiveDate::from_ymd_opt(1899, 12, 30)
    );
    assert_eq!(
        Excel1904.date_of_serial_from_zero(0),
        chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
    );
    assert_eq!(Excel1900.date_of_serial_from_zero(-1), None);
    assert_eq!(Excel1904.date_of_serial_from_zero(-1), None);
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

// ACCRINT with calc_method FALSE takes a settlement up to the last 1904 date, 2957003, though it
// reads past 9999-12-31 (probes/date-system-1904-financial-settlement-bound, T-1492, T-1494).
#[test]
fn accrint_settlement_limit_is_the_last_1904_date_read_for_calc_false() {
    use DateSystem::{Excel1900, Excel1904};
    assert_eq!(Excel1900.accrint_settlement_limit(false), 2_958_465);
    assert_eq!(
        Excel1904.accrint_settlement_limit(false),
        Excel1904.financial_1900_serial(2_957_003.0) as i64
    );
    assert_eq!(Excel1904.accrint_settlement_limit(true), 2_958_465);
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

// poi/ElapsedFormatTests.xlsx (a 1904 workbook) A2-A27, A42-A51 and poi/DateFormatTests.xlsx A13,
// A32, A40 (T-0934, T-0981): fractional seconds and plain `h`/`s` after an elapsed token.
#[test]
fn fractional_and_repeated_elapsed_seconds() {
    // The workbook's 3.14159, written as a quotient (the same double) for clippy's PI lint.
    let value = 314_159.0 / 100_000.0;
    let two_03_04 = 2.0 / 24.0 + 3.0 / (24.0 * 60.0) + 4.0 / (24.0 * 60.0 * 60.0);
    let cases = [
        (value, "[h]:m:s.000", "75:23:53.376"),
        (value, "[m]:s.000", "4523:53.376"),
        (value, "[s].000", "271433.376"),
        (value, "[hh]:mm:ss.0", "75:23:53.4"),
        (value, "[ss].000", "271433.376"),
        (-value, "[h]:m:s.000", "-75:23:53.376"),
        (two_03_04, "[hh]:mm:ss.0", "02:03:04.0"),
        (
            two_03_04,
            "[s]\" [yes, \"ss\"] seconds\"",
            "7384 [yes, 7384] seconds",
        ),
        (0.0, "[ss].000", "00.000"),
        (0.0, "[s]\" [yes, \"ss\"] seconds\"", "0 [yes, 00] seconds"),
        (
            value,
            "\"It was \"[h]\" [yes, \"h\"] hours and \"mm:ss",
            "It was 75 [yes, 75] hours and 23:53",
        ),
        (
            -value,
            "[s]\" [yes, \"ss\"] seconds\"",
            "-271433 [yes, 271433] seconds",
        ),
        // Clock formats round the second to the shown digits (04:05:06.008999934).
        (36191.1702084375, "hh:mm:ss.000 am/pm", "04:05:06.009 AM"),
        (36191.1702084375, "h:m:s.00", "4:5:6.01"),
        (17816.607951388887, "hh:mm:ss.000 am/pm", "02:35:27.000 PM"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1904),
            expected,
            "{value} {format}"
        );
    }
    // gen-m10 T!A16, T!A20: past 9999-12-31 a fraction of a second is #VALUE!; four zeros are
    // unprobed and stay an error.
    for (value, format) in [
        (1_234_567_890.0, "[mm]:ss.0"),
        (1_234_567_890.0, "h:mm:ss.00"),
        (1.0, "s.0000"),
    ] {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1900),
            "#VALUE!",
            "{value} {format}"
        );
    }
}

// A date Excel cannot show displays as 255 `#` while TEXT is #VALUE! (probes/displayed-text-formats
// S!B4: `yyyy-mm-dd` on a negative serial; gen-m10 "Displayed text": dates past 9999-12-31 and
// elapsed codes with a clock token on a negative). A bare elapsed code keeps its minus.
#[test]
fn unshowable_date_displays_hashes() {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    let hashes = "#".repeat(255);
    for (value, format) in [
        (-1.0, "yyyy-mm-dd"),
        (1_234_567_890.0, "m/d/yyyy"),
        (-0.5, "[h]:mm"),
        (1_234_567_890.0, "h:mm:ss.00"),
    ] {
        let formatted = format_number(value, format, &ctx);
        assert_eq!(formatted.text, "#VALUE!", "{format} on {value}");
        assert_eq!(formatted.displayed(), hashes, "{format} on {value}");
    }
    assert_eq!(format_number(-1.5, "[s]", &ctx).displayed(), "-129600");
    assert_eq!(
        format_number(45000.0, "yyyy-mm-dd", &ctx).displayed(),
        "2023-03-15"
    );
}

// gen-m10 displayed text (T-1732): fields come from the serial rounded to the shown second.
#[test]
fn fields_come_from_the_serial_rounded_to_the_second() {
    let v = 0.999_999_999_9;
    let cases = [
        ("h:mm", "0:00"),
        ("hh:mm:ss", "00:00:00"),
        ("[h]:mm:ss", "24:00:00"),
        ("h:mm:ss AM/PM", "12:00:00 AM"),
        ("yyyy-mm-dd", "1900-01-01"),
        ("dddd", "Sunday"),
        ("d \\d\\a\\y\\s h:mm", "1 days 0:00"),
        ("[m]:ss", "1440:00"),
        ("dd/mm/yy", "01/01/00"),
    ];
    for (format, expected) in cases {
        assert_eq!(
            format_in(v, format, DateSystem::Excel1900),
            expected,
            "{format}"
        );
    }
}

#[test]
fn fraction_digits_round_into_the_next_day() {
    // 0.99999999999 is 23:59:59.999999; `.000` rounds it up to midnight of the next date.
    assert_eq!(
        format_in(
            0.999_999_999_99,
            "yyyy-mm-dd hh:mm:ss.000",
            DateSystem::Excel1900
        ),
        "1900-01-01 00:00:00.000"
    );
}

// probes/text-format-aaa-weekday and -b (T-1880): three or more of one `a` are the weekday.
#[test]
fn a_runs_are_the_weekday() {
    let cases = [
        (1.0, "aaa", "Sun"),
        (1.0, "aaaa", "Sunday"),
        (1.0, "aaaaaaaaaa", "Sunday"),
        (1.0, "AAA", "Sun"),
        (1.0, "AAAA", "Sunday"),
        (1.0, "aa", "aa"),
        (1.0, "aAa", "aAa"),
        (1.0, "AAa", "AAa"),
        (1.0, "aaa\\a", "Suna"),
        (1.0, "aaaA", "SunA"),
        (1.0, "Aaaa", "ASun"),
        (1.0, "aaaAAA", "Sunday"),
        (1.0, "aaAA", "aaAA"),
        (45000.0, "aaa", "Wed"),
        (0.75, "aaa AM/PM", "Sat PM"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_in(value, format, DateSystem::Excel1900),
            expected,
            "{format}"
        );
    }
}
