use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // DEVSQ(number1, [number2], ...)
    pub(crate) fn fn_devsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut sum = 0.0;
        let mut sumsq = 0.0;
        let mut count: u64 = 0;

        // tiny helper so we don't repeat ourselves
        #[inline]
        fn accumulate(sum: &mut f64, sumsq: &mut f64, count: &mut u64, value: f64) {
            *sum += value;
            *sumsq += value * value;
            *count += 1;
        }

        if let Err(error) = self.for_each_value_skip_empty(args, cell, |value| {
            accumulate(&mut sum, &mut sumsq, &mut count, value);
        }) {
            return error;
        }

        if count == 0 {
            // Excel: DEVSQ/AVEDEV of no numbers is #NUM!, not #DIV/0!
            // (docs/excel-semantics/coercion.md, poi EverythingTests!H392).
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "DEVSQ with no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mut result = sumsq - (sum * sum) / n;

        // Numerical noise can make result slightly negative when it should be 0
        if result < 0.0 && result > -1e-12 {
            result = 0.0;
        }

        CalcResult::Number(result)
    }
}
