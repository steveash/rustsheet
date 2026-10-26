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
        // The mantissa rounds after the shift (gen-m10 05c1c646c95a2d60 F!T12, 659cd5776d423ce3
        // F!J7; T-2226).
        (1.0 / 3.0, "##0.0E+0", "333.3E-3"),
        (2.0 / 3.0, "##0.0E+0", "666.7E-3"),
        (-1.0 / 7.0, "##0.0E+0", "-142.9E-3"),
        (1.0 / 3.0, "##0.00E+00", "333.33E-03"),
        (0.125, "00.0E+0", "12.5E-2"),
    ];
    for (value, code, expected) in cases {
        assert_eq!(format_number(value, code, &ctx).text, expected, "{code}");
    }
}

// Literals between `E+`/`E-` and the exponent digits: the `E` stays put and the sign moves to the
// digits; a zero mantissa keeps one integer digit (poi/NumberFormatApproxTests.xlsx, excel-graph,
// T-0937; gen-m10/5eb95c63c7d6180c F!G13 for `#E+0`).
#[test]
fn scientific_literals_before_the_exponent_digits() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (123_456.789, "|#.#|E-|#|", "|1.2|E|5|"),
        (123_456.789, "|#.#|E+|#|", "|1.2|E|+5|"),
        (1.234_567_89e-5, "|#.#|E-|#|", "|1.2|E|-5|"),
        (1.234_567_89e-5, "|#.#|E+|#|", "|1.2|E|-5|"),
        (0.0, "|#.#|E-|#|", "|0.|E|0|"),
        (0.0, "|#.#|E+|#|", "|0.|E|+0|"),
        (-123_456.789, "|#.#|E-|#|", "-|1.2|E|5|"),
        (-1.234_567_89e-5, "|#.#|E+|#|", "-|1.2|E|-5|"),
        (0.0, "#E+0", "0E+0"),
    ];
    for (value, code, expected) in cases {
        assert_eq!(
            format_number(value, code, &ctx).text,
            expected,
            "{code} {value}"
        );
    }
}

// A one-digit mantissa rounds the 15-digit decimal half away from zero, once: 2.5 is "3E+0", 1.005
// (the double 1.00499999…) "1.01E+00", and 12.345 "1.23E+01" (gen-m10 05c1c646c95a2d60 F!I10,
// F!L11, 5eb95c63c7d6180c F!G8, F!G19; probes/t2705-scientific-mantissa-rounding; T-2705).
#[test]
fn scientific_mantissa_rounds_the_15_digit_decimal_half_away() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (2.5, "0E+0", "3E+0"),
        (2.5, "#E+0", "3E+0"),
        (-2.5, "#E+0", "-3E+0"),
        (12.345, "0.00E+00", "1.23E+01"),
        (1.005, "0.00E+00", "1.01E+00"),
        (2.5 - 2f64.powi(-51), "0E+0", "3E+0"),
        (0.285, "0.0E+0", "2.9E-1"),
        (100.5, "0.0E+0", "1.0E+2"),
    ];
    for (value, code, expected) in cases {
        assert_eq!(
            format_number(value, code, &ctx).text,
            expected,
            "{value} {code}"
        );
    }
}

// A scientific code never scales: a `%` anywhere is a literal and a trailing `,` is dropped
// (probes/t2720-scientific-percent A1, A2, A7, A8, A15; excel-graph, T-2720).
#[test]
fn scientific_percent_and_comma_do_not_scale() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (2.675, "0.00E+00%", "2.68E+00%"),
        (2.675, "0.00%E+00", "2.68%E+00"),
        (0.5, "0%E+0", "5%E-1"),
        (2675.0, "0.00E+00,", "2.68E+03"),
        (2.675, "##0.0E+0%", "2.7E+0%"),
    ];
    for (value, code, expected) in cases {
        assert_eq!(
            format_number(value, code, &ctx).text,
            expected,
            "{value} {code}"
        );
    }
}
