//! How literals print in formula text: numbers in a locale, array-constant elements and the
//! argument separator. Split out of `stringify.rs` to keep it under its size ratchet (T-0499).

use crate::expressions::parser::ArrayNode;
use crate::language::Language;
use crate::locale::Locale;
use crate::number_format::to_excel_precision_str;

/// The separator of function arguments, and of union operands, in `locale`: `,`, or `;`
/// where the decimal separator is not `.`.
pub(crate) fn argument_separator(locale: &Locale) -> &'static str {
    if locale.numbers.symbols.decimal == "." {
        ","
    } else {
        ";"
    }
}

/// `number` as a formula prints it, with the decimal separator of `locale`.
pub(super) fn format_number_locale(number: f64, locale: &Locale) -> String {
    let s = to_excel_precision_str(number);
    let decimal = &locale.numbers.symbols.decimal;
    if decimal == "." {
        s
    } else {
        s.replace('.', decimal)
    }
}

/// The printed element `node` of an array constant.
pub(super) fn to_string_array_node(
    node: &ArrayNode,
    locale: &Locale,
    language: &Language,
) -> String {
    match node {
        ArrayNode::Boolean(true) => language.booleans.r#true.to_uppercase(),
        ArrayNode::Boolean(false) => language.booleans.r#false.to_uppercase(),
        ArrayNode::Number(number) => format_number_locale(*number, locale),
        ArrayNode::String(value) => format!("\"{value}\""),
        ArrayNode::Error(kind) => format!("{kind}"),
        ArrayNode::Empty => "0".to_string(),
        ArrayNode::Lambda(_) => "#CALC!".to_string(), // never parsed: what a cell shows
        ArrayNode::Nested(_) => "#VALUE!".to_string(), // never parsed: what a cell shows
    }
}
