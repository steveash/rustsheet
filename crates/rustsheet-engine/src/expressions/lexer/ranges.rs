use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::{
    token::TokenType,
    utils::{column_to_number, is_valid_column, quote_name},
};

use super::Lexer;
use super::{ParsedRange, ParsedReference, Result};

/// The row part of a plain corner (`consume_corner_shape`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CornerRow {
    /// A column alone (`A`).
    None,
    /// A cell on the grid (`A1`).
    OnGrid,
    /// Column letters with a row of 0 or past the last row (`A0`, `A1048577`).
    OffGrid,
}

impl Lexer {
    /// `Sheet!LocalName` after a sheet prefix whose range did not parse: a defined name scoped to
    /// a sheet of this workbook, as the identifier `Sheet!LocalName` so the formula text
    /// round-trips (the parser resolves the scope; poi/56737.xlsx Uses!D2). `start` is where the
    /// range began; the position is left untouched when this is not a name.
    pub(super) fn consume_sheet_scoped_name(
        &mut self,
        sheet: &str,
        start: usize,
    ) -> Option<TokenType> {
        if sheet.starts_with('[') || sheet.contains(':') {
            return None;
        }
        let failed_at = self.position;
        self.position = start;
        let name = self.consume_identifier();
        if name_can_be_defined_name(&name) && !matches!(self.peek_char(), Some('(' | '[')) {
            return Some(TokenType::Ident(format!("{}!{name}", quote_name(sheet))));
        }
        self.position = failed_at;
        None
    }

    /// Consumes a reference in A1 style like:
    /// AS23, $AS23, AS$23, $AS$23, R12
    /// Or returns an error
    fn consume_reference_a1(&mut self) -> Result<ParsedReference> {
        let mut absolute_column = false;
        let mut absolute_row = false;
        let mut position = self.position;
        let len = self.len;
        if position < len && self.chars[position] == '$' {
            absolute_column = true;
            position += 1;
        }
        let mut column = "".to_string();
        while position < len {
            let x = self.chars[position].to_ascii_uppercase();
            match x {
                'A'..='Z' => column.push(x),
                _ => break,
            }
            position += 1;
        }
        if column.is_empty() {
            return Err(self.set_error("Failed to parse reference", position));
        }
        if position < len && self.chars[position] == '$' {
            absolute_row = true;
            position += 1;
        }
        let mut row = "".to_string();
        while position < len {
            let x = self.chars[position];
            match x {
                '0'..='9' => row.push(x),
                _ => break,
            }
            position += 1;
        }
        // Note that row numbers could start with 0 (`H01` is H1), but row 0 itself is off the grid.
        self.position = position;
        let column = column_to_number(&column).map_err(|error| self.set_error(&error, position))?;

        match row.parse::<i32>() {
            Ok(row) => {
                if row > LAST_ROW {
                    return Err(self.set_error("Row too large in reference", position));
                }
                if row < 1 {
                    return Err(self.set_error("Row zero in reference", position));
                }
                Ok(ParsedReference {
                    column,
                    row,
                    absolute_column,
                    absolute_row,
                })
            }
            Err(..) => Err(self.set_error("Failed to parse integer", position)),
        }
    }

    /// True when the text after the `:` at the current position starts with a sheet name followed
    /// by `!` (plain or quoted), e.g. the `Data!` in `B8:Data!E12`. Leaves the position unchanged.
    fn sheet_prefix_after_colon(&mut self) -> bool {
        let start = self.position;
        self.position += 1;
        let found = self.skip_sheet_prefix();
        // `consume_single_quote_string` moves the position to the end of input on failure.
        self.position = start;
        found
    }

    /// Skips a sheet prefix at the current position (quoted or plain name, then the `!`) and
    /// returns whether one was there.
    fn skip_sheet_prefix(&mut self) -> bool {
        let found = if self.peek_char() == Some('\'') {
            self.position += 1;
            self.consume_single_quote_string().is_ok() && self.peek_char() == Some('!')
        } else {
            !self.consume_identifier().is_empty() && self.peek_char() == Some('!')
        };
        if found {
            self.position += 1;
        }
        found
    }

    /// True when the text after the `:` at the current position is a sheet prefix followed by a
    /// second corner that is not a cell (`Data!A1:Data!B`, `Data!A1:Data!2`). Leaves the position
    /// unchanged.
    pub(super) fn second_prefixed_corner_is_not_cell(&mut self) -> bool {
        if self.peek_char() != Some(':') || !self.sheet_prefix_after_colon() {
            return false;
        }
        let start = self.position;
        self.position += 1;
        self.skip_sheet_prefix();
        let not_cell = self.consume_reference_a1().is_err();
        self.position = start;
        not_cell
    }

    /// Reads a corner made of a valid column and optional row digits (`A`, `B1`); no `$`. Returns
    /// whether it has a row, or `None` (position unspecified) when it is not such a corner.
    fn consume_plain_corner(&mut self) -> Option<bool> {
        match self.consume_corner_shape()? {
            CornerRow::None => Some(false),
            CornerRow::OnGrid => Some(true),
            CornerRow::OffGrid => None,
        }
    }

    /// Reads a valid column and row digits off the grid (`H0`, `A1048577`) at the current
    /// position and returns the text, or `None` with the position unchanged. After a sheet prefix
    /// such a cell is a name, as it is without one: `Data!H0` is `#NAME?`
    /// (probes/aggregate-range-function-num-grid, excel-graph, T-0296).
    pub(super) fn consume_off_grid_cell_as_name(&mut self) -> Option<String> {
        let start = self.position;
        if self.consume_corner_shape() == Some(CornerRow::OffGrid) {
            return Some(self.chars[start..self.position].iter().collect());
        }
        self.position = start;
        None
    }

    /// Reads a valid column and optional row digits (no `$`) not followed by more of a name or
    /// a call, and classifies the row; `None` (position unspecified) when it is not that shape.
    fn consume_corner_shape(&mut self) -> Option<CornerRow> {
        let from = self.position;
        while self.peek_char().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.position += 1;
        }
        let letters: String = self.chars[from..self.position].iter().collect();
        if !is_valid_column(&letters.to_ascii_uppercase()) {
            return None;
        }
        let digits_from = self.position;
        while self.peek_char().is_some_and(|c| c.is_ascii_digit()) {
            self.position += 1;
        }
        let digits: String = self.chars[digits_from..self.position].iter().collect();
        // `A1:INDEX(...)`, `B2.5` and the like are not corners.
        if self
            .peek_char()
            .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '$' | '(' | '.' | '_'))
        {
            return None;
        }
        if digits.is_empty() {
            return Some(CornerRow::None);
        }
        // A digit run too long for `i32` is past the last row too.
        let on_grid = digits
            .parse::<i32>()
            .is_ok_and(|r| (1..=LAST_ROW).contains(&r));
        Some(if on_grid {
            CornerRow::OnGrid
        } else {
            CornerRow::OffGrid
        })
    }

    /// Consumes the malformed corner pairs Excel reads as a name instead of a range, and returns
    /// their text: a second corner with its own sheet prefix next to a column or row corner
    /// (`Data!A:Data!B`, `Data!A1:Data!B`, `Data!A:Data!B1`), and a cell followed by a bare
    /// column (`Data!A1:B`). Excel gives `#NAME?` for them (references.md "Sheet-qualified column
    /// or row corners on both sides are not a range"); other parse failures are left alone.
    pub(super) fn consume_corner_pair_as_name(&mut self) -> Option<String> {
        let start = self.position;
        let first_has_row = self.consume_plain_corner();
        let mut result = None;
        if let (Some(first_has_row), Some(':')) = (first_has_row, self.peek_char()) {
            let has_prefix = self.sheet_prefix_after_colon();
            self.position += 1;
            if has_prefix {
                self.skip_sheet_prefix();
            }
            if let Some(second_has_row) = self.consume_plain_corner() {
                // Without a second prefix only `cell:column` is malformed; `A:B`, `A1:B2` and
                // `1:2` are ranges, and with a prefix two cells form a range as well.
                let malformed = if has_prefix {
                    !(first_has_row && second_has_row)
                } else {
                    first_has_row && !second_has_row
                };
                if malformed {
                    result = Some(self.chars[start..self.position].iter().collect());
                }
            }
        }
        if result.is_none() {
            self.position = start;
        }
        result
    }

    // Parsing a range is a parser on it's own right. Here is the grammar:
    //
    //    range       -> cell | cell ':' cell | row ':' row | column ':' column
    //    cell        -> column row
    //    column      -> '$' column_name | column_name
    //    row         -> '$' row_name | row_name
    //    column_name -> 'A'..'XFD'
    //    row_name    -> 1..1_048_576
    //
    /// Consumes a range of references in A1 style like:
    /// AS23:AS24, $AS23:AS24, AS$23:AS24, $AS$23:AS24, R12:R23, $R12:R23, R$12:R23, $R$12:R23
    pub(super) fn consume_range_a1(&mut self) -> Result<ParsedRange> {
        // first let's try to parse a cell
        let mut position = self.position;
        match self.consume_reference_a1() {
            Ok(cell) => {
                // `Data!B8:Data!E12`: when the second corner carries a sheet prefix the token ends at
                // the first corner. The colon is lexed as the range operator and the parser joins
                // the two references (references.md "Range corners may both carry the sheet prefix").
                if self.peek_char() == Some(':') && !self.sheet_prefix_after_colon() {
                    // It's a range
                    let colon = self.position;
                    self.position += 1;
                    if let Ok(cell2) = self.consume_reference_a1() {
                        Ok(ParsedRange {
                            left: cell,
                            right: Some(cell2),
                        })
                    } else {
                        // An off-grid second corner is a name joined by the range operator:
                        // `A1:B0` and `Data!A1:B1048577` are `#NAME?`, not references
                        // (probes/off-grid-corners, excel-graph, T-0296).
                        self.position = colon + 1;
                        if self.consume_off_grid_cell_as_name().is_some() {
                            self.position = colon;
                            return Ok(ParsedRange {
                                left: cell,
                                right: None,
                            });
                        }
                        Err(self.set_error("Expecting reference in range", self.position))
                    }
                } else {
                    // just a reference
                    Ok(ParsedRange {
                        left: cell,
                        right: None,
                    })
                }
            }
            Err(_) => {
                self.position = position;
                // It's either a row range or a column range (or not a range at all)
                let len = self.len;
                let mut absolute_left = false;
                if position < len && self.chars[position] == '$' {
                    absolute_left = true;
                    position += 1;
                }
                let mut column_left = "".to_string();
                let mut row_left = "".to_string();
                while position < len {
                    let x = self.chars[position].to_ascii_uppercase();
                    match x {
                        'A'..='Z' => column_left.push(x),
                        '0'..='9' => row_left.push(x),
                        _ => break,
                    }
                    position += 1;
                }
                if position >= len || self.chars[position] != ':' {
                    return Err(self.set_error("Expecting reference in range", self.position));
                }
                position += 1;
                let mut absolute_right = false;
                if position < len && self.chars[position] == '$' {
                    absolute_right = true;
                    position += 1;
                }
                let mut column_right = "".to_string();
                let mut row_right = "".to_string();
                while position < len {
                    let x = self.chars[position].to_ascii_uppercase();
                    match x {
                        'A'..='Z' => column_right.push(x),
                        '0'..='9' => row_right.push(x),
                        _ => break,
                    }
                    position += 1;
                }
                self.position = position;
                // At this point either the columns are the empty string or the rows are the empty string
                if !row_left.is_empty() {
                    // It is a row range 23:56
                    if row_right.is_empty() || !column_left.is_empty() || !column_right.is_empty() {
                        return Err(self.set_error("Error parsing Range", position));
                    }
                    // Note that row numbers can start with 0
                    let row_left = match row_left.parse::<i32>() {
                        Ok(n) => n,
                        Err(_) => {
                            return Err(
                                self.set_error(&format!("Failed parsing row {row_left}"), position)
                            )
                        }
                    };
                    let row_right = match row_right.parse::<i32>() {
                        Ok(n) => n,
                        Err(_) => {
                            return Err(self
                                .set_error(&format!("Failed parsing row {row_right}"), position))
                        }
                    };
                    if row_left > LAST_ROW {
                        return Err(self.set_error("Row too large in reference", position));
                    }
                    if row_right > LAST_ROW {
                        return Err(self.set_error("Row too large in reference", position));
                    }
                    return Ok(ParsedRange {
                        left: ParsedReference {
                            row: row_left,
                            absolute_row: absolute_left,
                            column: 1,
                            absolute_column: true,
                        },
                        right: Some(ParsedReference {
                            row: row_right,
                            absolute_row: absolute_right,
                            column: LAST_COLUMN,
                            absolute_column: true,
                        }),
                    });
                }
                // It is a column range
                if column_right.is_empty() || !row_right.is_empty() {
                    return Err(self.set_error("Error parsing Range", position));
                }
                let column_left = column_to_number(&column_left)
                    .map_err(|error| self.set_error(&error, position))?;
                let column_right = column_to_number(&column_right)
                    .map_err(|error| self.set_error(&error, position))?;
                Ok(ParsedRange {
                    left: ParsedReference {
                        row: 1,
                        absolute_row: true,
                        column: column_left,
                        absolute_column: absolute_left,
                    },
                    right: Some(ParsedReference {
                        row: LAST_ROW,
                        absolute_row: true,
                        column: column_right,
                        absolute_column: absolute_right,
                    }),
                })
            }
        }
    }

    /// Consumes a range of references in R1C1 style like:
    /// R12C3:R23C4, R[2]C[-2]:R[3]C[6], R3C[6]:R[-3]C4, R[-2]C:R[-2]C
    pub(super) fn consume_range_r1c1(&mut self) -> Result<ParsedRange> {
        // first let's try to parse a cell
        match self.consume_reference_r1c1() {
            Ok(cell) => {
                // Same as the A1 path: a sheet prefix on the second corner ends the token here.
                if self.peek_char() == Some(':') && !self.sheet_prefix_after_colon() {
                    // It's a range
                    self.position += 1;
                    if let Ok(cell2) = self.consume_reference_r1c1() {
                        Ok(ParsedRange {
                            left: cell,
                            right: Some(cell2),
                        })
                    } else {
                        Err(self.set_error("Expecting reference in range", self.position))
                    }
                } else {
                    // just a reference
                    Ok(ParsedRange {
                        left: cell,
                        right: None,
                    })
                }
            }
            Err(s) => Err(s),
        }
    }

    /// Consumes a reference in R1C1 style like:
    /// R12C3, R[2]C[-2], R3C[6], R[-3]C4, RC1, R[-2]C
    pub(super) fn consume_reference_r1c1(&mut self) -> Result<ParsedReference> {
        // R12C3, R[2]C[-2], R3C[6], R[-3]C4, RC1, R[-2]C
        let absolute_column;
        let absolute_row;
        let position = self.position;
        let row;
        let column;
        self.expect_char('R')?;
        match self.peek_char() {
            Some('[') => {
                absolute_row = false;
                self.expect_char('[')?;
                let c = match self.read_next_char() {
                    Some(s) => s,
                    None => {
                        return Err(self.set_error("Expected column number", position));
                    }
                };
                match self.consume_integer(c) {
                    Ok(v) => row = v,
                    Err(_) => {
                        return Err(self.set_error("Expected row number", position));
                    }
                }
                self.expect(TokenType::RightBracket)?;
            }
            Some(c) => {
                absolute_row = true;
                self.expect_char(c)?;
                match self.consume_integer(c) {
                    Ok(v) => row = v,
                    Err(_) => {
                        return Err(self.set_error("Expected row number", position));
                    }
                }
            }
            None => {
                return Err(self.set_error("Expected row number or '['", position));
            }
        }
        self.expect_char('C')?;
        match self.peek_char() {
            Some('[') => {
                self.expect_char('[')?;
                absolute_column = false;
                let c = match self.read_next_char() {
                    Some(s) => s,
                    None => {
                        return Err(self.set_error("Expected column number", position));
                    }
                };
                match self.consume_integer(c) {
                    Ok(v) => column = v,
                    Err(_) => {
                        return Err(self.set_error("Expected column number", position));
                    }
                }
                self.expect(TokenType::RightBracket)?;
            }
            Some(c) => {
                absolute_column = true;
                self.expect_char(c)?;
                match self.consume_integer(c) {
                    Ok(v) => column = v,
                    Err(_) => {
                        return Err(self.set_error("Expected column number", position));
                    }
                }
            }
            None => {
                return Err(self.set_error("Expected column number or '['", position));
            }
        }
        if let Some(c) = self.peek_char() {
            if c.is_alphanumeric() {
                return Err(self.set_error("Expected end of reference", position));
            }
        }

        Ok(ParsedReference {
            column,
            row,
            absolute_column,
            absolute_row,
        })
    }
}

/// Whether `text` after a sheet prefix may be a defined name rather than a cell: it starts like
/// a name and is not a cell in A1 (`A1`, `$B$2`) or R1C1 (`R2C3`, `RC`, `R`) style, which Excel
/// forbids as names.
fn name_can_be_defined_name(text: &str) -> bool {
    let Some(first) = text.chars().next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || first == '\\') {
        return false;
    }
    let upper = text.to_uppercase();
    let letters = upper.chars().take_while(char::is_ascii_alphabetic).count();
    let digits = upper[letters..].chars().all(|c| c.is_ascii_digit());
    let a1_cell = (1..=3).contains(&letters) && digits && letters < upper.len();
    // R1C1 shapes only: `R`, `C`, `RC`, `R<n>`, `C<n>`, `R<n>C<n>`.
    let r1c1 = r1c1_shape(&upper);
    !(a1_cell || r1c1)
}

/// Whether upper-case `text` is `R`, `C`, `RC`, `R<n>`, `C<n>` or `R<n>C<n>` (`<n>` digits).
fn r1c1_shape(text: &str) -> bool {
    let digits = |t: &str| {
        t.trim_start_matches(|c: char| c.is_ascii_digit())
            .to_string()
    };
    if let Some(rest) = text.strip_prefix('R') {
        let rest = digits(rest);
        return rest.is_empty() || digits(rest.strip_prefix('C').unwrap_or("x")).is_empty();
    }
    text.strip_prefix('C')
        .is_some_and(|rest| digits(rest).is_empty())
}
