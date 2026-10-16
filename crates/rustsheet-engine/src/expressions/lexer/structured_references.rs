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
use crate::expressions::token::{TableReference, TableSpecifier};

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

    /// Reads a column name up to the unescaped `end_char` (consumed when it is `]`),
    /// unescaping the `'` sequences.
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
    /// lex like their `[#This Row]` forms (Excel only shows `@` in the UI).
    fn consume_this_row_shorthand(&mut self, table_name: &str) -> Result<TokenType> {
        self.position += 1; // '@'
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
            _ => Some(TableReference::ColumnReference(
                self.consume_escaped_until(']')?,
            )),
        };
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
        })
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
        let peek_char = self.peek_char();
        if peek_char == Some(']') {
            // This is just a reference to the full table
            self.expect(TokenType::RightBracket)?;
            return Ok(TokenType::Ident(table_name.to_string()));
        }
        if peek_char == Some('@') {
            return self.consume_this_row_shorthand(table_name);
        }
        if peek_char == Some('#') {
            // Expecting MyTable[#Totals]
            if let Some(specifier) = self.consume_table_specifier()? {
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier: Some(specifier),
                    table_reference: None,
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
            });
        }
        self.expect(TokenType::LeftBracket)?;
        let mut specifier = self.consume_table_specifier()?;
        if specifier.is_some() {
            if self.peek_token() == TokenType::Comma {
                self.advance_token();
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
                        self.expect(TokenType::LeftBracket)?;
                    }
                }
            }
            if self.peek_token() == TokenType::RightBracket {
                self.expect(TokenType::RightBracket)?;
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier,
                    table_reference: None,
                });
            }
        }

        // Now it's either:
        // [Column Name]
        // [Column Name]:[Column Name]
        self.position -= 1;
        let column_reference = self.consume_column_reference()?;
        let table_reference = if self.peek_char() == Some(':') {
            self.position += 1;
            let column_reference_right = self.consume_column_reference()?;
            self.expect(TokenType::RightBracket)?;
            Some(TableReference::RangeReference((
                column_reference,
                column_reference_right,
            )))
        } else {
            self.expect(TokenType::RightBracket)?;
            Some(TableReference::ColumnReference(column_reference))
        };
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
        })
    }
}
