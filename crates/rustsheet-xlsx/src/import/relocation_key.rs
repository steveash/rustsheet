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
//!
//! A shifted copy skips the lexing (T-3384). After keying a formula, [`KeyLexer`] keeps it as a
//! template: its text, cell and key, and where each relative coordinate of its references is
//! written. A later formula whose text is the template's with each of those coordinates moved by
//! the cell's offset, every other character equal, has the template's key: outside the
//! references its text is the template's, every relative coordinate keeps its offset from the
//! cell, every absolute one its value, and the corner comparisons of a range whose corners agree
//! on `$` per axis cannot change. Its tokens are the template's: only the letters and digits of
//! a coordinate change, and the lexer reads those by what surrounds them as well as by what they
//! are (after a digit or `.`, `E1` is part of a number; before `!`, `R:B` is a span of sheets),
//! so every reference must stand between characters that end a token whatever its letters and
//! digits (`BEFORE_REFERENCE`, `AFTER_REFERENCE`). A template is kept only when that reading is
//! certain: ASCII text, every reference so bounded and written `[prefix]$?LETTERS$?DIGITS` (or
//! two such corners joined by `:`, or two column or row corners `$?LETTERS:$?LETTERS`,
//! `$?DIGITS:$?DIGITS`) with the values and `$`s of its token, no range whose corners differ in
//! `$` on an axis, and a formula that parsed (the import drops the template of one that did not,
//! `KeyLexer::forget_template`). A moved coordinate must stay on the grid (offsets are taken in
//! `i64`: a cell's row comes from the file unchecked), and is compared as written canonically
//! (upper case, no leading zero); anything else is lexed as before. Any change to the A1 lexer's
//! reading of references must be checked against this reader:
//! `template_keys_match_a_fresh_lex_on_random_text` compares the two on random text.

use rustsheet_engine::expressions::lexer::{Lexer, LexerMode};
use rustsheet_engine::expressions::parser::Node;
use rustsheet_engine::expressions::token::TokenType;
use rustsheet_engine::expressions::types::{CellReferenceRC, ParsedReference};
use rustsheet_engine::expressions::utils::{column_letters, is_valid_row};
use rustsheet_engine::language::get_default_language;
use rustsheet_engine::locale::get_default_locale;

/// A lexer for stored A1 formulas, reused across the formulas of a sheet.
pub(crate) struct KeyLexer {
    lexer: Lexer,
    /// The last formula keyed whose shifted copies can take its key (module docs).
    template: Option<Template>,
    /// Keys taken from a template, for the tests of T-3384.
    #[cfg(test)]
    pub(crate) copies: usize,
    /// The identifiers last looked up as stored function names, with the answer: a fill-down
    /// asks about the same few names in every copy (T-3384).
    functions: Vec<(String, bool)>,
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
            functions: Vec::new(),
            template: None,
            #[cfg(test)]
            copies: 0,
        }
    }
}

/// A relative coordinate of a template's references: `text[start..end]` writes `value`.
struct Coordinate {
    start: usize,
    end: usize,
    value: i32,
    /// A row (digits), else a column (letters).
    row: bool,
}

/// A formula keyed, whose copies shifted by an offset have its key (module docs).
struct Template {
    /// The formula text, ASCII.
    text: String,
    row: i32,
    column: i32,
    key: String,
    /// Its relative coordinates, in text order.
    relative: Vec<Coordinate>,
}

impl Template {
    /// The template's text as written at `at`: each relative coordinate moved by the offset from
    /// its cell to `at` and written canonically, everything else as is; `None` when a coordinate
    /// moves off the grid. Offsets are taken in `i64`: a cell's row is read from the file
    /// unchecked, and could be anything in `i32`.
    fn shifted(&self, at: &CellReferenceRC) -> Option<String> {
        let rows = i64::from(at.row) - i64::from(self.row);
        let columns = i64::from(at.column) - i64::from(self.column);
        let mut copy = String::with_capacity(self.text.len() + 8);
        let mut read = 0;
        let (mut digits, mut letters) = ([0u8; 20], [0u8; 3]);
        for coordinate in &self.relative {
            copy.push_str(self.text.get(read..coordinate.start)?);
            let moved = i64::from(coordinate.value) + if coordinate.row { rows } else { columns };
            let moved = i32::try_from(moved).ok()?;
            let shown = if coordinate.row {
                is_valid_row(moved).then_some(())?;
                decimal(u64::try_from(moved).ok()?, &mut digits)
            } else {
                column_letters(moved, &mut letters)?
            };
            copy.extend(shown.iter().map(|&b| char::from(b)));
            read = coordinate.end;
        }
        copy.push_str(self.text.get(read..)?);
        Some(copy)
    }

    /// The key of `formula` at `at` when it is this template as written at `at`.
    fn key_of_copy(&self, formula: &str, at: &CellReferenceRC) -> Option<String> {
        (self.shifted(at)? == formula).then(|| self.key.clone())
    }
}

/// `n` in decimal, written at the end of `buffer`, as `to_string` writes it but with no
/// allocation or formatting machinery: those were a sixth of the key's cost (T-3384).
fn decimal(mut n: u64, buffer: &mut [u8; 20]) -> &[u8] {
    let mut start = buffer.len();
    loop {
        start -= 1;
        // A digit is below 10, so the cast keeps it.
        buffer[start] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 || start == 0 {
            break;
        }
    }
    &buffer[start..]
}

/// A corner read back from its text, ending at `end`: `$?LETTERS$?DIGITS`, upper case, where a
/// column range's corner has no digits and a row range's no letters.
struct Corner {
    /// Where the corner's text starts.
    start: usize,
    letters: (usize, usize),
    digits: (usize, usize),
    absolute_column: bool,
    absolute_row: bool,
}

impl Corner {
    /// (has letters, has digits).
    fn form(&self) -> (bool, bool) {
        (
            self.letters.0 < self.letters.1,
            self.digits.0 < self.digits.1,
        )
    }

    /// Whether it is a cell's corner, with both letters and digits.
    fn is_cell(&self) -> bool {
        self.form() == (true, true)
    }
}

/// What may come right before a template's reference: the lexer reads a reference's letters and
/// digits differently after a digit (`1E1` is a number), a letter, `.` or `_` (`1.E1`), `:` (a
/// sheet span `R:B!`), a quote or `$`, so a template allows only these. `!` ends a sheet prefix.
const BEFORE_REFERENCE: &str = "!(,;{=<>+-*/^&% ";

/// What may come right after a template's reference: not `!` (`R:B!C1` is a span of sheets),
/// `:` (a range joined later), `(` (a call), `.`, `#` (a spill), a letter or a digit.
const AFTER_REFERENCE: &str = "),;}=<>+-*/^&% ";

/// Whether the reference text `chars[start..end]` stands between characters after and before
/// which the lexer ends a token whatever the reference's letters and digits are (T-3384).
fn bounded(chars: &[char], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).and_then(|at| chars.get(at));
    let after = chars.get(end);
    before.is_none_or(|c| BEFORE_REFERENCE.contains(*c))
        && after.is_none_or(|c| AFTER_REFERENCE.contains(*c))
}

/// The corner whose text ends at `end` in `chars`, or `None` when it is not written as one.
fn corner_before(chars: &[char], end: usize) -> Option<Corner> {
    let mut at = end;
    while at > 0 && chars[at - 1].is_ascii_digit() {
        at -= 1;
    }
    let digits = (at, end);
    let absolute_row = at > 0 && chars[at - 1] == '$';
    if absolute_row {
        at -= 1;
    }
    let letters_end = at;
    while at > 0 && chars[at - 1].is_ascii_uppercase() {
        at -= 1;
    }
    let letters = (at, letters_end);
    let absolute_column = at > 0 && chars[at - 1] == '$';
    if absolute_column {
        at -= 1;
    }
    (digits.0 < digits.1 || letters.0 < letters.1).then_some(Corner {
        start: at,
        letters,
        digits,
        absolute_column,
        absolute_row,
    })
}

/// The relative coordinates of `corner` (read from `chars`), when it writes `reference`
/// exactly: its values (no leading zero) and its `$`s; else `None`. A corner without digits (a
/// column range's) or letters (a row range's) stands for a row or column its token reads as
/// absolute and fixed by that form (`J:J` is rows 1 to the last), so it adds no coordinate.
fn corner_coordinates(
    chars: &[char],
    corner: &Corner,
    reference: &ParsedReference,
    relative: &mut Vec<Coordinate>,
) -> Option<()> {
    let shown = |range: (usize, usize)| chars[range.0..range.1].iter().map(|&c| c as u8);
    let has_row = corner.digits.0 < corner.digits.1;
    let has_column = corner.letters.0 < corner.letters.1;
    let (mut digits, mut letters) = ([0u8; 20], [0u8; 3]);
    let row_exact = if has_row {
        let written = u64::try_from(reference.row).map(|row| decimal(row, &mut digits));
        corner.absolute_row == reference.absolute_row
            && written.is_ok_and(|written| shown(corner.digits).eq(written.iter().copied()))
    } else {
        reference.absolute_row
    };
    let column_exact = if has_column {
        corner.absolute_column == reference.absolute_column
            && column_letters(reference.column, &mut letters)
                .is_some_and(|written| shown(corner.letters).eq(written.iter().copied()))
    } else {
        reference.absolute_column
    };
    if !(row_exact && column_exact) {
        return None;
    }
    if has_column && !reference.absolute_column {
        relative.push(Coordinate {
            start: corner.letters.0,
            end: corner.letters.1,
            value: reference.column,
            row: false,
        });
    }
    if has_row && !reference.absolute_row {
        relative.push(Coordinate {
            start: corner.digits.0,
            end: corner.digits.1,
            value: reference.row,
            row: true,
        });
    }
    Some(())
}

/// The relative coordinates of a range `left:right` whose text ends at `end`, when both corners
/// are cells written exactly and agree on `$` per axis; else `None`.
fn range_coordinates(
    chars: &[char],
    end: usize,
    left: &ParsedReference,
    right: &ParsedReference,
    relative: &mut Vec<Coordinate>,
) -> Option<()> {
    if left.absolute_row != right.absolute_row || left.absolute_column != right.absolute_column {
        return None;
    }
    let second = corner_before(chars, end)?;
    let colon = second.start.checked_sub(1)?;
    if chars.get(colon) != Some(&':') {
        return None;
    }
    let first = corner_before(chars, colon)?;
    if first.form() != second.form() || !bounded(chars, first.start, end) {
        return None;
    }
    corner_coordinates(chars, &first, left, relative)?;
    corner_coordinates(chars, &second, right, relative)
}

/// One coordinate's part of the key: its offset from the cell when relative, its value when
/// absolute (the two are told apart by the `$`).
fn push_coordinate(key: &mut String, value: i32, at: i32, absolute: bool) {
    if absolute {
        key.push('$');
        push_integer(key, i64::from(value));
    } else {
        key.push('~');
        push_integer(key, i64::from(value) - i64::from(at));
    }
}

/// Writes `n` in decimal, as `to_string` would (`decimal`).
fn push_integer(key: &mut String, n: i64) {
    if n < 0 {
        key.push('-');
    }
    let mut digits = [0u8; 20];
    key.extend(
        decimal(n.unsigned_abs(), &mut digits)
            .iter()
            .map(|&b| char::from(b)),
    );
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
    /// The formula the template would take as its copy at `at`, for the tests of T-3384.
    #[cfg(test)]
    pub(crate) fn template_copy(&self, at: &CellReferenceRC) -> Option<String> {
        self.template.as_ref()?.shifted(at)
    }

    /// Forgets the template, for a formula that did not parse: its reading is not the one the
    /// module docs reason about, so its copies are keyed by lexing them (T-3384).
    pub(crate) fn forget_template(&mut self) {
        self.template = None;
    }

    /// How many identifiers [`KeyLexer::is_stored_function`] remembers.
    const FUNCTIONS: usize = 8;

    /// Whether `name` is a stored function name (`Functions::lookup_stored`), from the last few
    /// answers when it is one of them.
    fn is_stored_function(&mut self, name: &str) -> bool {
        if let Some((_, stored)) = self.functions.iter().find(|(known, _)| known == name) {
            return *stored;
        }
        let stored = get_default_language()
            .functions
            .lookup_stored(name)
            .is_some();
        if self.functions.len() == Self::FUNCTIONS {
            self.functions.remove(0);
        }
        self.functions.push((name.to_string(), stored));
        stored
    }

    /// The relocation key of `formula` at `at`, or `None` when the formula holds something whose
    /// parse is not a function of the key alone (module docs).
    pub(crate) fn key(&mut self, formula: &str, at: &CellReferenceRC) -> Option<String> {
        // The key marks references with U+0001 and writes sheet names raw: a formula holding the
        // marker (which XML 1.0 cannot carry anyway) could alias another's key, so it is not keyed.
        if formula.contains('\u{1}') {
            return None;
        }
        if let Some(key) = self
            .template
            .as_ref()
            .and_then(|template| template.key_of_copy(formula, at))
        {
            #[cfg(test)]
            {
                self.copies += 1;
            }
            return Some(key);
        }
        self.lexer.set_formula(formula);
        // The relative coordinates written, while the formula can still be a template.
        let mut relative = formula.is_ascii().then(Vec::new);
        let mut key = String::with_capacity(formula.len() + 16);
        let mut start = 0;
        let mut previous_is_reference = false;
        loop {
            let token = self.lexer.next_token();
            // The lexer's own chars, not a copy per formula (T-3384).
            let chars = self.lexer.chars();
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
                    self.template = relative.map(|relative| Template {
                        text: formula.to_string(),
                        row: at.row,
                        column: at.column,
                        key: key.clone(),
                        relative,
                    });
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
                    if let Some(coordinates) = relative.as_mut() {
                        let written = corner_before(chars, end)
                            .filter(Corner::is_cell)
                            .filter(|corner| bounded(chars, corner.start, end))
                            .and_then(|corner| corner_coordinates(chars, &corner, &r, coordinates));
                        if written.is_none() {
                            relative = None;
                        }
                    }
                    push_reference(&mut key, sheet.as_deref(), &r, at);
                    previous_is_reference = true;
                }
                TokenType::Range { sheet, left, right } => {
                    key.extend(&text[..leading]);
                    if let Some(coordinates) = relative.as_mut() {
                        let written = range_coordinates(chars, end, &left, &right, coordinates);
                        if written.is_none() {
                            relative = None;
                        }
                    }
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
                    let bare = name.rsplit('.').next().unwrap_or_default();
                    let binds =
                        bare.eq_ignore_ascii_case("LET") || bare.eq_ignore_ascii_case("LAMBDA");
                    // `text` borrows the lexer, which the peek moves: extend the key first.
                    key.extend(text);
                    let called = matches!(self.lexer.peek_token(), TokenType::LeftParenthesis);
                    if !called || binds || !self.is_stored_function(name) {
                        return None;
                    }
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
