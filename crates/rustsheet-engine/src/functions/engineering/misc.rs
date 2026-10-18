//! DELTA and GESTEP. Both arguments lift over an array and refuse a multi-cell range
//! (`ArgKind::LiftArrays`, SPEC-0011; probes/scalar-lift-shapes H10,
//! probes/eng-later-argument-lift H17, H18).

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
    number_format::to_precision,
};

impl Model {
    /// `DELTA(number1, [number2])`.
    pub(crate) fn fn_delta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Delta, args, cell, &mut |model, values| {
            model.compare_with_default(values, cell).map_or_else(
                |error| error,
                |(a, b)| CalcResult::Number(f64::from(a == b)),
            )
        })
    }

    /// `GESTEP(number, [step])`.
    pub(crate) fn fn_gestep(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gestep, args, cell, &mut |model, values| {
            model.compare_with_default(values, cell).map_or_else(
                |error| error,
                |(a, b)| CalcResult::Number(f64::from(a >= b)),
            )
        })
    }

    /// The two numbers DELTA and GESTEP compare, to 16 significant digits: the first, and the
    /// second or 0 when it is omitted. A boolean is `#VALUE!` (engineering.md "Engineering
    /// function argument modes", `DELTA(TRUE,1)`).
    fn compare_with_default(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        let (first, second) = match values {
            [first] => (first, None),
            [first, second] => (first, Some(second)),
            _ => return Err(CalcResult::new_args_number_error(cell)),
        };
        let first = self.cast_number_no_bools_ref(first, cell)?;
        let second = match second {
            Some(second) => self.cast_number_no_bools_ref(second, cell)?,
            None => 0.0,
        };
        Ok((to_precision(first, 16), to_precision(second, 16)))
    }
}
