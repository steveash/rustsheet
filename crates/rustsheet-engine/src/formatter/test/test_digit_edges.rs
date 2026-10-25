#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::{format_number, FormatContext},
    locale::get_locale,
};

fn text(value: f64, format: &str) -> String {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    format_number(value, format, &ctx).text
}

// T-1738: `?` pads an empty decimal with spaces and keeps the point on an integer (gen-m10,
// excel-graph 2026-10-16: 1000 is "1000.  ", 61 is "61.  ").
#[test]
fn question_decimals_keep_the_point_on_an_integer() {
    assert_eq!(text(1000.0, "?.??"), "1000.  ");
    assert_eq!(text(61.0, "?.??"), "61.  ");
}

// T-1738: `#%` of 0.0049 is "%": 0.49 rounds to 0 and `#` prints no zero (gen-m10).
#[test]
fn hash_percent_of_a_value_rounding_to_zero_is_the_sign_alone() {
    assert_eq!(text(0.0049, "#%"), "%");
}
