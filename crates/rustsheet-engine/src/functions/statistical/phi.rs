use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::statistical::special::{normal_density_overflows, standard_normal_pdf};
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
                    // A boolean is a number: PHI(TRUE) = 0.24197072451914337
                    // (probes-numeric/t1049-phi A32, A33).
                    .cast_number_ref(x, cell)
                    .and_then(|x| {
                        // A square past the double range is #NUM!, not the limit 0
                        // (`PHI(1E+200)`; `normal_density_overflows`).
                        if normal_density_overflows(x) {
                            return Err(num_error(cell));
                        }
                        Ok(flush_subnormal(standard_normal_pdf(x)))
                    }),
            )
        })
    }
}
