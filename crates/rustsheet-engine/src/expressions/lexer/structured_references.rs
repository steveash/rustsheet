// Grammar:
// structured references -> table_name "[" arguments "]"
//  arguments -> table_reference | "["specifier"]"  "," table_reference
//  specifier > "#All"      |
//              "#This Row" |
//              "#Data"     |
//              "#Headers"  |
//              "#Totals"
// this_row_shorthand -> "[@]" | "[@" column_name "]" | "[@" "[" column_name "]" "]" | "[@" range_reference "]"
// table_reference -> column_reference | range_reference
// column reference -> column_name | "["column_name"]"
// range_reference -> column_reference":"column_reference

use crate::expressions::token::TokenType;
use crate::expressions::token::{TableReference, TableSpacing, TableSpecifier};

use super::Result;
use super::{Lexer, LexerError};

/// The two contiguous item-specifier pairs Excel accepts together (gen-m8b specials e804cf2f8af00680).
fn combine_specifiers(
    first: Option<TableSpecifier>,
    second: Option<TableSpecifier>,
) -> Option<TableSpecifier> {
    match (first?, second?) {
        (TableSpecifier::Headers, TableSpecifier::Data) => Some(TableSpecifier::HeadersData),
        (TableSpecifier::Data, TableSpecifier::Totals) => Some(TableSpecifier::DataTotals),
        _ => None,
    }
}

impl Lexer {
    /// A structured reference after `name` in R1C1 mode: one is stored as written (SPEC-0016),
    /// so storage (R1C1) reads it too. A name that a relative reference can start with before
    /// `[` (`R`, `C`, `RC`, `R<n>C`: `R[1]C1`, `RC[1]`, `R1C[1]`) is never a table here.
    /// `None` when there is none.
    pub(super) fn r1c1_structured_reference(&mut self, name: &str) -> Option<TokenType> {
        let upper = name.to_ascii_uppercase();
        let is_reference_part = match upper.strip_prefix('R') {
            Some(rest) => {
                let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
                rest.is_empty() || rest == "C"
            }
            None => upper == "C",
        };
        if self.peek_char() != Some('[')
            || !crate::expressions::utils::is_valid_a1_identifier(name)
            || is_reference_part
        {
            return None;
        }
        Some(self.consume_structured_reference(name).unwrap_or_else(|_| {
            TokenType::Illegal(self.set_error("Invalid structured reference", self.position))
        }))
    }

    /// Moves back to char index `position`, a position this lexer already passed: the parser
    /// re-reads a `[` operand as a structured reference once it knows the table (T-2979).
    pub(crate) fn rewind_to(&mut self, position: usize) {
        // Never forward: moving ahead would skip text the caller has not read.
        self.position = position.min(self.position);
    }

    /// The stored (R1C1) prefix of an unqualified structured reference, after its `[`: the table
    /// name up to `]`, consumed when a `[` follows it (`[T][@Col]`, `StructuredReference::stored_text`).
    /// `None`, position unchanged, otherwise.
    pub(crate) fn stored_table_prefix(&mut self) -> Option<String> {
        let start = self.position;
        let end = (start..self.len).find(|&i| self.chars[i] == ']')?;
        let name: String = self.chars[start..end].iter().collect();
        if self.chars.get(end + 1) != Some(&'[')
            || !crate::expressions::utils::is_valid_a1_identifier(&name)
        {
            return None;
        }
        self.position = end + 1;
        Some(name)
    }

    fn consume_table_specifier(&mut self) -> Result<Option<TableSpecifier>> {
        if self.peek_char() == Some('#') {
            // It's a specifier
            // The longest specifier name is "#this row]" (10 chars).
            let rest_of_formula: String = self.chars
                [self.position..self.len.min(self.position + 10)]
                .iter()
                .collect();
            // Item specifiers match ignoring case (gen-m8b names-case 5b5c0e20e82153bb).
            let rest_lower = rest_of_formula.to_lowercase();
            let (name, specifier) = [
                ("#this row]", TableSpecifier::ThisRow),
                ("#all]", TableSpecifier::All),
                ("#data]", TableSpecifier::Data),
                ("#headers]", TableSpecifier::Headers),
                ("#totals]", TableSpecifier::Totals),
            ]
            .into_iter()
            .find(|(name, _)| rest_lower.starts_with(name))
            .ok_or_else(|| LexerError {
                position: self.position,
                message: "Invalid structured reference".to_string(),
            })?;
            self.position += name.chars().count();
            Ok(Some(specifier))
        } else {
            Ok(None)
        }
    }

    fn consume_column_reference(&mut self) -> Result<String> {
        self.consume_whitespace();
        let end_char = if self.peek_char() == Some('[') {
            self.position += 1;
            ']'
        } else {
            ')'
        };

        self.consume_escaped_until(end_char)
    }

    /// Reads a column name up to the unescaped `end_char` (consumed when it is `]`, an error when
    /// that `]` is missing), unescaping the `'` sequences.
    fn consume_escaped_until(&mut self, end_char: char) -> Result<String> {
        let mut position = self.position;
        while position < self.len {
            let next_char = self.chars[position];
            if next_char != end_char {
                position += 1;
                if next_char == '\'' {
                    if position == self.len {
                        return Err(LexerError {
                            position: self.position,
                            message: "Invalid column name".to_string(),
                        });
                    }
                    // skip next char
                    position += 1;
                }
            } else {
                break;
            }
        }
        let chars: String = self.chars[self.position..position].iter().collect();
        if end_char == ']' {
            // An unterminated column name (`T[[Col`) is refused here: stepping past the missing
            // `]` would leave the position beyond the formula.
            if position >= self.len {
                return Err(LexerError {
                    position: self.position,
                    message: "Invalid column name".to_string(),
                });
            }
            position += 1;
        }
        self.position = position;
        Ok(chars
            .replace("'[", "[")
            .replace("']", "]")
            .replace("'#", "#")
            .replace("'@", "@")
            .replace("''", "'"))
    }

    /// The `@` shorthand for `#This Row`: `T[@]`, `T[@Col]`, `T[@[Col]]` and `T[@[A]:[B]]`
    /// lex like their `[#This Row]` forms (Excel only shows `@` in the UI), after the run `open`
    /// inside the outer `[`. A run after the `@` (`[@ Col]`) is not part of the column: Excel
    /// reads the column and prints the reference padded on both sides, `[ @Col ]` (syntax.md
    /// "Whitespace beside a missing argument", probes/t2800-this-row-whitespace F2:F3, T-3069),
    /// which reads back as the same reference, its runs kept as the padding.
    fn consume_this_row_shorthand(&mut self, table_name: &str, open: String) -> Result<TokenType> {
        self.position += 1; // '@'
        let after_at = self.whitespace_run();
        let mut close = String::new();
        let specifier = Some(TableSpecifier::ThisRow);
        let table_reference = match self.peek_char() {
            Some(']') => {
                self.position += 1;
                None
            }
            Some('[') => {
                let left = self.consume_column_reference()?;
                let reference = if self.peek_char() == Some(':') {
                    self.position += 1;
                    let right = self.consume_column_reference()?;
                    TableReference::RangeReference((left, right))
                } else {
                    TableReference::ColumnReference(left)
                };
                self.expect(TokenType::RightBracket)?;
                Some(reference)
            }
            _ => {
                let text = self.consume_escaped_until(']')?;
                let name = text.trim_end_matches(char::is_whitespace);
                close = text[name.len()..].to_string();
                Some(TableReference::ColumnReference(name.to_string()))
            }
        };
        // `[ @Col ]` brings its own runs before `@` and after the column; `[@ Col]` has only the
        // run after `@`, which then stands for both sides (Excel prints it `[ @Col ]`).
        let spacing = TableSpacing {
            open: if open.is_empty() {
                after_at.clone()
            } else {
                open
            },
            close: if close.is_empty() { after_at } else { close },
            comma_space: false,
        };
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
            spacing,
        })
    }

    /// The whitespace run at the current position, consumed.
    fn whitespace_run(&mut self) -> String {
        let start = self.position;
        self.consume_whitespace();
        self.chars[start..self.position].iter().collect()
    }

    /// The bare column of `T[ Col ]`, after the run `open`: the name up to the closing `]`, its
    /// trailing run kept as the closing padding. `None` (position unchanged for the caller to
    /// reset) when no run precedes that `]` or the name is empty.
    fn padded_bare_column(&mut self, table_name: &str, open: &str) -> Result<Option<TokenType>> {
        let text = self.consume_escaped_until(']')?;
        let name = text.trim_end_matches(char::is_whitespace);
        if name.is_empty() || name.len() == text.len() {
            return Ok(None);
        }
        Ok(Some(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference(name.to_string())),
            spacing: TableSpacing {
                open: open.to_string(),
                close: text[name.len()..].to_string(),
                comma_space: false,
            },
        }))
    }

    /// Possibilities:
    ///  1. `MyTable`[#Totals] or `MyTable`[#This Row]
    ///  2. `MyTable`[`MyColumn`]
    ///  3. `MyTable`[[My Column]]
    ///  4. `MyTable`[[#This Row], [My Column]]
    ///  5. `MyTable`[[#Totals], [`MyColumn`]]
    ///  6. `MyTable`[[#This Row], [Jan]:[Dec]]
    ///  7. `MyTable`[]
    ///
    /// Multiple specifiers are not supported yet:
    ///  1. `MyTable`[[#Data], [#Totals], [`MyColumn`]]
    ///
    /// In particular note that names of columns are escaped only when they are in the first argument
    /// We use '[' and ']'
    /// When there is only a specifier but not a reference the specifier is not in brackets
    ///
    /// Invalid:
    /// * `MyTable`[#Totals, [Jan]:[March]] => `MyTable`[[#Totals], [Jan]:[March]]
    //
    // NOTES:
    // * MyTable[[#Totals]] is translated into MyTable[#Totals]
    // * Excel shows '@' instead of '#This Row':
    //     MyTable[[#This Row], [Jan]:[Dec]] => MyTable[@[Jan]:[Dec]]
    //   But this is only a UI thing that we will ignore for now.
    pub(crate) fn consume_structured_reference(&mut self, table_name: &str) -> Result<TokenType> {
        self.expect(TokenType::LeftBracket)?;
        // A run after the outer `[` is kept before nested brackets (`T[ [Col] ]`) and, in text
        // the engine printed itself, around a bare column padded on both sides (`T[ Col ]`, the
        // form Excel prints for `T[ [Col] ]`). Typed or in a file, Excel refuses or loads as
        // `#REF!` every padded bare column (`T[ Col ]`, `T[ Col]`, `T[Col ]`; syntax.md,
        // probes/t2800-stored-ws-0..2, T-3069): the run is then read as part of the column
        // name, which no table has.
        let start = self.position;
        let mut spacing = TableSpacing {
            open: self.whitespace_run(),
            ..TableSpacing::default()
        };
        // A padded `#` start (`T[ #Data ]`) is unprobed and keeps its old reading.
        if !spacing.open.is_empty() && !matches!(self.peek_char(), Some('[' | '#' | '@')) {
            if self.padded_bare_columns {
                if let Some(token) = self.padded_bare_column(table_name, &spacing.open)? {
                    return Ok(token);
                }
            }
            // Back onto the run, so the bare column reads it as part of the name, up to `]`.
            self.position = start;
            spacing.open.clear();
        }
        let peek_char = self.peek_char();
        if peek_char == Some(']') {
            // This is just a reference to the full table
            self.expect(TokenType::RightBracket)?;
            return Ok(TokenType::Ident(table_name.to_string()));
        }
        if peek_char == Some('@') {
            return self.consume_this_row_shorthand(table_name, spacing.open);
        }
        if peek_char == Some('#') {
            // Expecting MyTable[#Totals]
            if let Some(specifier) = self.consume_table_specifier()? {
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier: Some(specifier),
                    table_reference: None,
                    spacing,
                });
            } else {
                return Err(LexerError {
                    position: self.position,
                    message: "Invalid structured reference".to_string(),
                });
            }
        } else if peek_char != Some('[') {
            // Expecting MyTable[MyColumn]
            self.position -= 1;
            let column_name = self.consume_column_reference()?;
            return Ok(TokenType::StructuredReference {
                table_name: table_name.to_string(),
                specifier: None,
                table_reference: Some(TableReference::ColumnReference(column_name)),
                spacing,
            });
        }
        self.expect(TokenType::LeftBracket)?;
        let mut specifier = self.consume_table_specifier()?;
        if specifier.is_some() {
            if self.peek_token() == TokenType::Comma {
                self.advance_token();
                spacing.comma_space = !self.whitespace_run().is_empty();
                self.expect(TokenType::LeftBracket)?;
                // A second item specifier: [[#Headers],[#Data],...] or [[#Data],[#Totals],...]
                if self.peek_char() == Some('#') {
                    let position = self.position;
                    let second = self.consume_table_specifier()?;
                    specifier = Some(combine_specifiers(specifier, second).ok_or(LexerError {
                        position,
                        message: "Invalid structured reference".to_string(),
                    })?);
                    if self.peek_token() == TokenType::Comma {
                        self.advance_token();
                        spacing.comma_space = !self.whitespace_run().is_empty();
                        self.expect(TokenType::LeftBracket)?;
                    }
                }
            }
            let before_close = self.position;
            spacing.close = self.whitespace_run();
            if self.peek_token() == TokenType::RightBracket {
                self.expect(TokenType::RightBracket)?;
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier,
                    table_reference: None,
                    spacing,
                });
            }
            self.position = before_close;
            spacing.close.clear();
        }

        // Now it's either:
        // [Column Name]
        // [Column Name]:[Column Name]
        self.position -= 1;
        let column_reference = self.consume_column_reference()?;
        let table_reference = if self.peek_char() == Some(':') {
            self.position += 1;
            let column_reference_right = self.consume_column_reference()?;
            TableReference::RangeReference((column_reference, column_reference_right))
        } else {
            TableReference::ColumnReference(column_reference)
        };
        spacing.close = self.whitespace_run();
        self.expect(TokenType::RightBracket)?;
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference: Some(table_reference),
            spacing,
        })
    }
}
