#![allow(clippy::unwrap_used)]

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::{TableReference, TableSpecifier, TokenType::*},
};
use crate::language::get_language;
use crate::locale::get_locale;

fn new_lexer(formula: &str) -> Lexer {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    Lexer::new(formula, LexerMode::A1, locale, language)
}

#[test]
fn table_this_row() {
    let mut lx = new_lexer("tbInfo[[#This Row], [Jan]:[Dec]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::ThisRow),
            table_reference: Some(TableReference::RangeReference((
                "Jan".to_string(),
                "Dec".to_string()
            )))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier() {
    let mut lx = new_lexer("tbInfo[December]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("December".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier_white_spaces() {
    let mut lx = new_lexer("tbInfo[[First Month]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("First Month".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_totals_no_reference() {
    let mut lx = new_lexer("tbInfo[#Totals]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::Totals),
            table_reference: None
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

// The unquoted `[book]Sheet!` prefix is a reference prefix (T-1381); a `[` that no `book]Sheet!`
// follows stays a bracket, so structured references and a LAMBDA's `[x]` lex as before.
#[test]
fn external_book_prefix_and_plain_brackets() {
    let mut lx = new_lexer("[other.xlsx]Sheet1!B5");
    assert_eq!(
        lx.next_token(),
        Reference {
            sheet: Some("[other.xlsx]Sheet1".to_string()),
            row: 5,
            column: 2,
            absolute_row: false,
            absolute_column: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
    let mut lx = new_lexer("[x],1");
    assert_eq!(lx.next_token(), LeftBracket);
    assert_eq!(lx.next_token(), Ident("x".to_string()));
    let mut lx = new_lexer("tbInfo[Jan]");
    assert!(matches!(lx.next_token(), StructuredReference { .. }));
}
