use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    pub(crate) fn fn_stdev_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut sum = 0.0;
        let mut sumsq = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(sum: &mut f64, sumsq: &mut f64, count: &mut u64, value: f64) {
            *sum += value;
            *sumsq += value * value;
            *count += 1;
        }

        if let Err(error) = self.for_each_value(args, cell, |value| {
            accumulate(&mut sum, &mut sumsq, &mut count, value);
        }) {
            return error;
        }

        if count == 0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "STDEV.P with no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mut var = (sumsq - (sum * sum) / n) / n;

        // clamp tiny negatives from FP noise
        if var < 0.0 && var > -1e-12 {
            var = 0.0;
        }

        CalcResult::Number(var.sqrt())
    }

    pub(crate) fn fn_stdev_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut sum = 0.0;
        let mut sumsq = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(sum: &mut f64, sumsq: &mut f64, count: &mut u64, value: f64) {
            *sum += value;
            *sumsq += value * value;
            *count += 1;
        }

        if let Err(error) = self.for_each_value(args, cell, |value| {
            accumulate(&mut sum, &mut sumsq, &mut count, value);
        }) {
            return error;
        }

        if count <= 1 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "STDEV.S requires at least two numeric values".to_string(),
            );
        }

        let n = count as f64;
        let mut var = (sumsq - (sum * sum) / n) / (n - 1.0);

        if var < 0.0 && var > -1e-12 {
            var = 0.0;
        }

        CalcResult::finite_number_or_num(var.sqrt(), cell)
    }

    pub(crate) fn fn_stdeva(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut sum = 0.0;
        let mut sumsq = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(sum: &mut f64, sumsq: &mut f64, count: &mut u64, value: f64) {
            *sum += value;
            *sumsq += value * value;
            *count += 1;
        }

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    accumulate(&mut sum, &mut sumsq, &mut count, value);
                }
                CalcResult::Range { left, right } => {
                    if let Err(error) =
                        self.for_each_coerced_number_in_range(left, right, cell, |value| {
                            accumulate(&mut sum, &mut sumsq, &mut count, value);
                        })
                    {
                        return error;
                    }
                }
                CalcResult::Array(array) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    accumulate(&mut sum, &mut sumsq, &mut count, value);
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
                                _ => {
                                    // ignore non-numeric for now
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // ignore non-numeric for now
                }
            }
        }

        if count <= 1 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "STDEVA requires at least two numeric values".to_string(),
            );
        }

        let n = count as f64;
        let mut var = (sumsq - (sum * sum) / n) / (n - 1.0);

        if var < 0.0 && var > -1e-12 {
            var = 0.0;
        }

        CalcResult::Number(var.sqrt())
    }

    pub(crate) fn fn_stdevpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut sum = 0.0;
        let mut sumsq = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(sum: &mut f64, sumsq: &mut f64, count: &mut u64, value: f64) {
            *sum += value;
            *sumsq += value * value;
            *count += 1;
        }

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    accumulate(&mut sum, &mut sumsq, &mut count, value);
                }
                CalcResult::Range { left, right } => {
                    if let Err(error) =
                        self.for_each_coerced_number_in_range(left, right, cell, |value| {
                            accumulate(&mut sum, &mut sumsq, &mut count, value);
                        })
                    {
                        return error;
                    }
                }
                CalcResult::Array(array) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    accumulate(&mut sum, &mut sumsq, &mut count, value);
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
                                _ => {
                                    // ignore non-numeric for now
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // ignore non-numeric for now
                }
            }
        }

        if count == 0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "STDEVPA with no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mut var = (sumsq - (sum * sum) / n) / n;

        if var < 0.0 && var > -1e-12 {
            var = 0.0;
        }

        CalcResult::Number(var.sqrt())
    }
}
