#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn simple_test() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "h:mm AM/PM";
    let value = 16.001_423_611_111_11; // =1/86400 => 12:02 AM
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "12:02 AM");
}

#[test]
fn padded_vs_unpadded() {
    let ctx = FormatContext::new(get_default_locale());
    let padded_format = "hh:mm:ss AM/PM";
    let unpadded_format = "h:m:s AM/PM";
    let value = 0.25351851851851853; // => 6:05:04 AM (21904/(24*60*60)) where 21904 = 6 * 3600 + 5*60 + 4
    let formatted = format_number(value, padded_format, &ctx);
    assert_eq!(formatted.text, "06:05:04 AM");

    let formatted = format_number(value, unpadded_format, &ctx);
    assert_eq!(formatted.text, "6:5:4 AM");
}

#[test]
fn elapsed_hour() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[hh]:mm:ss";
    // 1.5 days = 36 hours
    let value = 1.5;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "36:00:00");
}

#[test]
fn elapsed_minute() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[mm]:ss";
    // 2 days = 2880 minutes
    let value = 2.0;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "2880:00");
}

#[test]
fn elapsed_second() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[ss]";
    // 0.5 days = 43200 seconds
    let value = 0.5;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "43200");
}

#[test]
fn elapsed_hour_padded() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[hh]:mm:ss";
    // 0.1 days = 2.4 hours
    let value = 0.1;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "02:24:00");
}

/// gen-m10 and gen-m5 (T-1731): a clock token over a serial past 9999-12-31 is unshowable, a lone
/// elapsed token is not.
#[test]
fn clock_past_last_date_is_unshowable() {
    let ctx = FormatContext::new(get_default_locale());
    for format in ["h:mm", "[h]:mm", "[m]:ss", "h AM/PM", "h:mm:ss AM/PM"] {
        let formatted = format_number(1_234_567_890.0, format, &ctx);
        assert!(formatted.unshowable, "{format}");
    }
    let formatted = format_number(1e15, "[s]", &ctx);
    assert!(!formatted.unshowable);
    assert_eq!(formatted.text, "86400000000000000000");
}

#[test]
fn minutes_before_seconds_without_hour() {
    // `mm` directly before `:ss` is minutes even with no hour in the code
    // (gen-m10 text cells, excel-graph 2026-10-16, T-1733).
    let ctx = FormatContext::new(get_default_locale());
    for (value, format, expected) in [
        (1000.0, "mm:ss", "00:00"),
        (123_456.789, "mm:ss", "56:10"),
        (1.5, "mm:ss", "00:00"),
        // Unprobed in Excel, same rule.
        (123_456.789, "m:ss", "56:10"),
    ] {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{format}"
        );
    }
}

// `[h]` and `[m]` without a seconds token count from the serial rounded to the second, as the
// clock does (probes/elapsed-no-seconds-boundary A1-A8, A20, T-2385).
#[test]
fn elapsed_without_seconds_rounds_to_the_second() {
    let ctx = FormatContext::new(get_default_locale());
    let short = |hours: f64, seconds: f64| (hours - seconds / 3600.0) / 24.0;
    for (value, format, text) in [
        (short(3.0, 0.4), "[h]", "3"),
        (short(3.0, 0.4), "[h]:mm", "3:00"),
        (short(3.0, 0.4), "[m]", "180"),
        (short(3.0, 0.6), "[h]", "2"),
        (short(3.0, 0.6), "[h]:mm", "2:59"),
        (short(3.0, 0.6), "[m]", "179"),
        ((5.0 - 0.4 / 60.0) / 1440.0, "[m]", "5"),
    ] {
        assert_eq!(format_number(value, format, &ctx).text, text, "{format}");
    }
}
