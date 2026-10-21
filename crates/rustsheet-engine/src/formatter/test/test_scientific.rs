#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn scientific_minus_negative() {
    let ctx = FormatContext::new(get_default_locale());
    let b = format_number(0.000002, "0.00E-00", &ctx);
    assert_eq!(b.text, "2.00E-06");
}

#[test]
fn scientific_minus_positive() {
    let ctx = FormatContext::new(get_default_locale());
    let b = format_number(2_000_000.0, "0.00E-00", &ctx);
    assert_eq!(b.text, "2.00E06");
}

#[test]
fn scientific_positive() {
    let ctx = FormatContext::new(get_default_locale());
    let b = format_number(2_000_000.0, "0.00E+00", &ctx);
    assert_eq!(b.text, "2.00E+06");
}

#[test]
fn scientific_negative() {
    let ctx = FormatContext::new(get_default_locale());
    let b = format_number(0.000002, "0.00E+00", &ctx);
    assert_eq!(b.text, "2.00E-06");
}

// Excel cases from eval/suites/gen-m10 (excel-graph, T-1730).
#[test]
fn scientific_zero_exponent_and_engineering() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (1.5, "0.00E+00", "1.50E+00"),
        (1.5, "0.000E-00", "1.500E00"),
        (123_456.789, "##0.0E+0", "123.5E+3"),
        (61.0, "##0.0E+0", "61.0E+0"),
        (36.5, "00.0E+0", "36.5E+0"),
        (2_958_465.999_99, "00.0E+0", "03.0E+6"),
    ];
    for (value, code, expected) in cases {
        assert_eq!(format_number(value, code, &ctx).text, expected, "{code}");
    }
}
