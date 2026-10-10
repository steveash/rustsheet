//! DECIMAL: text in a radix to a number (docs/excel-semantics/engineering.md, "DECIMAL of
//! unreadable text").

use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The longest text DECIMAL accepts; longer is `#VALUE!` (probes/decimal-unreadable).
const DECIMAL_MAX_TEXT_LEN: usize = 255;

/// Reads `text` in `radix` the way Excel's DECIMAL does: leading spaces and tabs are skipped,
/// radix 16 also skips leading zeros and one `x`/`X`, empty text is 0, any other character
/// (sign, point, trailing space) is unreadable. The sum is a float, so it never overflows.
fn decimal_from_text(text: &str, radix: u32) -> Option<f64> {
    let mut rest = text.trim_start_matches([' ', '\t']);
    if radix == 16 {
        rest = rest.trim_start_matches('0');
        rest = rest.strip_prefix(['x', 'X']).unwrap_or(rest);
    }
    let mut value = 0.0_f64;
    for c in rest.chars() {
        value = value * f64::from(radix) + f64::from(c.to_digit(radix)?);
    }
    Some(value)
}

impl Model {
    pub(crate) fn fn_decimal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let radix = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        if !(2..=36).contains(&radix) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Radix must be between 2 and 36".to_string(),
            };
        }
        // Excel: see docs/excel-semantics/engineering.md#decimal-of-unreadable-text
        if text.chars().count() > DECIMAL_MAX_TEXT_LEN {
            return CalcResult::new_error(Error::VALUE, cell, "Text is too long".to_string());
        }
        match decimal_from_text(&text, radix as u32) {
            Some(n) => CalcResult::Number(n),
            None => CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: format!("'{}' is not a valid number in base {}", text, radix),
            },
        }
    }
}
