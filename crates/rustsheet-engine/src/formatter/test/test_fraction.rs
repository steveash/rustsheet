#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::get_locale,
};

/// `value` under `format` in the en locale.
fn text(value: f64, format: &str) -> String {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    format_number(value, format, &ctx).text
}

// Expected texts are Excel's displayed text in eval/suites/gen-m10 (excel-graph, T-1611), cells
// named by case-id prefix; docs/excel-semantics/number-formats.md "Fraction formats".

#[test]
fn numerator_is_rounded_and_never_reduced() {
    assert_eq!(text(1.5, "# ?/4"), "1 2/4"); // 039accab F7
    assert_eq!(text(36.5, "# ?/10"), "36 5/10"); // 43980980 P2
    assert_eq!(text(0.125, "# ?/4"), " 1/4"); // 039accab F12: 0.5 rounds up
}

#[test]
fn numerator_rounds_at_fifteen_digits() {
    // 12.345 * 100 is 1234.4999999999998 (05c1c646 D11, 43980980 P16).
    assert_eq!(text(12.345, "?/100"), "1235/100");
    assert_eq!(text(12.345, "# ?/10"), "12 3/10");
}

#[test]
fn full_numerator_carries_into_the_integer() {
    assert_eq!(text(1.75, "# ?/2"), "2    "); // bfb1a4b0 S16
    assert_eq!(text(45000.75, "# ?/2"), "45001    "); // 8e073c5b I13
    assert_eq!(text(0.9999999999, "# ?/4"), "1    "); // 039accab F3
}

#[test]
fn zero_numerator_blanks_the_fraction() {
    assert_eq!(text(1E-5, "# ?/4"), "0    "); // 039accab F4
    assert_eq!(text(61.0, "# ??/16"), "61      "); // 05c1c646 A5
    assert_eq!(text(0.0, "# ?/2"), "0    "); // 43980980 N5
}

#[test]
fn zero_integer_is_left_out_under_sharp() {
    assert_eq!(text(0.5, "# ?/4"), " 2/4"); // 039accab F8
    assert_eq!(text(0.125, "# ?/10"), " 1/10"); // 43980980 P10
}

#[test]
fn minus_leads() {
    assert_eq!(text(-0.5, "# ?/4"), "- 2/4"); // 9a8b7d68 A1
    assert_eq!(text(-1.0 / 7.0, "# ??/16"), "-  2/16"); // 05c1c646 A14
    assert_eq!(text(-1.0 / 7.0, "# ?/2"), "-0    "); // bfb1a4b0 S4
}

#[test]
fn question_mark_pads_the_numerator() {
    assert_eq!(text(1.5, "# ??/16"), "1  8/16"); // 05c1c646 A3
    assert_eq!(text(2.0 / 3.0, "# ??/16"), " 11/16"); // 05c1c646 A19
}

#[test]
fn no_integer_part_puts_the_value_in_the_numerator() {
    assert_eq!(text(1.5, "?/100"), "150/100"); // 05c1c646 D3
    assert_eq!(text(61.0, "?/100"), "6100/100"); // 05c1c646 D5
    assert_eq!(text(-1.0 / 7.0, "?/100"), "-14/100"); // 05c1c646 D14
    assert_eq!(text(0.0, "?/100"), "0/100"); // bfbcd16c Y17
}

#[test]
fn large_integer_part() {
    // TEXT(1E+15,"# ?/4"), 039accab T!A6.
    assert_eq!(text(1E15, "# ?/4"), "1000000000000000    ");
}

#[test]
fn placeholder_denominator_is_the_last_convergent() {
    // 14/99 is closer to PI's fraction part, but 1/7 is the last convergent that fits.
    assert_eq!(text(std::f64::consts::PI, "# ??/??"), "3  1/7 "); // 4c6a49fe G13
    assert_eq!(text(std::f64::consts::PI, "# ???/???"), "3  16/113"); // 05c1c646 C18
                                                                      // 71/90 is closer, the binary fraction part is 0.78900000000430737...
    assert_eq!(text(123_456.789, "# ??/??"), "123456 15/19"); // 20512106 L16
    assert_eq!(text(12.345, "# ???/???"), "12  69/200"); // 05c1c646 C11
    assert_eq!(text(12.345, "?/?"), "37/3"); // 43980980 Z16
}

#[test]
fn placeholder_denominator_without_integer_is_improper() {
    assert_eq!(text(36.5, "?/?"), "73/2"); // 039accab E18
    assert_eq!(text(0.0, "?/?"), "0/1"); // 43980980 Z5
    assert_eq!(text(-1.5, "??/??"), "- 3/2 "); // 039accab U1
}

#[test]
fn placeholder_denominator_padding_and_blanks() {
    assert_eq!(text(1.5, "# ???/???"), "1   1/2  "); // 05c1c646 C3
    assert_eq!(text(999.9999, "# ?/?"), "1000    "); // 0e3d8eba F5
    assert_eq!(text(1.0 / 3.0, "0 ?/?"), "0 1/3"); // 0e3d8eba W4
    assert_eq!(text(-1.0 / 7.0, "0 ?/?"), "-0 1/7"); // 71ad5f93 G7
}

#[test]
fn sharp_numerator_drops_the_separator() {
    assert_eq!(text(0.5, "# #/#"), "1/2"); // 039accab K8
    assert_eq!(text(1_234_567_890.0, "# #/#"), "1234567890"); // 039accab K2
    assert_eq!(text(1.5, "# #/#"), "1 1/2");
}
