use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `PERMUT(number, number_chosen) = n! / (n-k)! = n*(n-1)*...*(n-k+1)`: both arguments lift
    /// over an array and a range (SPEC-0011; probes/stat-argument-modes H465-H468,
    /// probes/stat-later-argument-lift H57).
    pub(crate) fn fn_permut(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Permut, args, cell, &mut |model, values| {
            number_or_error(model.permut_of(values, cell))
        })
    }

    fn permut_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [n, k] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let n = self.cast_number_ref(n, cell)?.floor() as i64;
        let k = self.cast_number_ref(k, cell)?.floor() as i64;
        if n < 0 || k < 0 || k > n {
            return Err(num_error(cell));
        }
        // n * (n-1) * ... * (n-k+1), smallest factor first. The factors are distinct integers of
        // at least 1, so after 171 of them the product is at least 171!, past `f64::MAX`. An
        // infinite product stays infinite, so the loop stops there: at most 171 factors, the
        // kernel's own per-call cap (SPEC-0011 security rule; the lifting table charges each
        // element that weight), where `PERMUT(1E15,1E15)` looped 10^15 times. The result is
        // unchanged.
        let mut result = 1.0_f64;
        for factor in (n - k + 1)..=n {
            if result.is_infinite() {
                break;
            }
            result *= factor as f64;
        }
        Ok(result)
    }

    /// `PERMUTATIONA(number, number_chosen) = n^k` (permutations with repetition): both
    /// arguments lift over an array and a range (probes/stat-argument-modes H469-H472,
    /// probes/stat-later-argument-lift H59).
    pub(crate) fn fn_permutationa(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::Permutationa, args, cell, &mut |model, values| {
            let [n, k] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.permutationa_of(n, k, cell))
        })
    }

    fn permutationa_of(
        &self,
        n: &CalcResult,
        k: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let n = self.cast_number_ref(n, cell)?.floor() as i64;
        let k = self.cast_number_ref(k, cell)?.floor() as i64;
        if n < 0 || k < 0 {
            return Err(num_error(cell));
        }
        Ok((n as f64).powf(k as f64))
    }
}
