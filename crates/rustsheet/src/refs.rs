//! Sheet, cell and range identities (SPEC-0001 "Identities and references") and pure A1
//! parsing/printing that does not need a workbook: sheet names are resolved through a
//! caller-supplied lookup.

use std::fmt;

use rustsheet_engine::expressions::utils::{column_to_number, number_to_column, quote_name};
use rustsheet_engine::{LAST_COLUMN, LAST_ROW};

use crate::RefError;

/// Inputs longer than this are rejected before any scanning (sheet names are at most 31 chars).
const MAX_REF_LEN: usize = 512;
/// Highest 0-based row index.
const MAX_ROW: u32 = LAST_ROW.unsigned_abs() - 1;
/// Highest 0-based column index.
const MAX_COL: u32 = LAST_COLUMN.unsigned_abs() - 1;

/// Persistent identifier of a worksheet (the engine's `sheet_id`), stable across
/// rename and reorder; not a positional index.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SheetId(u32);

impl SheetId {
    /// Wraps an engine sheet id.
    #[must_use]
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// The wrapped engine sheet id.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

/// A single cell: 0-based `row` and `col` on `sheet`.
///
/// `Display` prints the A1 address without a sheet (a `SheetId` carries no name); use
/// [`CellRef::to_a1_in`] for the sheet-qualified form.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CellRef {
    /// Worksheet.
    pub sheet: SheetId,
    /// 0-based row, at most 1,048,575.
    pub row: u32,
    /// 0-based column, at most 16,383.
    pub col: u32,
}

impl CellRef {
    /// Builds a bounds-checked cell reference.
    ///
    /// # Errors
    /// [`RefError::OutOfBounds`] when `row` or `col` exceeds the Excel limits.
    pub fn new(sheet: SheetId, row: u32, col: u32) -> Result<Self, RefError> {
        if row > MAX_ROW || col > MAX_COL {
            return Err(RefError::OutOfBounds);
        }
        Ok(Self { sheet, row, col })
    }

    /// A1 address qualified with `sheet_name`, quoted when Excel requires it.
    #[must_use]
    pub fn to_a1_in(&self, sheet_name: &str) -> String {
        format!("{}!{self}", quote_name(sheet_name))
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_cell(f, self.row, self.col)
    }
}

/// An inclusive, 0-based rectangle on `sheet`.
///
/// `Display` prints the A1 range without a sheet; use [`RangeRef::to_a1_in`] for the
/// sheet-qualified form. Whole columns print as `A:B`, whole rows as `1:3`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RangeRef {
    /// Worksheet.
    pub sheet: SheetId,
    /// First row.
    pub top: u32,
    /// First column.
    pub left: u32,
    /// Last row (inclusive).
    pub bottom: u32,
    /// Last column (inclusive).
    pub right: u32,
}

impl RangeRef {
    /// Builds a bounds-checked range.
    ///
    /// # Errors
    /// [`RefError::OutOfBounds`] when a corner exceeds the Excel limits; [`RefError::Invalid`]
    /// when `bottom < top` or `right < left`.
    pub fn new(
        sheet: SheetId,
        top: u32,
        left: u32,
        bottom: u32,
        right: u32,
    ) -> Result<Self, RefError> {
        if bottom > MAX_ROW || right > MAX_COL || top > MAX_ROW || left > MAX_COL {
            return Err(RefError::OutOfBounds);
        }
        if bottom < top || right < left {
            return Err(RefError::Invalid(format!(
                "inverted range ({top},{left}):({bottom},{right})"
            )));
        }
        Ok(Self {
            sheet,
            top,
            left,
            bottom,
            right,
        })
    }

    /// A1 range qualified with `sheet_name`, quoted when Excel requires it.
    #[must_use]
    pub fn to_a1_in(&self, sheet_name: &str) -> String {
        format!("{}!{self}", quote_name(sheet_name))
    }

    fn whole_rows(&self) -> bool {
        self.left == 0 && self.right == MAX_COL
    }

    fn whole_cols(&self) -> bool {
        self.top == 0 && self.bottom == MAX_ROW
    }
}

impl fmt::Display for RangeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A range spanning the whole sheet prints as corners, so `A:XFD`/`1:1048576` never
        // become ambiguous on re-parse.
        if self.whole_cols() && !self.whole_rows() {
            write_col(f, self.left)?;
            f.write_str(":")?;
            write_col(f, self.right)
        } else if self.whole_rows() && !self.whole_cols() {
            write!(
                f,
                "{}:{}",
                self.top.saturating_add(1),
                self.bottom.saturating_add(1)
            )
        } else if self.top == self.bottom && self.left == self.right {
            write_cell(f, self.top, self.left)
        } else {
            write_cell(f, self.top, self.left)?;
            f.write_str(":")?;
            write_cell(f, self.bottom, self.right)
        }
    }
}

/// What a method addresses: a cell, a range or an A1 string parsed per call.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Target<'a> {
    /// A single cell.
    Cell(CellRef),
    /// A rectangle.
    Range(RangeRef),
    /// Unparsed A1 text.
    A1(&'a str),
}

impl From<CellRef> for Target<'_> {
    fn from(cell: CellRef) -> Self {
        Self::Cell(cell)
    }
}

impl From<RangeRef> for Target<'_> {
    fn from(range: RangeRef) -> Self {
        Self::Range(range)
    }
}

impl<'a> From<&'a str> for Target<'a> {
    fn from(text: &'a str) -> Self {
        Self::A1(text)
    }
}

/// Case-insensitive sheet lookup over `(id, name)` pairs, for use as the `resolve_sheet`
/// argument of [`parse_cell`]/[`parse_range`]. `None` (unqualified reference) selects the
/// first sheet.
#[must_use]
pub fn resolve_sheet_by_name(sheets: &[(SheetId, &str)], name: Option<&str>) -> Option<SheetId> {
    match name {
        None => sheets.first().map(|(id, _)| *id),
        Some(wanted) => {
            let wanted = wanted.to_lowercase();
            sheets
                .iter()
                .find(|(_, n)| n.to_lowercase() == wanted)
                .map(|(id, _)| *id)
        }
    }
}

/// Parses a single-cell reference such as `A1`, `$A$1`, `Sheet1!A1` or `'It''s'!A1`.
///
/// `resolve_sheet` receives the unquoted sheet name, or `None` for an unqualified reference,
/// and returns its id (see [`resolve_sheet_by_name`]).
///
/// # Errors
/// [`RefError::UnknownSheet`] when the lookup fails, [`RefError::OutOfBounds`] beyond
/// `XFD1048576`, [`RefError::Unsupported`] for defined names and structured references, and
/// [`RefError::Invalid`] for anything else.
pub fn parse_cell(
    a1: &str,
    resolve_sheet: impl FnOnce(Option<&str>) -> Option<SheetId>,
) -> Result<CellRef, RefError> {
    let (sheet, body) = split_sheet(a1)?;
    let sheet = lookup(sheet, resolve_sheet)?;
    match parse_area(body)? {
        Area {
            first: Part::Cell { row, col },
            last: None,
        } => Ok(CellRef { sheet, row, col }),
        _ => Err(RefError::Invalid(a1.to_string())),
    }
}

/// Parses a range such as `A1`, `B2:A1`, `'My Sheet'!A1:B2`, `A:A` or `1:3`. Corners are
/// normalised so `top <= bottom` and `left <= right`; a single cell is a 1x1 range.
///
/// # Errors
/// As [`parse_cell`].
pub fn parse_range(
    a1: &str,
    resolve_sheet: impl FnOnce(Option<&str>) -> Option<SheetId>,
) -> Result<RangeRef, RefError> {
    let (sheet, body) = split_sheet(a1)?;
    let sheet = lookup(sheet, resolve_sheet)?;
    let (top, left, bottom, right) = match parse_area(body)? {
        Area {
            first: Part::Cell { row, col },
            last: None,
        } => (row, col, row, col),
        Area {
            first: Part::Cell { row: r1, col: c1 },
            last: Some(Part::Cell { row: r2, col: c2 }),
        } => (r1.min(r2), c1.min(c2), r1.max(r2), c1.max(c2)),
        Area {
            first: Part::Col(c1),
            last: Some(Part::Col(c2)),
        } => (0, c1.min(c2), MAX_ROW, c1.max(c2)),
        Area {
            first: Part::Row(r1),
            last: Some(Part::Row(r2)),
        } => (r1.min(r2), 0, r1.max(r2), MAX_COL),
        _ => return Err(RefError::Invalid(a1.to_string())),
    };
    Ok(RangeRef {
        sheet,
        top,
        left,
        bottom,
        right,
    })
}

fn lookup(
    sheet: Option<String>,
    resolve_sheet: impl FnOnce(Option<&str>) -> Option<SheetId>,
) -> Result<SheetId, RefError> {
    resolve_sheet(sheet.as_deref()).ok_or_else(|| RefError::UnknownSheet(sheet.unwrap_or_default()))
}

/// Splits an optional (possibly quoted) sheet prefix from the reference body.
fn split_sheet(text: &str) -> Result<(Option<String>, &str), RefError> {
    let invalid = || RefError::Invalid(text.chars().take(64).collect());
    if text.len() > MAX_REF_LEN {
        return Err(invalid());
    }
    let Some(quoted) = text.strip_prefix('\'') else {
        return match text.split_once('!') {
            None => Ok((None, text)),
            Some(("", _)) => Err(invalid()),
            Some((name, body)) => Ok((Some(name.to_string()), body)),
        };
    };
    let mut name = String::new();
    let mut chars = quoted.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '\'' {
            name.push(c);
        } else if chars.next_if(|&(_, n)| n == '\'').is_some() {
            name.push('\'');
        } else {
            let rest = quoted[i + 1..].strip_prefix('!').ok_or_else(invalid)?;
            return if name.is_empty() {
                Err(invalid())
            } else {
                Ok((Some(name), rest))
            };
        }
    }
    Err(invalid())
}

#[derive(Copy, Clone)]
enum Part {
    Cell { row: u32, col: u32 },
    Col(u32),
    Row(u32),
}

struct Area {
    first: Part,
    last: Option<Part>,
}

fn parse_area(body: &str) -> Result<Area, RefError> {
    let Some((a, b)) = body.split_once(':') else {
        // A bare letters-only token is a defined name (or a column without `:`), neither of
        // which v0 accepts.
        if is_name_like(body) {
            return Err(RefError::Unsupported);
        }
        return Ok(Area {
            first: parse_part(body, false)?,
            last: None,
        });
    };
    Ok(Area {
        first: parse_part(a, true)?,
        last: Some(parse_part(b, true)?),
    })
}

fn is_name_like(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_' || c == '\\')
        && token
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '\\'))
        && !is_cell_token(token)
}

/// `[$]LETTERS[$]DIGITS` with at most three letters.
fn is_cell_token(token: &str) -> bool {
    let t = token.strip_prefix('$').unwrap_or(token);
    let letters = t.chars().take_while(char::is_ascii_alphabetic).count();
    let rest = t[letters..].strip_prefix('$').unwrap_or(&t[letters..]);
    (1..=3).contains(&letters) && !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

// The engine's `parse_reference_a1` is not reused: it is uppercase-only, takes no ranges and
// returns `Option`, so it cannot classify OutOfBounds vs Unsupported vs Invalid.
fn parse_part(token: &str, in_range: bool) -> Result<Part, RefError> {
    let invalid = || RefError::Invalid(token.chars().take(64).collect());
    if token.contains('[') {
        return Err(RefError::Unsupported);
    }
    let t = token.strip_prefix('$').unwrap_or(token);
    let letters = t.chars().take_while(char::is_ascii_alphabetic).count();
    let (letter_part, rest) = t.split_at(letters);
    let digits = rest.strip_prefix('$').unwrap_or(rest);
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return if is_name_like(token) {
            Err(RefError::Unsupported)
        } else {
            Err(invalid())
        };
    }
    if letters > 3 && !digits.is_empty() {
        // e.g. `ABCD1` is a legal defined name
        return Err(RefError::Unsupported);
    }
    if letters > 3 {
        // Reject before `column_to_number`, whose arithmetic overflows on long letter runs.
        return Err(RefError::OutOfBounds);
    }
    let col = if letter_part.is_empty() {
        None
    } else {
        let n = column_to_number(&letter_part.to_ascii_uppercase())
            .map_err(|_| RefError::OutOfBounds)?;
        Some(u32::try_from(n - 1).map_err(|_| RefError::OutOfBounds)?)
    };
    let row = if digits.is_empty() {
        None
    } else {
        // Digits are validated above; only an absurdly long run can fail to parse.
        let n = digits.parse::<u64>().map_err(|_| RefError::OutOfBounds)?;
        if n == 0 || n > u64::from(MAX_ROW) + 1 {
            return Err(RefError::OutOfBounds);
        }
        Some(u32::try_from(n - 1).map_err(|_| RefError::OutOfBounds)?)
    };
    match (col, row) {
        (Some(col), Some(row)) => Ok(Part::Cell { row, col }),
        (Some(col), None) if in_range && !rest.contains('$') => Ok(Part::Col(col)),
        (None, Some(row)) if in_range && !token.starts_with("$$") && !rest.contains('$') => {
            Ok(Part::Row(row))
        }
        _ => Err(invalid()),
    }
}

fn write_col(f: &mut fmt::Formatter<'_>, col: u32) -> fmt::Result {
    let name = col
        .checked_add(1)
        .and_then(|c| i32::try_from(c).ok())
        .and_then(number_to_column)
        .ok_or(fmt::Error)?;
    f.write_str(&name)
}

fn write_cell(f: &mut fmt::Formatter<'_>, row: u32, col: u32) -> fmt::Result {
    write_col(f, col)?;
    write!(f, "{}", row.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const S1: SheetId = SheetId(7);
    const S2: SheetId = SheetId(9);

    fn sheets() -> Vec<(SheetId, &'static str)> {
        vec![(S1, "Sheet1"), (S2, "My Sheet"), (SheetId(11), "It's")]
    }

    fn cell(a1: &str) -> Result<CellRef, RefError> {
        parse_cell(a1, |n| resolve_sheet_by_name(&sheets(), n))
    }

    fn range(a1: &str) -> Result<RangeRef, RefError> {
        parse_range(a1, |n| resolve_sheet_by_name(&sheets(), n))
    }

    #[test]
    fn parses_cells() {
        let at = |sheet, row, col| CellRef { sheet, row, col };
        assert_eq!(cell("A1").unwrap(), at(S1, 0, 0));
        assert_eq!(cell("$B$3").unwrap(), at(S1, 2, 1));
        assert_eq!(cell("b3").unwrap(), at(S1, 2, 1));
        assert_eq!(cell("sheet1!A1").unwrap().sheet, S1);
        assert_eq!(cell("'my sheet'!C2").unwrap(), at(S2, 1, 2));
        assert_eq!(cell("'It''s'!A1").unwrap().sheet, SheetId(11));
        assert_eq!(cell("XFD1048576").unwrap(), at(S1, MAX_ROW, MAX_COL));
    }

    #[test]
    fn parses_ranges() {
        let r = range("'My Sheet'!A1:B2").unwrap();
        assert_eq!(
            (r.sheet, r.top, r.left, r.bottom, r.right),
            (S2, 0, 0, 1, 1)
        );
        assert_eq!(range("B2:A1").unwrap(), range("A1:B2").unwrap());
        assert_eq!(
            range("A1").unwrap(),
            RangeRef {
                sheet: S1,
                top: 0,
                left: 0,
                bottom: 0,
                right: 0
            }
        );
        let col = range("A:A").unwrap();
        assert_eq!(
            (col.top, col.bottom, col.left, col.right),
            (0, MAX_ROW, 0, 0)
        );
        let rows = range("1:3").unwrap();
        assert_eq!(
            (rows.top, rows.bottom, rows.left, rows.right),
            (0, 2, 0, MAX_COL)
        );
        assert_eq!(range("$A:$B").unwrap(), range("A:B").unwrap());
        assert_eq!(range("$1:$3").unwrap(), range("1:3").unwrap());
    }

    #[test]
    fn errors_are_classified() {
        assert!(matches!(cell("Nope!A1"), Err(RefError::UnknownSheet(n)) if n == "Nope"));
        assert!(matches!(cell("XFE1"), Err(RefError::OutOfBounds)));
        assert!(matches!(cell("A1048577"), Err(RefError::OutOfBounds)));
        assert!(matches!(cell("A0"), Err(RefError::OutOfBounds)));
        assert!(matches!(
            cell("A99999999999999999999999"),
            Err(RefError::OutOfBounds)
        ));
        for n in [8, 12, 500] {
            let l = "A".repeat(n);
            assert!(matches!(
                range(&format!("{l}:A")),
                Err(RefError::OutOfBounds)
            ));
            assert!(matches!(
                range(&format!("A:{l}")),
                Err(RefError::OutOfBounds)
            ));
        }
        assert!(matches!(cell("MyName"), Err(RefError::Unsupported)));
        assert!(matches!(cell("ABCD1"), Err(RefError::Unsupported)));
        assert!(matches!(cell("Table1[Col]"), Err(RefError::Unsupported)));
        assert!(matches!(range("Table1[#All]"), Err(RefError::Unsupported)));
        assert!(matches!(cell("A1:B2"), Err(RefError::Invalid(_))));
    }

    #[test]
    fn garbage_never_panics() {
        let long = "A".repeat(10_000);
        let long_digits = format!("A{}", "9".repeat(10_000));
        for text in [
            "",
            "!",
            "A1:",
            ":A1",
            "'",
            "''",
            "'!A1",
            "'a'",
            "'a'A1",
            "A1:B2:C3",
            "A:1",
            "$",
            "$$A$1",
            "ü1",
            "A1!",
            &long,
            &long_digits,
        ] {
            assert!(cell(text).is_err(), "{text:?}");
            assert!(range(text).is_err(), "{text:?}");
        }
    }

    #[test]
    fn bounds_constructors() {
        assert!(CellRef::new(S1, MAX_ROW, MAX_COL).is_ok());
        assert!(matches!(
            CellRef::new(S1, MAX_ROW + 1, 0),
            Err(RefError::OutOfBounds)
        ));
        assert!(matches!(
            RangeRef::new(S1, 0, 0, 0, MAX_COL + 1),
            Err(RefError::OutOfBounds)
        ));
        assert!(matches!(
            RangeRef::new(S1, 2, 0, 1, 0),
            Err(RefError::Invalid(_))
        ));
    }

    #[test]
    fn display_round_trips() {
        for a1 in [
            "A1",
            "XFD1048576",
            "A1:B2",
            "A:A",
            "B:D",
            "1:3",
            "A1:XFD1048576",
            "C3",
        ] {
            assert_eq!(range(a1).unwrap().to_string(), a1);
        }
        assert_eq!(range("A1:A1").unwrap().to_string(), "A1");
        for name in ["Sheet1", "My Sheet", "It's", "P&L", "A1", "2024"] {
            let id = SheetId(1);
            let r = RangeRef::new(id, 0, 0, 4, 3).unwrap();
            let text = r.to_a1_in(name);
            let parsed = parse_range(&text, |n| (n == Some(name)).then_some(id)).unwrap();
            assert_eq!(parsed, r, "{text}");
            let c = CellRef::new(id, 1, 1).unwrap();
            let text = c.to_a1_in(name);
            assert_eq!(
                parse_cell(&text, |n| (n == Some(name)).then_some(id)).unwrap(),
                c
            );
        }
        assert_eq!(
            CellRef::new(S1, 0, 0).unwrap().to_a1_in("My Sheet"),
            "'My Sheet'!A1"
        );
    }

    #[test]
    fn target_conversions() {
        let c = CellRef::new(S1, 0, 0).unwrap();
        assert!(matches!(Target::from(c), Target::Cell(_)));
        assert!(matches!(
            Target::from(range("A1:B2").unwrap()),
            Target::Range(_)
        ));
        assert!(matches!(Target::from("A1"), Target::A1("A1")));
    }
}
