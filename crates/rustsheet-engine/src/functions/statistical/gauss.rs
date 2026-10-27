use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::statistical::special::normal_cdf;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// phi(0) = 1/sqrt(2*pi), the density at 0.
const PHI_0: f64 = 0.398_942_280_401_432_7;
/// Below this |z| Excel returns z * phi(0) (`GAUSS(1E-300)` = 3.989422804014327E-301,
/// `GAUSS(1E-15)` = 3.989422804014328E-16). Above it Excel still shows the cancellation of
/// Phi(z) - 0.5 (`GAUSS(1E-10)` = 3.9894310077670525E-11, `GAUSS(4E-15)` = 1.5543122344752192E-15;
/// gen-m9 oracle), so the cut lies in (1E-15, 4E-15); 2E-15 is the evidence-bounded choice.
/// Excel: see docs/excel-semantics/statistics.md, "GAUSS of a tiny argument is x times phi(0) (T-2255)"
const TINY_Z: f64 = 2e-15;

impl Model {
    /// `GAUSS(z)`: lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H315-H317).
    pub(crate) fn fn_gauss(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gauss, args, cell, &mut |model, values| {
            let [z] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(z, cell).and_then(|z| {
                let result = if z.abs() < TINY_Z {
                    z * PHI_0
                } else {
                    normal_cdf(z, 0.0, 1.0) - 0.5
                };
                if result.is_finite() {
                    Ok(result)
                } else {
                    Err(num_error(cell))
                }
            }))
        })
    }
}
