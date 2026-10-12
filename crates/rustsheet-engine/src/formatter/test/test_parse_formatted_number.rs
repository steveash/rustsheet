#![allow(clippy::unwrap_used)]

use crate::types::DateSystem;
use crate::{
    formatter::format::parse_formatted_number,
    locale::{get_default_locale, get_locale},
};

const PARSE_ERROR_MSG: &str = "Could not parse number";

fn parse(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_default_locale();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

fn parse_de(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("de").unwrap();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

fn parse_fr(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("fr").unwrap();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

#[test]
fn numbers() {
    // whole numbers
    assert_eq!(parse("400", &["$"]), Ok((400.0, None)));

    // decimal numbers
    assert_eq!(parse("4.456", &["$"]), Ok((4.456, None)));

    // scientific notation
    assert_eq!(
        parse("23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.123456789e-11", &["$"]),
        Ok((2.123456789e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E-9", &["$"]),
        Ok((4.5e-9, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e+2", &["$"]),
        Ok((2300.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E9", &["$"]),
        Ok((4.5e9, Some("0.00E+00".to_string())))
    );

    // negative numbers
    assert_eq!(parse("-400", &["$"]), Ok((-400.0, None)));
    assert_eq!(parse("-4.456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse("-23e-12", &["$"]),
        Ok((-2.3e-11, Some("0.00E+00".to_string())))
    );

    // trims space
    assert_eq!(parse(" 400  ", &["$"]), Ok((400.0, None)));
}

#[test]
fn percentage() {
    // whole numbers
    assert_eq!(parse("400%", &["$"]), Ok((4.0, Some("0%".to_string()))));
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Percentage in scientific notation will not be formatted as percentage
    assert_eq!(
        parse("23e-12%", &["$"]),
        Ok((23e-12 / 100.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3E4%", &["$"]),
        Ok((230.0, Some("0.00E+00".to_string())))
    );
}

#[test]
fn currency() {
    // whole numbers
    assert_eq!(
        parse("400$", &["$"]),
        Ok((400.0, Some("#,##0$".to_string())))
    );
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Currencies in scientific notation will not be formatted as currencies
    assert_eq!(
        parse("23e-12$", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3e-12$", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("€23e-12", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );

    // switch side of currencies
    assert_eq!(
        parse("$400", &["$"]),
        Ok((400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(
        parse("$4.456", &["$"]),
        Ok((4.456, Some("$#,##0.00_);[Red]($#,##0.00)".to_string())))
    );
    assert_eq!(
        parse("$23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("$2.3e-12", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e-12€", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
}

#[test]
fn negative_currencies() {
    assert_eq!(
        parse("-400$", &["$"]),
        Ok((-400.0, Some("#,##0$".to_string())))
    );
    assert_eq!(
        parse("-$400", &["$"]),
        Ok((-400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(
        parse("$-400", &["$"]),
        Ok((-400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
}

#[test]
fn errors() {
    // Strings are not numbers
    assert_eq!(parse("One", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // Not partial parsing
    assert_eq!(parse("23 Hello", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("Hello 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("2 3", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // No space between
    assert_eq!(parse("- 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

#[test]
fn errors_wrong_currency() {
    assert_eq!(parse("123€", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

// German uses ',' as decimal separator and '.' as group separator
#[test]
fn numbers_de() {
    assert_eq!(parse_de("3,15", &["$"]), Ok((3.15, None)));
    assert_eq!(parse_de("-4,456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse_de("1.234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_de("1.234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_de("1.234.567", &["$"]),
        Ok((1234567.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_de() {
    assert_eq!(
        parse_de("12,5%", &["$"]),
        Ok((0.125, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse_de("1.234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_de() {
    // en-US formatted numbers are not valid in the German locale
    assert_eq!(
        parse_de("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_de("1.23,4", &["$"]).is_err());
}

// French uses ',' as decimal separator and
// '\u{202f}' (narrow no-break space) as group separator
#[test]
fn numbers_fr() {
    assert_eq!(parse_fr("3,15", &["$"]), Ok((3.15, None)));
    // "1,000" is one, not one thousand
    assert_eq!(parse_fr("1,000", &["$"]), Ok((1.0, None)));
    assert_eq!(
        parse_fr("1\u{202f}234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_fr("-12\u{202f}345\u{202f}678", &["$"]),
        Ok((-12345678.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_fr() {
    assert_eq!(
        parse_fr("12,5%", &["$"]),
        Ok((0.125, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_fr() {
    // '.' is neither the decimal nor the group separator in French
    assert_eq!(parse_fr("1.000", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // en-US formatted numbers are not valid in the French locale
    assert_eq!(
        parse_fr("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_fr("1\u{202f}23,4", &["$"]).is_err());
}

#[test]
fn long_dates_en_us() {
    assert_eq!(
        parse("03/02/2024", &["$"]),
        Ok((45353.0, Some("mm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/02/2024", &["$"]),
        Ok((45353.0, Some("m/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/2/24", &["$"]),
        Ok((45353.0, Some("m/d/yy".to_string())))
    );

    assert_eq!(
        parse("02-10-1975", &["$"]),
        Ok((27435.0, Some("mm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-1975", &["$"]),
        Ok((27435.0, Some("m-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-75", &["$"]),
        Ok((27435.0, Some("m-dd-yy".to_string())))
    );
}

#[test]
fn month_name_first_separated_dates_are_not_dates() {
    // Excel en-US: probes/date-text-grammar A15, A16, A59-A62 are #VALUE!.
    for text in [
        "Mar/02/2024",
        "March/02/2024",
        "Feb-10-1975",
        "February-10-1975",
    ] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}

#[test]
fn month_name_dates_en_us() {
    // probes/date-text-grammar A1, A8, A10, A17.
    for text in ["July 1, 2026", "1 July 2026", "1-Jul-2026", "Jul 2026"] {
        assert!(
            matches!(parse(text, &["$"]), Ok((46204.0, Some(_)))),
            "{text}"
        );
    }
    // Number coercion does not trim date text (ironcalc-calc MROUND_TRUNC_INT.xlsx MROUND!C32).
    assert!(parse(" July 1, 2026 ", &["$"]).is_err());
}

#[test]
fn iso_dates() {
    assert_eq!(
        parse("2024/03/02", &["$"]),
        Ok((45353.0, Some("yyyy/mm/dd".to_string())))
    );
    assert_eq!(
        parse("2024/March/02", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
}

#[test]
fn long_dates_with_dots() {
    // en-US's short date does not use dots, so dotted text is not a date
    // (probes/date-text-grammar A39/A40: "1.7.2026" and "7.1.2026" are #VALUE!).
    assert!(parse("03.02.2024", &["$"]).is_err());
}

// Excel: probes/paren-negative-text, docs/excel-semantics/coercion.md.
#[test]
fn parenthesised_number_is_negative() {
    assert_eq!(parse("(1)", &["$"]), Ok((-1.0, None)));
    assert_eq!(
        parse("(1,000)", &["$"]),
        Ok((-1000.0, Some("#,##0".to_string())))
    );
    assert!(parse("-(1)", &["$"]).is_err());
    assert!(parse("(-1)", &["$"]).is_err());
    assert!(parse("()", &["$"]).is_err());
}

// Excel: probes/paren-negative-variants (excel-graph, T-0277).
#[test]
fn parenthesised_number_with_padding_currency_percent() {
    assert_eq!(parse("( 1 )", &["$"]), Ok((-1.0, None)));
    assert_eq!(parse("(1.5)", &["$"]), Ok((-1.5, None)));
    assert_eq!(
        parse("($1)", &["$"]),
        Ok((-1.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(parse("(1%)", &["$"]), Ok((-0.01, Some("0%".to_string()))));
    assert!(parse("(-1)", &["$"]).is_err());
    assert!(parse("(+1)", &["$"]).is_err());
    assert!(parse("((1))", &["$"]).is_err());
    assert!(parse("(1)x", &["$"]).is_err());
    // probes/paren-number-text-forms: a sign after the currency and a trailing currency.
    assert!(parse("($-1)", &["$"]).is_err());
    assert!(parse("($ -1)", &["$"]).is_err());
    assert!(parse("(1$)", &["$"]).is_err());
    // probes/paren-number-text-currency-variants: only a trailing euro sign is accepted.
    assert!(parse("($+1)", &["$", "€"]).is_err());
    assert!(parse("(1 $)", &["$", "€"]).is_err());
    assert!(parse("(€-1)", &["$", "€"]).is_err());
    assert_eq!(
        parse("(1€)", &["$", "€"]),
        Ok((-1.0, Some("#,##0€".to_string())))
    );
}

/// Formats Excel picks for typed text (probes/typed-currency-percent-input, excel-graph).
#[test]
fn typed_currency_and_percent_formats() {
    let usd = |n: f64, decimals: bool| {
        let digits = if decimals { "#,##0.00" } else { "#,##0" };
        Ok((n, Some(format!("${digits}_);[Red](${digits})"))))
    };
    assert_eq!(parse("$1.50", &["$"]), usd(1.5, true));
    assert_eq!(parse("($1.50)", &["$"]), usd(-1.5, true));
    assert_eq!(parse("-$1", &["$"]), usd(-1.0, false));
    assert_eq!(parse("-$1.50", &["$"]), usd(-1.5, true));
    assert_eq!(parse("$1,234.5", &["$"]), usd(1234.5, true));
    // `-$1e3` is negative like `$-1e3`.
    assert_eq!(
        parse("-$1e3", &["$"]),
        Ok((-1000.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("1.5%", &["$"]),
        Ok((0.015, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse("(1.5%)", &["$"]),
        Ok((-0.015, Some("0.00%".to_string())))
    );
    assert_eq!(parse("1,000%", &["$"]), Ok((10.0, Some("0%".to_string()))));
    assert_eq!(parse("-1%", &["$"]), Ok((-0.01, Some("0%".to_string()))));
}
