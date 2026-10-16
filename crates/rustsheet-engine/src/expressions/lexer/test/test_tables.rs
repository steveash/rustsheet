#![allow(clippy::unwrap_used)]

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::{OpSum, TableReference, TableSpecifier, TokenType::*},
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

// A defined name of a linked workbook, `[1]!Name` or `[1]Sheet!Name`, is one identifier
// (poi/ref-56737.xlsx Refs!G3, I3; T-1382); a cell after the same prefix stays a reference.
#[test]
fn external_book_names_are_identifiers() {
    let mut lx = new_lexer("[1]!NR_Global_B2");
    assert_eq!(lx.next_token(), Ident("[1]!NR_Global_B2".to_string()));
    assert_eq!(lx.next_token(), EOF);
    let mut lx = new_lexer("[1]Defines!NR_To_A1+1");
    assert_eq!(lx.next_token(), Ident("[1]Defines!NR_To_A1".to_string()));
    assert_eq!(lx.next_token(), Addition(OpSum::Add));
    let mut lx = new_lexer("[1]Uses!$A$1");
    assert!(matches!(lx.next_token(), Reference { .. }));
    let mut lx = new_lexer("[1]!HsGetValue(1)");
    assert_eq!(lx.next_token(), Ident("[1]!HsGetValue".to_string()));
    assert_eq!(lx.next_token(), LeftParenthesis);
    let mut lx = new_lexer("[1]!");
    assert_eq!(lx.next_token(), LeftBracket);
}

#[test]
fn table_this_row_at_shorthand_matches_long_form() {
    for (short, long) in [
        ("t[@Qty]", "t[[#This Row],[Qty]]"),
        ("t[@[Unit Price]]", "t[[#This Row],[Unit Price]]"),
        ("t[@[A]:[B]]", "t[[#This Row],[A]:[B]]"),
        ("t[@]", "t[#This Row]"),
        ("t[@a'@b]", "t[[#This Row],[a@b]]"),
    ] {
        let mut a = new_lexer(short);
        let mut b = new_lexer(long);
        let ta = a.next_token();
        assert_eq!(ta, b.next_token(), "{short}");
        assert!(matches!(ta, StructuredReference { .. }), "{short}");
        assert_eq!(a.next_token(), EOF, "{short}");
    }
}
