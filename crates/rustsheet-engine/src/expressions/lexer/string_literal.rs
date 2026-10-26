//! The longest string literal Excel accepts (T-2762).

use super::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::language::Language;
use crate::locale::Locale;

/// Most UTF-16 units a string literal's value may hold; a doubled quote counts once. Excel refuses
/// a longer one, typed and stored (T-2739, docs/excel-semantics/syntax.md "Formula length"; T-2762).
pub const MAX_STRING_LITERAL_UNITS: usize = 4_095;

/// The error for a string literal over [`MAX_STRING_LITERAL_UNITS`].
pub const STRING_LITERAL_TOO_LONG_MESSAGE: &str = "String literal is longer than 4095 characters";

/// Whether `formula` holds a string literal over [`MAX_STRING_LITERAL_UNITS`], which Excel refuses
/// when it loads a workbook (T-2762).
pub fn has_overlong_string_literal(
    formula: &str,
    locale: &'static Locale,
    language: &'static Language,
) -> bool {
    // A literal of 4,096 units needs at least 4,096 bytes of source (an emoji is 4 bytes for 2 units).
    if formula.len() <= MAX_STRING_LITERAL_UNITS {
        return false;
    }
    let mut lexer = Lexer::new(formula, LexerMode::A1, locale, language);
    loop {
        match lexer.next_token() {
            TokenType::EOF => return false,
            TokenType::Illegal(error) if error.message == STRING_LITERAL_TOO_LONG_MESSAGE => {
                return true
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_names_the_limit() {
        assert!(STRING_LITERAL_TOO_LONG_MESSAGE.contains(&MAX_STRING_LITERAL_UNITS.to_string()));
    }
}
