use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::statistical::special::normal_cdf;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `GAUSS(z)`: lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H315-H317).
    pub(crate) fn fn_gauss(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gauss, args, cell, &mut |model, values| {
            let [z] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(z, cell).and_then(|z| {
                let result = normal_cdf(z, 0.0, 1.0) - 0.5;
                if result.is_finite() {
                    Ok(result)
                } else {
                    Err(num_error(cell))
                }
            }))
        })
    }
}
