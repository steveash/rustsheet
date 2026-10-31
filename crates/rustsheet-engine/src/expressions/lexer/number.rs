//! Number literals: [`Lexer::consume_number`].

use crate::number_format::parse_excel_number;

use super::{Lexer, Result};

impl Lexer {
    // Consumes a number in the current locale.
    // It only takes into account the decimal separator
    // Note that we do not parse the thousands separator
    // Let's say ',' is the thousands separator. Then 1,234 would be an error.
    // This is ok for most cases:
    // =IF(A1=1,234, TRUE, FALSE) will not work
    // If a user introduces a single number in the cell 1,234 we should be able to parse
    // and format the cell appropriately
    pub(super) fn consume_number(&mut self, first: char) -> Result<f64> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = first.to_string();
        // numbers before the decimal point
        while position < len {
            let x = self.chars[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        if position < len && self.chars[position].to_string() == self.locale.numbers.symbols.decimal
        {
            // numbers after the decimal point
            chars.push('.');
            position += 1;
            while position < len {
                let x = self.chars[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
        // The exponent marker: `E`, or `**` in typed text.
        let marker = match self.chars.get(position..) {
            Some(['e' | 'E', ..]) => 1,
            Some(['*', '*', ..]) if self.double_star_exponent => 2,
            _ => 0,
        };
        if marker > 0 && position + marker < len {
            // exponential side
            let x = self.chars[position + marker];
            if x == '-' || x == '+' || x.is_ascii_digit() {
                chars.push('e');
                chars.push(x);
                position += marker + 1;
                while position < len {
                    let x = self.chars[position];
                    if x.is_ascii_digit() {
                        chars.push(x);
                    } else {
                        break;
                    }
                    position += 1;
                }
            }
        }
        self.position = position;
        match parse_excel_number(&chars) {
            // Excel refuses a workbook whose formula holds a literal number text would not read
            // (`=1E-310`, `=0E309`: numbers.md "Number text of 1E+308 or more is not a number").
            None => Err(self.set_error(&format!("Failed to parse to double: {chars}"), position)),
            // A literal below the smallest normal reads as 0 (`=1E-308=0` is TRUE,
            // probes/subnormal-function-results A70): `parse_excel_number` flushes it.
            Some(v) => Ok(v),
        }
    }
}
