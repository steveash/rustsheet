//! T-2762: a string literal over 4,095 UTF-16 units is refused (T-2739 probes `t2739-typed-*`).
#![allow(clippy::unwrap_used)]

use crate::expressions::lexer::{
    has_overlong_string_literal, Lexer, LexerMode, STRING_LITERAL_TOO_LONG_MESSAGE,
};
use crate::expressions::token::TokenType;
use crate::language::get_language;
use crate::locale::get_locale;

fn first_token(formula: &str) -> TokenType {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    Lexer::new(formula, LexerMode::A1, locale, language).next_token()
}

fn is_refused(formula: &str) -> bool {
    matches!(first_token(formula), TokenType::Illegal(e) if e.message == STRING_LITERAL_TOO_LONG_MESSAGE)
}

fn overlong(formula: &str) -> bool {
    has_overlong_string_literal(
        formula,
        get_locale("en").unwrap(),
        get_language("en").unwrap(),
    )
}

#[test]
fn ascii_literal_limit() {
    assert!(!is_refused(&format!("\"{}\"", "a".repeat(4_095))));
    assert!(is_refused(&format!("\"{}\"", "a".repeat(4_096))));
}

#[test]
fn doubled_quote_counts_once() {
    assert!(!is_refused(&format!("\"{}\"\"\"", "a".repeat(4_094))));
    assert!(is_refused(&format!("\"{}\"\"\"", "a".repeat(4_095))));
}

#[test]
fn emoji_count_two_units() {
    assert!(!is_refused(&format!("\"{}\"", "\u{1F600}".repeat(2_047))));
    assert!(is_refused(&format!("\"{}\"", "\u{1F600}".repeat(2_048))));
}

#[test]
fn limit_is_per_literal() {
    let literal = format!("\"{}\"", "a".repeat(4_000));
    assert!(!overlong(&format!("{literal}&{literal}")));
    assert!(overlong(&format!("1&LEN(\"{}\")", "a".repeat(4_096))));
}
