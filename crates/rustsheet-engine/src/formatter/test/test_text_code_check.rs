#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::{format_number, format_text_value, FormatContext},
    locale::get_locale,
};

fn number(value: f64, format: &str) -> (String, bool) {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    let f = format_number(value, format, &ctx);
    (f.text, f.error.is_some())
}

fn rejects_text(text: &str, format: &str) -> bool {
    format_text_value(text, format).error.is_some()
}

// Values from probes/t2202-text-code-check-on-text (T-2202): a code rejected for a number is
// rejected for a text or boolean value too, and an accepted code returns the text unchanged.
#[test]
fn a_rejected_code_rejects_text_too() {
    for code in [
        "ab1",
        "*xb1",
        "a*?b1",
        "*?b1",
        "n",
        "0E0",
        "0b",
        "n;n;n;@",
        "0b;0b;0b;@",
    ] {
        assert!(number(2.5, code).1, "{code} on 2.5");
        assert!(rejects_text("a", code), "{code} on \"a\"");
        assert!(rejects_text("", code), "{code} on \"\"");
        assert!(rejects_text("TRUE", code), "{code} on TRUE");
    }
    for code in ["b1", "a*?", "*?", "b", "0.0", "@", "0;-0;0;@", "General"] {
        assert!(!number(2.5, code).1, "{code} on 2.5");
        assert_eq!(format_text_value("a", code).text, "a", "{code} on \"a\"");
    }
}

// A calendar prefix is valid at the start of a section, after a bracket too.
#[test]
fn a_calendar_prefix_only_starts_a_section() {
    assert_eq!(number(2.5, "b1").0, "");
    assert!(!number(2.5, "0;b1").1);
    assert!(!number(2.5, "[Red]b1").1);
    assert!(number(-2.5, "0;ab1").1);
}

// `[$USD]`: currency text of several characters prints as it is (gen-m10, T-1611).
#[test]
fn currency_text_prints_as_it_is() {
    assert_eq!(number(-1.5, "#,##0.00 [$USD]").0, "-1.50 USD");
    assert_eq!(
        number(1_234_567_890.0, "#,##0.00 [$USD]").0,
        "1,234,567,890.00 USD"
    );
    assert_eq!(number(1.5, "[$€]0.00").0, "€1.50");
    assert_eq!(format_text_value("TRUE", "#,##0.00 [$USD]").text, "TRUE");
    // A locale may follow the text, as it follows a one-character symbol; an unclosed bracket
    // is an error.
    assert_eq!(number(1.5, "0.0 [$USD-409]").0, "1.5 USD");
    assert!(number(1.5, "0.0 [$USD").1);
}

// Values from probes/text-format-digits-date-token (T-2377): a bare digit beside a date or time
// token rejects the whole code, an unused section included; AM/PM alone and quoted digits do not.
#[test]
fn a_bare_digit_beside_a_date_token_rejects_the_code() {
    for code in ["42abc", "4b", "b4", "4 d", "4:mm", "[h]4", "4g", "0;4b"] {
        assert!(number(3.25, code).1, "{code} on 3.25");
        assert!(rejects_text("x", code), "{code} on \"x\"");
    }
    for (code, text) in [
        ("4am/pm", "4AM"),
        ("\\4b", "443"),
        ("\"4\"b", "443"),
        ("4;b", "4"),
        ("4a", "4a"),
    ] {
        assert_eq!(number(3.25, code), (text.to_string(), false), "{code}");
    }
}

// `[Cyan]` is one of Excel's eight colors, so the format is accepted (gen-m10, T-1737).
#[test]
fn cyan_is_a_color() {
    assert_eq!(
        number(0.0, "[Green]#,##0;[Magenta](#,##0);[Cyan]0"),
        ("0".to_string(), false)
    );
    assert_eq!(number(5.0, "[Cyan]0"), ("5".to_string(), false));
}
