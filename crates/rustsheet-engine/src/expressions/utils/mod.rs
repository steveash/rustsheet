use super::types::*;
use crate::constants::{LAST_COLUMN, LAST_ROW};

pub mod shift;
#[cfg(test)]
mod test;

/// Converts column letter identifier to number.
pub fn column_to_number(column: &str) -> Result<i32, String> {
    if column.is_empty() {
        return Err("Column identifier cannot be empty.".to_string());
    }

    if !column.is_ascii() {
        return Err("Column identifier must be ASCII.".to_string());
    }

    let mut column_number: i32 = 0;
    for character in column.chars() {
        if !character.is_ascii_uppercase() {
            return Err("Column identifier can use only A-Z characters".to_string());
        }
        // Checked: a long letter run (e.g. `ISERROR` after a sheet prefix) overflows i32, and a
        // wrapped value could land back in range and lex as a valid column (found by fuzzing,
        // T-0021).
        column_number = column_number
            .checked_mul(26)
            .and_then(|n| n.checked_add((character as i32) - 64))
            .ok_or_else(|| "Column is not valid.".to_string())?;
    }

    match is_valid_column_number(column_number) {
        true => Ok(column_number),
        false => Err("Column is not valid.".to_string()),
    }
}

/// If input number is outside valid range `None` is returned.
pub fn number_to_column(mut i: i32) -> Option<String> {
    if !is_valid_column_number(i) {
        return None;
    }
    let mut column = "".to_string();
    while i > 0 {
        let r = ((i - 1) % 26) as u8;
        column.insert(0, (65 + r) as char);
        i = (i - 1) / 26;
    }
    Some(column)
}

/// Checks if column number is in valid range.
pub fn is_valid_column_number(column: i32) -> bool {
    (1..=LAST_COLUMN).contains(&column)
}

pub fn is_valid_column(column: &str) -> bool {
    // last column XFD
    if column.len() > 3 {
        return false;
    }

    let column_number = column_to_number(column);

    match column_number {
        Ok(column_number) => is_valid_column_number(column_number),
        Err(_) => false,
    }
}

pub fn is_valid_row(row: i32) -> bool {
    (1..=LAST_ROW).contains(&row)
}

fn is_valid_row_str(row: &str) -> bool {
    match row.parse::<i32>() {
        Ok(r) => is_valid_row(r),
        Err(_r) => false,
    }
}

/// Whether Excel reads `name` as an R1C1 reference, or as one followed by more text. A sheet with
/// such a name is quoted in formula text and a defined name with it is refused (Graph rejects the
/// workbook). LET and LAMBDA names are not tested: `LET(R1C1D,5,R1C1D*2)` is 10.
/// docs/excel-semantics/references.md "R1C1-shaped sheet and defined names" (probes/r1c1-shaped-
/// sheet-names*, r1c1-name-*, r1c1-let-name-*; excel-graph, 2026-10-15, T-1652).
///
/// The test ignores case and is not a reference parse (brackets cannot occur in these names):
/// - `R`, `C` and `RC` on their own;
/// - `R<row>` followed by `C` and anything at all (`R1CD`, `R1C0D`, `R1C99999`, `R1C1.`);
/// - otherwise `R<row>`, `C<column>` or `RC<column>`, with the row in 1..=1048576 and the column
///   in 1..=16384 (`R0D`, `C16385D` and `RC16385D` are not), followed by nothing or by anything
///   but a dot (`R1_`, `C1R`, `RC1D`; not `R1.`, `R1.5`, `RC1.`).
///
/// `RCR`, `RCD` and `CR` are not R1C1 references: a letter after a bare `R`, `C` or `RC` ends it.
/// (`R1.C1` is quoted too, as the Lotus-style range `R1:C1`: see [`is_dotted_a1_range`].)
pub(crate) fn is_r1c1_shaped(name: &str) -> bool {
    let text = name.as_bytes();
    let is_letter = |position: usize, letter: u8| {
        text.get(position)
            .is_some_and(|c| c.eq_ignore_ascii_case(&letter))
    };
    let mut position = 1;
    if is_letter(0, b'R') {
        match leading_number(text, &mut position) {
            Some(row) if !is_valid_row(row) => return false,
            Some(_) if is_letter(position, b'C') => return true,
            Some(_) => {}
            None if is_letter(1, b'C') => {
                position = 2;
                match leading_number(text, &mut position) {
                    None => return text.len() == 2,
                    Some(column) if !is_valid_column_number(column) => return false,
                    Some(_) => {}
                }
            }
            None => return text.len() == 1,
        }
    } else if is_letter(0, b'C') {
        match leading_number(text, &mut position) {
            None => return text.len() == 1,
            Some(column) if !is_valid_column_number(column) => return false,
            Some(_) => {}
        }
    } else {
        return false;
    }
    text.get(position) != Some(&b'.')
}

/// Whether `name` is exactly `R<n>`, `C<n>` or `R<n>C<n>` (any case) with a number out of range
/// (`R0`, `C0`, `R0C1`, `R1048577`, `R1048577C1`). Graph loads a defined name spelled so, but every
/// use is `#NAME?`: docs/excel-semantics/references.md "R1C1-shaped sheet and defined names"
/// (probes/r1c1-name-r0 and siblings, r1c1-name-use-*; excel-graph, T-1657). `LET` names are
/// not affected (probes/r1c1-let-name-r0).
pub(crate) fn is_out_of_range_r1c1_name(name: &str) -> bool {
    let text = name.as_bytes();
    let mut position = 1;
    let (row, column) = if text.first().is_some_and(|c| c.eq_ignore_ascii_case(&b'R')) {
        let Some(row) = leading_number(text, &mut position) else {
            return false;
        };
        if position == text.len() {
            (Some(row), None)
        } else if text[position].eq_ignore_ascii_case(&b'C') {
            position += 1;
            (Some(row), leading_number(text, &mut position))
        } else {
            return false;
        }
    } else if text.first().is_some_and(|c| c.eq_ignore_ascii_case(&b'C')) {
        (None, leading_number(text, &mut position))
    } else {
        return false;
    };
    // A bare `R1C`, `C` or trailing text is not this shape.
    if position != text.len() || (row.is_none() && column.is_none()) {
        return false;
    }
    row.is_some_and(|r| !is_valid_row(r)) || column.is_some_and(|c| !is_valid_column_number(c))
}

/// Whether `name` starts with a Lotus-style range: an A1 cell, one or more dots and an A1 cell,
/// then nothing or a dot and anything (`A1.B2`, `A1..B2`, `R1.C1`, `A1.B2.C3`; not `A1.B2D`,
/// `A1.XFE1`, `A0.B1`). Excel reads `SUM(A1.B2)` as `SUM(A1:B2)`, and quotes a sheet with such a
/// name (docs/excel-semantics/references.md "R1C1-shaped sheet and defined names",
/// probes/r1c1-shaped-sheet-names-4, -5, lotus-dot-range; excel-graph, T-1652).
fn is_dotted_a1_range(name: &str) -> bool {
    let Some((first, rest)) = name.split_once('.') else {
        return false;
    };
    let rest = rest.trim_start_matches('.');
    let second = rest.split('.').next().unwrap_or(rest);
    parse_reference_a1(first).is_some() && parse_reference_a1(second).is_some()
}

/// Whether `name` is `R<digits>C<digits>` in uppercase, which the R1C1 lexer reads as a cell
/// reference when both numbers are in range. Formulas are stored in R1C1 form, so an identifier
/// spelled like that would come back as a reference after a reload.
fn is_r1c1_cell_text(name: &str) -> bool {
    let text = name.as_bytes();
    let mut position = 1;
    text.first() == Some(&b'R')
        && leading_number(text, &mut position).is_some()
        && text.get(position) == Some(&b'C')
        && {
            position += 1;
            leading_number(text, &mut position).is_some()
        }
        && position == text.len()
}

/// The decimal digits at `position`, which is left after them, or `None` if there are none. A
/// number past `i32::MAX` saturates there (it is out of any sheet's range either way).
fn leading_number(text: &[u8], position: &mut usize) -> Option<i32> {
    let start = *position;
    let mut number = 0_i32;
    while let Some(digit) = text.get(*position).filter(|c| c.is_ascii_digit()) {
        number = number
            .saturating_mul(10)
            .saturating_add(i32::from(digit - b'0'));
        *position += 1;
    }
    (*position > start).then_some(number)
}

pub fn parse_reference_a1(r: &str) -> Option<ParsedReference> {
    let chars = r.chars();
    let mut absolute_column = false;
    let mut absolute_row = false;
    let mut row = "".to_string();
    let mut column = "".to_string();
    let mut state = 1; // 1(column), 2(row)

    for ch in chars {
        match ch {
            'A'..='Z' if state == 1 => {
                column.push(ch);
            }
            '0'..='9' => {
                if state == 1 {
                    state = 2;
                }
                row.push(ch);
            }
            '$' => {
                if column == *"" {
                    absolute_column = true;
                } else if state == 1 {
                    absolute_row = true;
                    state = 2;
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }
    if !is_valid_column(&column) {
        return None;
    }
    if !is_valid_row_str(&row) {
        return None;
    }
    let row = match row.parse::<i32>() {
        Ok(r) => r,
        Err(_) => return None,
    };

    Some(ParsedReference {
        row,
        column: column_to_number(&column).ok()?,
        absolute_column,
        absolute_row,
    })
}

/// Returns true if `name` is a valid identifier in an A1-mode formula expression.
/// Unlike `is_valid_identifier`, it permits the R1C1-shaped names ("R", "C", "RC",
/// "R12C", "R1C1D") that are valid as LAMBDA parameters or LET variables but not as
/// defined names (`is_r1c1_shaped`). An uppercase `R<n>C<m>` is refused, though.
pub fn is_valid_a1_identifier(name: &str) -> bool {
    // Excel takes `LET(R1C1,5,R1C1*2)` (10, probes/r1c1-let-name-r1c1), but the engine's R1C1
    // storage would turn the name into a cell reference on reload (TODO(T-1658)).
    // An out-of-range `R0C1` stays a name in R1C1 form too (the lexer reads it so; T-1657).
    is_identifier_text(name) && (!is_r1c1_cell_text(name) || is_out_of_range_r1c1_name(name))
}

/// The rules every name follows: its characters, and not `TRUE`, `FALSE` or an A1 cell.
fn is_identifier_text(name: &str) -> bool {
    // https://support.microsoft.com/en-us/office/names-in-formulas-fc2935f9-115d-4bef-a370-3aa8bb4c91f1
    // https://github.com/MartinTrummer/excel-names/
    let upper = name.to_uppercase();
    let len = upper.chars().count();

    let mut chars = upper.chars();

    if len > 255 || len == 0 {
        return false;
    }
    let first = match chars.next() {
        Some(ch) => ch,
        None => return false,
    };

    // The first character of a name must be a letter, an underscore character (_), or a backslash (\).
    // A letter of any script: Excel for the web loads `LET(é,1,É+1)` (2) and `LET(Ä,2,ä*10)` (20)
    // (probes/let-name-non-ascii, T-1387). An ASCII-only test rejected `é` but passed `ß`, whose
    // uppercase is `SS`.
    if !(first.is_alphabetic() || first == '_' || first == '\\') {
        return false;
    }
    if upper == *"TRUE" || upper == *"FALSE" {
        return false;
    }
    if parse_reference_a1(name).is_some() {
        return false;
    }
    for ch in chars {
        if !(ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '\\') {
            return false;
        }
    }

    true
}

/// Whether `name` is a valid defined or table name: one that Excel does not read as an R1C1
/// reference ([`is_r1c1_shaped`]; that also refuses "R", "C", "r" and "c"). `R0C1` passes, as
/// Excel loads it (T-1657: its uses are `#NAME?` there).
pub fn is_valid_identifier(name: &str) -> bool {
    !is_r1c1_shaped(name) && is_identifier_text(name)
}

pub(crate) fn name_needs_quoting(name: &str) -> bool {
    // Excel only leaves a sheet name unquoted when it is made of letters, digits,
    // '_' and '.', and does not start with a digit or a '.'. Anything else
    // (space, '&', '=', '-', '+', '(', ')', '$', ',', ';', '{', '}', '\'', '!', ...)
    // must be quoted, otherwise `'P&L'!A1` would be read back as `P & L!A1`.
    let mut chars = name.chars();
    match chars.next() {
        // ADDRESS(…, "") emits a bare "!" prefix; an empty name is never quoted.
        None => return false,
        Some(c) if c.is_ascii_digit() || c == '.' => return true,
        Some(c) if !(c.is_alphabetic() || c == '_') => return true,
        Some(_) => {}
    }
    if chars.any(|c| !(c.is_alphanumeric() || c == '_' || c == '.')) {
        return true;
    }
    if parse_reference_a1(name).is_some() {
        // cell reference in A1 notation, e.g. B1048576 is quoted, B1048577 is not
        return true;
    }
    // An R1C1 reference or one followed by more text (RC, R1C1D, r1, C1R, R, C), or a
    // Lotus-style range (A1.B2, R1.C1).
    is_r1c1_shaped(name) || is_dotted_a1_range(name)
}

/// A defined name of this workbook without its scope prefix: `Name` for `Name`, `Sheet!Name` and
/// `[0]Sheet!Name` (the parser keeps the prefix in the node so the formula text round-trips as
/// typed). A linked workbook's `[1]!Name` is returned whole, so it never finds this workbook's
/// name of the same spelling.
pub(crate) fn bare_defined_name(name: &str) -> &str {
    if name.starts_with('[') && !name.starts_with("[0]") {
        return name;
    }
    name.rsplit_once('!').map_or(name, |(_, bare)| bare)
}

/// Quotes a string sheet name if it needs to
/// NOTE: Invalid characters in a sheet name \, /, *, \[, \], :, ?
pub fn quote_name(name: &str) -> String {
    if name_needs_quoting(name) {
        return format!("'{}'", name.replace('\'', "''"));
    };
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_name() {
        assert_eq!(quote_name("Sheet1"), "Sheet1");
        assert_eq!(quote_name("Sheet 1"), "'Sheet 1'");
        // escape and quote
        assert_eq!(quote_name("Sheet1'"), "'Sheet1'''");
        assert_eq!(quote_name("Data(2024)"), "'Data(2024)'");
        // any character outside [letters, digits, '_', '.'] needs quoting
        assert_eq!(quote_name("P&L"), "'P&L'");
        assert_eq!(quote_name("Q1=Q2"), "'Q1=Q2'");
        assert_eq!(quote_name("Sheet!"), "'Sheet!'");
        assert_eq!(quote_name("100%"), "'100%'");
        assert_eq!(quote_name(".hidden"), "'.hidden'");
        // unicode letters, '_' and '.' inside the name do not
        assert_eq!(quote_name("Été"), "Été");
        assert_eq!(quote_name("_draft"), "_draft");
        assert_eq!(quote_name("Sheet.1"), "Sheet.1");
        assert_eq!(quote_name("Data$2024"), "'Data$2024'");
        assert_eq!(quote_name("Data-2024"), "'Data-2024'");
        assert_eq!(quote_name("Data+2024"), "'Data+2024'");
        assert_eq!(quote_name("Data,2024"), "'Data,2024'");
        assert_eq!(quote_name("Data;2024"), "'Data;2024'");
        assert_eq!(quote_name("Data{2024}"), "'Data{2024}'");

        assert_eq!(quote_name("2024"), "'2024'");
        assert_eq!(quote_name("1Data"), "'1Data'");
        assert_eq!(quote_name("A1"), "'A1'");
        assert_eq!(quote_name("R1C1"), "'R1C1'");
        assert_eq!(quote_name("MySheet"), "MySheet");

        assert_eq!(quote_name("B1048576"), "'B1048576'");
        assert_eq!(quote_name("B1048577"), "B1048577");
    }
}
