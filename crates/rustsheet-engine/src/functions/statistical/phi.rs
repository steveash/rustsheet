use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::number_or_error;
use crate::functions::statistical::special::standard_normal_pdf;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `PHI(x)`, the standard normal density: lifts over an array and a range (SPEC-0011;
    /// probes/stat-argument-modes H473-H475).
    pub(crate) fn fn_phi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Phi, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(
                model
                    .cast_number_no_bools_ref(x, cell)
                    .map(|x| flush_subnormal(standard_normal_pdf(x))),
            )
        })
    }
}
