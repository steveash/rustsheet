use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The smallest number PERMUT refuses (Excel: 2^31 - 1; probes/stat-overflow-arguments E5, B8).
const PERMUT_NUMBER_LIMIT: f64 = crate::limits::COUNT_LIMIT_EXCLUSIVE;

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
        let n = self.cast_number_ref(n, cell)?.floor();
        let k = self.cast_number_ref(k, cell)?.floor();
        // Excel: a number of 2^31 - 1 or more is #NUM!, also for number_chosen 0
        // (`PERMUT(2147483646,1)` is 2147483646, `PERMUT(2147483647,1)` and `PERMUT(1E30,0)` are
        // #NUM!; probes/stat-overflow-arguments B8, D1-D9, E1-E8). It also keeps both in i64.
        if n < 0.0 || k < 0.0 || k > n || n >= PERMUT_NUMBER_LIMIT {
            return Err(num_error(cell));
        }
        let (n, k) = (n as i64, k as i64);
        // n * (n-1) * ... * (n-k+1), smallest factor first: (n-k)+1, (n-k)+2, ..., n. Counting
        // up from `n - k` (0 <= k <= n) never computes `n - k + 1` for k = 0. The factors are
        // distinct integers of at least 1, so after 171 of them the product is at least 171!,
        // past `f64::MAX`, and stays infinite: the loop stops there, at most 171
        // multiplications whatever the arguments.
        let low = n - k;
        let mut result = 1.0_f64;
        for i in 1..=k {
            if result.is_infinite() {
                break;
            }
            result *= (low + i) as f64;
        }
        // A product past f64::MAX is #NUM! (`PERMUT(171,171)`, `PERMUT(200,180)`; A13-A15, A18).
        if result.is_infinite() {
            return Err(num_error(cell));
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
        let n = self.cast_number_ref(n, cell)?.floor();
        // number_chosen truncates toward zero before its sign check, so one in (-1, 0) is 0 and
        // the result 1 (`PERMUTATIONA(10,-0.75)`, `PERMUTATIONA(0,-0.0001)`; gen-m9 0620505280d2ea6e
        // A280, probes/t2475-stat-subexpressions P!A28).
        let k = self.cast_number_ref(k, cell)?.trunc() as i64;
        // Excel: a number of about 1E+15 is #NUM!, even with number_chosen 0
        // (`PERMUTATIONA(1000000000000000,0)`, `PERMUTATIONA(FACT(18),2)`; gen-m9 775d053de498b8c7
        // A152, b9a32ce9a1fd0d71 A378), while `PERMUTATIONA(123456789.123,1)` is a number
        // (1b31092a0d79e5f1 A306). The cut is UNVERIFIED: PERMUT's 2^31 - 1 is taken (T-2474).
        if n < 0.0 || k < 0 || n >= PERMUT_NUMBER_LIMIT {
            return Err(num_error(cell));
        }
        Ok(n.powf(k as f64))
    }
}
