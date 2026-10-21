//! DECIMAL: text in a radix to a number (docs/excel-semantics/engineering.md, "DECIMAL of
//! unreadable text").

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, value_error};
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, cast::MAX_SHORT_TEXT_UNITS, expressions::parser::Node, model::Model,
};

/// Reads `text` in `radix` the way Excel's DECIMAL does: leading spaces and tabs are skipped,
/// radix 16 also skips leading zeros and one `x`/`X`, empty text is 0, any other character
/// (sign, point, trailing space) is unreadable. The sum is a float with no 2^53 limit; one past
/// `f64::MAX` (199 `Z`s in radix 36, probes/decimal-unreadable) is `None` too, as Excel gives `#NUM!`.
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
    value.is_finite().then_some(value) // CalcResult::finite_number_or_num's rule
}

impl Model {
    /// `DECIMAL(text, radix)`: both arguments lift over arrays and ranges
    /// (probes/t2198-m5-lift-argument-kinds; docs/excel-semantics/arrays.md).
    pub(crate) fn fn_decimal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Decimal, args, cell, &mut |model, values| {
            let [text, radix] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let text = match Model::cast_string_ref(text, cell) {
                Ok(text) => text,
                Err(error) => return error,
            };
            let radix = match model.kernel_number(radix, cell) {
                Ok(f) => f.trunc() as i32,
                Err(error) => return error,
            };
            // Excel: see docs/excel-semantics/engineering.md#decimal-of-unreadable-text. The
            // length is in UTF-16 units and is checked before the radix (probes/decimal-unreadable).
            if text.encode_utf16().count() > MAX_SHORT_TEXT_UNITS {
                return value_error(cell);
            }
            if !(2..=36).contains(&radix) {
                return num_error(cell);
            }
            // The range check above makes the cast to u32 lossless.
            match decimal_from_text(&text, radix as u32) {
                Some(n) => CalcResult::Number(n),
                None => num_error(cell),
            }
        })
    }
}
