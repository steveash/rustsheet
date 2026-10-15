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
fn simple_test() {
    let ctx = FormatContext::new(get_default_locale());
    let bond_james_bond = format_number(7.0, "000", &ctx);
    assert_eq!(bond_james_bond.text, "007");
}

#[test]
fn test_general() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(7.0, "General", &ctx).text, "7");
}

#[test]
fn simple_test_comma() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(1007.0, "000", &ctx).text, "1007");
    assert_eq!(format_number(1008.0, "#", &ctx).text, "1008");
    assert_eq!(format_number(1009.0, "#,#", &ctx).text, "1,009");
    assert_eq!(format_number(12_345_678.0, "#,#", &ctx).text, "12,345,678");
    assert_eq!(format_number(12_345_678.0, "0,0", &ctx).text, "12,345,678");
    assert_eq!(format_number(1005.0, "00-00", &ctx).text, "10-05");
    assert_eq!(format_number(7.0, "0?0", &ctx).text, "0 7");
    assert_eq!(format_number(7.0, "0#0", &ctx).text, "07");
    assert_eq!(
        format_number(1234.0, "000 \"Millions\"", &ctx).text,
        "1234 Millions"
    );
    assert_eq!(
        format_number(1235.0, "#,000 \"Millions\"", &ctx).text,
        "1,235 Millions"
    );
    assert_eq!(format_number(1007.0, "0,00", &ctx).text, "1,007");
    assert_eq!(format_number(10_000_007.0, "0,00", &ctx).text, "10,000,007");
}

#[test]
fn test_negative_numbers() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(-123.0, "0.0", &ctx).text, "-123.0");
    assert_eq!(format_number(-3.0, "000.0", &ctx).text, "-003.0");
    assert_eq!(format_number(-0.00001, "000.0", &ctx).text, "000.0");
}

#[test]
fn test_decimal_part() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(3.1, "0.00", &ctx).text, "3.10");
    // assert_eq!(format_number(3.1, "00-.-0?0", &ctx).text, "03-.-1 0");
}

#[test]
fn test_color() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(3.1, "[blue]0.00", &ctx).text, "3.10");
    assert_eq!(format_number(3.1, "[blue]0.00", &ctx).color, Some(4));

    assert_eq!(format_number(3.1, "[Color42]0.00", &ctx).text, "3.10");
    assert_eq!(format_number(3.1, "[Color42]0.00", &ctx).color, Some(42));
}

#[test]
fn test_failed_to_parse_color() {
    let ctx = FormatContext::new(get_default_locale());
    let error = Some("Problem parsing format string".to_string());
    assert_eq!(format_number(3.1, "[foo]0.00", &ctx).error, error);
    assert_eq!(format_number(3.1, "[Color999]0.00", &ctx).error, error);
    assert_eq!(format_number(3.1, "[Color42.5]0.00", &ctx).error, error);
}

#[test]
fn test_conditional() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(42.00, "[=]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[<]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[>]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[<=]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[>=]0.00", &ctx).text, "42.00");
}

#[test]
fn test_failed_to_parse_conditional() {
    let ctx = FormatContext::new(get_default_locale());
    let error = Some("Problem parsing format string".to_string());

    assert_eq!(format_number(42.00, "[=0.00", &ctx).error, error);
    assert_eq!(format_number(42.00, "[=", &ctx).error, error);
}

#[test]
fn test_string_double_quota() {
    // Excel: a doubled quote ends one literal and starts the next; it is not an escaped quote
    // (probes/format-doubled-quotes C1, C2).
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(
        format_number(42.00, "0 \"\"\"Millions\"\"\"", &ctx).text,
        "42 Millions"
    );
    assert_eq!(format_number(42.00, "0 \"a\"\"b\"", &ctx).text, "42 ab");
}

#[test]
fn test_failed_to_parse_string() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(
        format_number(42.00, "0 \"\"\"Millions\"\"", &ctx).error,
        Some("Problem parsing format string".to_string())
    );
}

#[test]
fn test_exponential_conditional() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(42.00, "[=0.0e0]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[=0.0e00]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[=0.0e+0]0.00", &ctx).text, "42.00");
    assert_eq!(format_number(42.00, "[=0.0e-0]0.00", &ctx).text, "42.00");
}

#[test]
fn test_failed_to_parse_exponential_conditional() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(
        format_number(42.00, "[=0.0ea]0.00", &ctx).error,
        Some("Problem parsing format string".to_string())
    );
}

#[test]
fn dollar_euro() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[$€]#,##0.00";
    let t = format_number(3.1, format, &ctx);
    assert_eq!(t.text, "€3.10");
}

#[test]
fn test_currency_valid_formats() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(2.34, "[$€]#,##0.00", &ctx).text, "€2.34");
    assert_eq!(format_number(3.0, "[$€]#,##0.00", &ctx).text, "€3.00");
    assert_eq!(format_number(42.0, "[$€]0", &ctx).text, "€42");
}

#[test]
fn test_failed_to_parse_currency() {
    let ctx = FormatContext::new(get_default_locale());
    let error = Some("Problem parsing format string".to_string());
    assert_eq!(format_number(42.00, "[$", &ctx).error, error);
    assert_eq!(format_number(42.00, "[$€", &ctx).error, error);
    assert_eq!(format_number(42.00, "[$€0", &ctx).error, error);
}

#[test]
fn test_failed_to_parse_bracket() {
    let ctx = FormatContext::new(get_default_locale());
    let error = Some("Problem parsing format string".to_string());
    assert_eq!(format_number(42.0, "[", &ctx).error, error);
}

#[test]
fn test_parts() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(3.1, "0.00;(0.00);(-)", &ctx).text, "3.10");
    assert_eq!(format_number(-3.1, "0.00;(0.00);(-)", &ctx).text, "(3.10)");
    assert_eq!(format_number(0.0, "0.00;(0.00);(-)", &ctx).text, "(-)");
}

#[test]
fn test_zero() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(0.0, "$#,##0", &ctx).text, "$0");
    assert_eq!(format_number(-1.0 / 3.0, "0", &ctx).text, "0");
    assert_eq!(format_number(-1.0 / 3.0, "0;(0)", &ctx).text, "(0)");
}

#[test]
fn test_negative_currencies() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(-23.0, "$#,##0", &ctx).text, "-$23");
}

#[test]
fn test_percent() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(0.12, "0.00%", &ctx).text, "12.00%");
    assert_eq!(format_number(0.12, "0.00%%", &ctx).text, "1200.00%%");
}

#[test]
fn test_percent_correct_rounding() {
    let ctx = FormatContext::new(get_default_locale());
    // Formatting does Excel rounding (15 significant digits)
    assert_eq!(
        format_number(0.1399999999999999, "0.0%", &ctx).text,
        "14.0%"
    );
    assert_eq!(
        format_number(-0.1399999999999999, "0.0%", &ctx).text,
        "-14.0%"
    );
    // Formatting does proper rounding
    assert_eq!(format_number(0.1399, "0.0%", &ctx).text, "14.0%");
    assert_eq!(format_number(-0.1399, "0.0%", &ctx).text, "-14.0%");
    assert_eq!(format_number(0.02666, "0.00%", &ctx).text, "2.67%");
    assert_eq!(format_number(0.0266, "0.00%", &ctx).text, "2.66%");
    assert_eq!(format_number(0.0233, "0.00%", &ctx).text, "2.33%");
    assert_eq!(format_number(0.02666, "0%", &ctx).text, "3%");
    assert_eq!(format_number(-0.02666, "0.00%", &ctx).text, "-2.67%");
    assert_eq!(format_number(-0.02666, "0%", &ctx).text, "-3%");

    // precision 0
    assert_eq!(format_number(0.135, "0%", &ctx).text, "14%");
    assert_eq!(format_number(0.13499, "0%", &ctx).text, "13%");
    assert_eq!(format_number(-0.135, "0%", &ctx).text, "-14%");
    assert_eq!(format_number(-0.13499, "0%", &ctx).text, "-13%");

    // precision 1
    assert_eq!(format_number(0.1345, "0.0%", &ctx).text, "13.5%");
    assert_eq!(format_number(0.1343, "0.0%", &ctx).text, "13.4%");
    assert_eq!(format_number(-0.1345, "0.0%", &ctx).text, "-13.5%");
    assert_eq!(format_number(-0.134499, "0.0%", &ctx).text, "-13.4%");
}

#[test]
fn test_scientific() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(2.5e-14, "0.00E+0", &ctx).text, "2.50E-14");
    assert_eq!(format_number(3e-4, "0.00E+00", &ctx).text, "3.00E-04");
}

#[test]
fn test_currency() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(123.1, "$#,##0", &ctx).text, "$123");
    assert_eq!(format_number(123.1, "#,##0 €", &ctx).text, "123 €");
}

#[test]
fn test_date() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(
        format_number(41181.0, "dd/mm/yyyy", &ctx).text,
        "29/09/2012"
    );
    assert_eq!(
        format_number(41181.0, "dd-mm-yyyy", &ctx).text,
        "29-09-2012"
    );
    assert_eq!(
        format_number(41304.0, "dd-mm-yyyy", &ctx).text,
        "30-01-2013"
    );
    assert_eq!(
        format_number(42657.0, "dd-mm-yyyy", &ctx).text,
        "14-10-2016"
    );

    assert_eq!(
        format_number(41181.0, "dddd-mmmm-yyyy", &ctx).text,
        "Saturday-September-2012"
    );
    assert_eq!(
        format_number(41181.0, "ddd-mmm-yy", &ctx).text,
        "Sat-Sep-12"
    );
    assert_eq!(
        format_number(41181.0, "ddd-mmmmm-yy", &ctx).text,
        "Sat-S-12"
    );
    assert_eq!(
        format_number(41181.0, "ddd-mmmmmmm-yy", &ctx).text,
        "Sat-September-12"
    );
}

#[test]
fn test_german_locale() {
    let ctx = FormatContext::new(get_locale("de").expect(""));
    assert_eq!(format_number(1234.56, "General", &ctx).text, "1234,56");
}

// Excel evidence: poi/NumberFormatTests.xlsx Tests!A5, A18, A22, A107 (docs/excel-semantics/number-formats.md).
#[test]
fn pound_yen_and_pipe_are_unquoted_literals() {
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(12.3, "£000.00", &ctx).text, "£012.30");
    assert_eq!(
        format_number(-12.3, "\"before \"¥#.00", &ctx).text,
        "-before ¥12.30"
    );
    assert_eq!(format_number(1.1, "|?.?|", &ctx).text, "|1.1|");
}

#[test]
fn general_fixed_versus_scientific() {
    // T-0938: probes/number-text-exponent-digits D8, poi/GeneralFormatTests (A13, A17, A18, A26-A28,
    // A61) and probes/text-general-exponent-boundary A9, A10 (excel-graph).
    let ctx = FormatContext::new(get_default_locale());
    let general = |value: f64| format_number(value, "General", &ctx).text;
    assert_eq!(general(1.234_567_890_123_45E-5), "1.23457E-05");
    assert_eq!(general(1E-9), "0.000000001");
    assert_eq!(general(-1E-9), "-0.000000001");
    assert_eq!(general(1E-10), "1E-10");
    assert_eq!(general(9.999_999_6E-10), "1E-09");
    assert_eq!(general(0.01), "0.01");
    assert_eq!(general(1.234_567_891_2), "1.234567891");
    // Unprobed 6-against-6 tie (T-1225): the engine prints fixed.
    assert_eq!(general(1.123_456_789E-4), "0.000112346");
    assert_eq!(general(1E10), "10000000000");
    assert_eq!(general(-1E10), "-10000000000");
    assert_eq!(general(9.999_999_6E10), "99999996000");
    assert_eq!(general(1E11), "1E+11");
    assert_eq!(general(1.123_456_789E-10), "1.12346E-10");
}

#[test]
fn colon_runs_print_as_one() {
    // probes/text-format-length-limit-f A30:A39, -g A42:A46 (T-1434).
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(1.0, "0::", &ctx).text, "1:");
    assert_eq!(format_number(1.0, "d:::", &ctx).text, "1:");
    assert_eq!(format_number(1.0, "0: :", &ctx).text, "1: :");
    assert_eq!(format_number(1.0, "d:\"\":", &ctx).text, "1::");
}

#[test]
fn single_text_section_prints_a_number_in_general() {
    // probes/text-format-length-limit-i A19:A26 (T-1434).
    let ctx = FormatContext::new(get_default_locale());
    assert_eq!(format_number(1.0, "@\"aaa\"", &ctx).text, "1");
    assert_eq!(format_number(1.5, "@", &ctx).text, "1.5");
}

/// Bare letters and digits 1-9 are literals, a point with no placeholder before it shows the
/// integer part, and an empty format shows only a minus (probes/text-format-bare-literals, T-1262).
#[test]
fn bare_letters_digits_and_empty_format() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (std::f64::consts::PI, "x", "x"),
        (-std::f64::consts::PI, "x", "-x"),
        (1.0, "0a0", "0a1"),
        (1234.0, "0x00", "12x34"),
        (-1.0, "2.5", "-21.5"),
        (1.005, "-21.5", "-211.5"),
        (12.3, ".", "12."),
        (0.0, ".", "."),
        (12.3, "0.1", "12.1"),
        (12.3, "1.0", "112.3"),
        (5.0, "", ""),
        (-0.0001, "", "-"),
        (5.0, "General x", "5 x"),
        (5.0, "x General", "x 5"),
        (1.0, "0\\:\\:", "1::"),
        (1.0, "0::a", "1:a"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{format}"
        );
    }
}

/// `b` is the Buddhist-era year, `e` the year, `g` the (empty) era and `B1`/`B2` print nothing;
/// a lone closing `B` and `B2` before a date token are errors (probes/text-format-bare-literals).
#[test]
fn buddhist_year_era_and_calendar_tokens() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        ("ABC", "A43C"),
        ("bb", "43"),
        ("bbbb", "2443"),
        ("e", "1900"),
        ("e+", "1900+"),
        ("g", ""),
        ("B1", ""),
        ("B2", ""),
        ("x d", "x 9"),
    ];
    for (format, expected) in cases {
        assert_eq!(
            format_number(100.0, format, &ctx).text,
            expected,
            "{format}"
        );
    }
    for format in ["B", "dB", "a.b", "n", "E", "B2d", "0 General"] {
        assert!(
            format_number(100.0, format, &ctx).error.is_some(),
            "{format}"
        );
    }
}
