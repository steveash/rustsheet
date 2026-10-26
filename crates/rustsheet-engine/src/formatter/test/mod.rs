use crate::{
    formatter::format::{format_number, FormatContext},
    locale::get_locale,
};

/// `value` under `format` in the en locale (empty if the locale is missing, which fails the test).
fn text(value: f64, format: &str) -> String {
    get_locale("en").map_or_else(
        |_| String::new(),
        |locale| format_number(value, format, &FormatContext::new(locale)).text,
    )
}

mod test_condition;
mod test_dates;
mod test_de_examples;
mod test_digit_edges;
mod test_en_examples;
mod test_fr_examples;
mod test_fraction;
mod test_general;
mod test_locale_tag;
mod test_lunar_calendar;
mod test_parse_formatted_number;
mod test_scientific;
mod test_special_dollar;
mod test_text_code_check;
mod test_time;
