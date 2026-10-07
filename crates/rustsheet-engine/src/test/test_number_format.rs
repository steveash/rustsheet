#![allow(clippy::unwrap_used)]

use crate::number_format::{format_number, parse_excel_number};
use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn test_simple_format() {
    let formatted = format_number(2.3, "General", "en");
    assert_eq!(formatted.text, "2.3".to_string());
}

// T-0986: General keeps 6 significant digits in E notation, 5 from an exponent magnitude of 99
// (probes/number-text-exponent-digits D1:D15).
#[test]
fn general_drops_a_digit_at_exponent_magnitude_99() {
    let cases = [
        (1.234_567_890_123_45e98, "1.23457E+98"),
        (1.234_567_890_123_45e99, "1.2346E+99"),
        (1.234_567_890_123_45e100, "1.2346E+100"),
        (1.234_567_890_123_45e307, "1.2346E+307"),
        (1.234_567_890_123_45e-98, "1.23457E-98"),
        (1.234_567_890_123_45e-99, "1.2346E-99"),
        (1.234_567_890_123_45e-300, "1.2346E-300"),
    ];
    for (value, text) in cases {
        assert_eq!(format_number(value, "General", "en").text, text, "{value}");
    }
}

// T-1001: a mantissa that rounds up to 10 carries into the exponent
// (probes/text-general-exponent-boundary A5-A8, A10).
#[test]
fn general_carries_a_rounded_up_mantissa_into_the_exponent() {
    let cases = [
        (9.999_999_6e98, "1E+99"),
        (9.999_999_6e99, "1E+100"),
        (9.999_999_6e-98, "1E-97"),
        (9.999_999_6e-99, "1E-98"),
        (9.999_999_6e-10, "1E-09"),
        (-9.999_999_6e98, "-1E+99"),
    ];
    for (value, text) in cases {
        assert_eq!(format_number(value, "General", "en").text, text, "{value}");
    }
}

#[test]
fn test_maximum_zeros() {
    let formatted = format_number(1.0 / 3.0, "#,##0.0000000000000000000", "en");
    assert_eq!(formatted.text, "0.3333333333333330000".to_string());

    let formatted = format_number(1234.0 + 1.0 / 3.0, "#,##0.0000000000000000000", "en");
    assert_eq!(formatted.text, "1,234.3333333333300000000".to_string());
}

#[test]
fn test_leading_comma_text() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, ",10").unwrap(); // A1
    model.set_user_input(0, 2, 1, ",100").unwrap(); // A2
    model.set_user_input(0, 3, 1, ",1000").unwrap(); // A3

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok(",10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok(",100".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok(",1000".to_string())
    );
}

#[test]
fn test_wrong_locale() {
    let formatted = format_number(2.3, "General", "ens");
    assert_eq!(formatted.text, "#ERROR!".to_string());
}

#[test]
fn parse_excel_number_truncates_to_15_digits() {
    // Integer digits past 15 become zeros, fraction digits past 15 are dropped (not rounded).
    assert_eq!(
        parse_excel_number("123456789012345678"),
        Some(123_456_789_012_345_000.0)
    );
    assert_eq!(
        parse_excel_number("-123456789012345678"),
        Some(-123_456_789_012_345_000.0)
    );
    assert_eq!(
        parse_excel_number("12345.6789012345678"),
        Some(12345.6789012345)
    );
    // Leading zeros are not significant.
    assert_eq!(
        parse_excel_number("0.000123456789012345678"),
        Some(0.000123456789012345)
    );
    // The exponent is kept.
    assert_eq!(
        parse_excel_number("1.23456789012345678E+20"),
        Some(1.23456789012345E+20)
    );
    assert_eq!(parse_excel_number("1.00000000000000049"), Some(1.0));
    // Short input is unchanged, bad text is None.
    assert_eq!(parse_excel_number("0.1"), Some(0.1));
    assert_eq!(parse_excel_number("abc"), None);
    assert_eq!(parse_excel_number(""), None);
}
