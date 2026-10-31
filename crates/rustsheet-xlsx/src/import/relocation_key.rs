//! The relocation key of a stored A1 formula (T-3326): its text with every cell reference
//! replaced by what the parser makes of it at the formula's cell, so two copies of one formula
//! (a fill-down of `=COUNTIFS(A2:A9,B2)`) have the same key and the import can reuse the first
//! copy's R1C1 text without parsing the others.
//!
//! Soundness: equal keys (with the same sheet and array flag) give equal trees, so equal R1C1
//! text. The A1 parse reads the cell it is in at these places (T-3318's audit), each covered:
//! - `reference_operand` and `range_operand` subtract the cell from relative coordinates: a
//!   reference token is keyed by its offset from the cell for a relative coordinate and by its
//!   value for an absolute one, with its sheet prefix as written;
//! - a range orders its corners by their absolute values, which for corners of mixed `$` depends
//!   on the cell: the key records both comparisons (rows, columns) as well;
//! - identifiers read as coordinates (dotted ranges, a callee spelled like a cell, `A1(...)`) and
//!   structured references (this-row forms read the table extent at the cell): only identifiers
//!   that name a stored function and are called are kept, and a reference directly followed by
//!   `(` or any structured reference makes the formula uncached;
//! - LET and LAMBDA bind names from the raw text of an argument: a formula calling either is
//!   uncached (as is any other identifier, such as a defined name);
//! - parse errors carry the formula text and positions: the caller keeps only formulas that
//!   parsed (`Node::ParseErrorKind` is never cached);
//! - the context sheet: a cache serves one sheet.
//!
//! Everything else in the text (numbers, strings, errors, operators, whitespace, which can be
//! the intersection operator) is kept verbatim, so equal keys mean equal text outside the
//! references.

use rustsheet_engine::expressions::lexer::{Lexer, LexerMode};
use rustsheet_engine::expressions::parser::Node;
use rustsheet_engine::expressions::token::TokenType;
use rustsheet_engine::expressions::types::{CellReferenceRC, ParsedReference};
use rustsheet_engine::language::get_default_language;
use rustsheet_engine::locale::get_default_locale;

/// A lexer for stored A1 formulas, reused across the formulas of a sheet.
pub(crate) struct KeyLexer {
    lexer: Lexer,
}

impl Default for KeyLexer {
    fn default() -> Self {
        KeyLexer {
            lexer: Lexer::new(
                "",
                LexerMode::A1,
                get_default_locale(),
                get_default_language(),
            ),
        }
    }
}

/// One coordinate's part of the key: its offset from the cell when relative, its value when
/// absolute (the two are told apart by the `$`).
fn push_coordinate(key: &mut String, value: i32, at: i32, absolute: bool) {
    if absolute {
        key.push('$');
        key.push_str(&value.to_string());
    } else {
        key.push('~');
        key.push_str(&(value - at).to_string());
    }
}

fn push_reference(
    key: &mut String,
    sheet: Option<&str>,
    r: &ParsedReference,
    at: &CellReferenceRC,
) {
    key.push('\u{1}');
    if let Some(sheet) = sheet {
        key.push_str(sheet);
        key.push('!');
    }
    push_coordinate(key, r.row, at.row, r.absolute_row);
    key.push(',');
    push_coordinate(key, r.column, at.column, r.absolute_column);
    key.push('\u{1}');
}

impl KeyLexer {
    /// The relocation key of `formula` at `at`, or `None` when the formula holds something whose
    /// parse is not a function of the key alone (module docs).
    pub(crate) fn key(&mut self, formula: &str, at: &CellReferenceRC) -> Option<String> {
        // The key marks references with U+0001 and writes sheet names raw: a formula holding the
        // marker (which XML 1.0 cannot carry anyway) could alias another's key, so it is not keyed.
        if formula.contains('\u{1}') {
            return None;
        }
        let language = get_default_language();
        self.lexer.set_formula(formula);
        let chars: Vec<char> = formula.chars().collect();
        let mut key = String::with_capacity(formula.len() + 16);
        let mut start = 0;
        let mut previous_is_reference = false;
        loop {
            let token = self.lexer.next_token();
            let end = self.lexer.get_position_index().min(chars.len());
            let text = chars.get(start..end)?;
            let leading = text.iter().take_while(|c| c.is_whitespace()).count();
            if previous_is_reference && matches!(token, TokenType::LeftParenthesis) {
                return None;
            }
            previous_is_reference = false;
            match token {
                TokenType::EOF => {
                    key.extend(text);
                    return Some(key);
                }
                TokenType::Illegal(_) | TokenType::StructuredReference { .. } => return None,
                TokenType::Reference {
                    sheet,
                    row,
                    column,
                    absolute_column,
                    absolute_row,
                } => {
                    key.extend(&text[..leading]);
                    let r = ParsedReference {
                        row,
                        column,
                        absolute_row,
                        absolute_column,
                    };
                    push_reference(&mut key, sheet.as_deref(), &r, at);
                    previous_is_reference = true;
                }
                TokenType::Range { sheet, left, right } => {
                    key.extend(&text[..leading]);
                    push_reference(&mut key, sheet.as_deref(), &left, at);
                    push_reference(&mut key, None, &right, at);
                    // The corners are ordered by their absolute values.
                    key.push(if left.row <= right.row { '<' } else { '>' });
                    key.push(if left.column <= right.column {
                        '<'
                    } else {
                        '>'
                    });
                    previous_is_reference = true;
                }
                TokenType::Ident(ref name) => {
                    let upper = name.to_ascii_uppercase();
                    let bare = upper.rsplit('.').next().unwrap_or_default();
                    let called = matches!(self.lexer.peek_token(), TokenType::LeftParenthesis);
                    if !called
                        || bare == "LET"
                        || bare == "LAMBDA"
                        || language.functions.lookup_stored(name).is_none()
                    {
                        return None;
                    }
                    key.extend(text);
                }
                _ => key.extend(text),
            }
            start = end;
        }
    }
}

/// The R1C1 text of the formulas a sheet's load printed last, by parsed tree (T-3318).
///
/// A1 parsing stores a relative reference as its offset from the formula's cell, so the trees of
/// a formula copied along a row or column are equal, and the R1C1 text is a function of the tree
/// and of whether the cell holds an array formula alone (`add_implicit_intersection` and
/// `to_rc_format` read nothing else), up to the sign of a zero, which the derived `PartialEq`
/// ignores: trees holding a `-0` are not cached. A hit skips both, which were about 40% of loading
/// corpus-bench/52450's 36,640 copies of one formula. Only the last few trees are kept: copies
/// come in runs, and comparing a tree that differs stops at the first different node.
#[derive(Default)]
pub(super) struct RcTextCache {
    /// (tree as parsed, array formula, R1C1 text), most recent last.
    entries: Vec<(Node, bool, String)>,
    /// (relocation key, array formula, R1C1 text), most recent last: a hit skips the parse too
    /// (T-3326, `relocation_key`).
    keyed: Vec<(String, bool, String)>,
    pub(super) lexer: KeyLexer,
}

impl RcTextCache {
    /// How many trees, and how many keys, are kept.
    const CAPACITY: usize = 8;

    pub(super) fn get(&self, tree: &Node, is_array_formula: bool) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|(t, array, _)| *array == is_array_formula && t == tree)
            .map(|(.., text)| text.as_str())
    }

    pub(super) fn insert(&mut self, tree: Node, is_array_formula: bool, text: String) {
        if self.entries.len() == Self::CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push((tree, is_array_formula, text));
    }

    pub(super) fn get_keyed(&self, key: &str, is_array_formula: bool) -> Option<&str> {
        self.keyed
            .iter()
            .rev()
            .find(|(k, array, _)| *array == is_array_formula && k == key)
            .map(|(.., text)| text.as_str())
    }

    pub(super) fn insert_keyed(&mut self, key: String, is_array_formula: bool, text: String) {
        if self.keyed.len() == Self::CAPACITY {
            self.keyed.remove(0);
        }
        self.keyed.push((key, is_array_formula, text));
    }
}
