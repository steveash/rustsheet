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

/// A `*x` fill adds nothing to a number section; `_x` still adds a space (T-1735).
#[test]
fn fill_adds_nothing_but_ghost_adds_a_space() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(3.0, "*.0.00", &ctx).text, "3.00");
    assert_eq!(format_number(7.0, "0*\"a", &ctx).text, "7a");
    assert_eq!(format_number(1000.0, "_(* #,##0_)", &ctx).text, " 1,000 ");
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
