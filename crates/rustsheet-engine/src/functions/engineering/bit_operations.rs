//! BITAND, BITOR, BITXOR, BITLSHIFT and BITRSHIFT. Both arguments of each lift over an array and
//! over a multi-cell range (`ArgKind::Lift`, SPEC-0011; probes/eng-argument-modes E8:E12,
//! probes/scalar-lift-shapes H7, probes/eng-later-argument-lift H12:H15, H20; engineering.md
//! "Engineering function argument modes"): each kernel below is the scalar function of one
//! element, and builds its own errors with empty messages.

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

// 2^48-1
const MAX: f64 = 281474976710655.0;

/// A `#NUM!` element: its message is empty, so a failing element allocates nothing.
fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

/// The bitwise operation of BITAND, BITOR and BITXOR.
#[derive(Clone, Copy)]
enum BitOp {
    And,
    Or,
    Xor,
}

/// The direction of BITLSHIFT (`Left`) and BITRSHIFT (`Right`); a negative shift amount turns it.
#[derive(Clone, Copy)]
enum Shift {
    Left,
    Right,
}

impl Model {
    /// `BITAND(number1, number2)`.
    pub(crate) fn fn_bitand(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bitand, args, cell, &mut |model, values| {
            model.bit_op_of(values, BitOp::And, cell)
        })
    }

    /// `BITOR(number1, number2)`.
    pub(crate) fn fn_bitor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bitor, args, cell, &mut |model, values| {
            model.bit_op_of(values, BitOp::Or, cell)
        })
    }

    /// `BITXOR(number1, number2)`.
    pub(crate) fn fn_bitxor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bitxor, args, cell, &mut |model, values| {
            model.bit_op_of(values, BitOp::Xor, cell)
        })
    }

    /// `BITLSHIFT(number, shift_amount)`.
    pub(crate) fn fn_bitlshift(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bitlshift, args, cell, &mut |model, values| {
            model.bit_shift_of(values, Shift::Left, cell)
        })
    }

    /// `BITRSHIFT(number, shift_amount)`.
    pub(crate) fn fn_bitrshift(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bitrshift, args, cell, &mut |model, values| {
            model.bit_shift_of(values, Shift::Right, cell)
        })
    }

    /// BITAND, BITOR or BITXOR of one element: both numbers are read first, then checked
    /// together (integers, then not negative, then at most 2^48-1).
    fn bit_op_of(&self, values: &[CalcResult], op: BitOp, cell: CellReferenceIndex) -> CalcResult {
        let [number1, number2] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let number1 = match self.cast_number_ref(number1, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let number2 = match self.cast_number_ref(number2, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if number1.trunc() != number1 || number2.trunc() != number2 {
            return num_error(cell);
        }
        if number1 < 0.0 || number2 < 0.0 {
            return num_error(cell);
        }
        if number1 > MAX || number2 > MAX {
            return num_error(cell);
        }
        let number1 = number1.trunc() as i64;
        let number2 = number2.trunc() as i64;
        let result = match op {
            BitOp::And => number1 & number2,
            BitOp::Or => number1 | number2,
            BitOp::Xor => number1 ^ number2,
        };
        CalcResult::Number(result as f64)
    }

    /// BITLSHIFT or BITRSHIFT of one element: the number is checked before the shift amount.
    fn bit_shift_of(
        &self,
        values: &[CalcResult],
        direction: Shift,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let [number, shift] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let number = match self.cast_number_ref(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let shift = match self.cast_number_ref(shift, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if number.trunc() != number || !(0.0..=MAX).contains(&number) {
            return num_error(cell);
        }
        if shift.abs() > 53.0 {
            return num_error(cell);
        }
        let number = number.trunc() as i64;
        let shift = shift.trunc() as i64;
        let left = match direction {
            Shift::Left => shift > 0,
            Shift::Right => shift <= 0,
        };
        let result = if left {
            number << shift.abs()
        } else {
            number >> shift.abs()
        };
        let result = result as f64;
        if result.abs() > MAX {
            return num_error(cell);
        }
        CalcResult::Number(result)
    }
}
