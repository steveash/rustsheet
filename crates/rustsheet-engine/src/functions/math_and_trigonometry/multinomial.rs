use crate::calc_result::Range;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// The largest `n` whose `n!` is finite in a double; MULTINOMIAL's arguments may sum to at most this.
const MAX_FACTORIAL_ARGUMENT: u64 = 170;

fn log_factorial(n: u64) -> f64 {
    (1..=n).map(|i| (i as f64).ln()).sum()
}

impl Model {
    // ── MULTINOMIAL ───────────────────────────────────────────────────────────

    /// `=MULTINOMIAL(number1, [number2], ...)`
    ///
    /// Returns `(n1 + n2 + ... + nk)! / (n1! * n2! * ... * nk!)`.
    /// All arguments must be non-negative integers (or numbers truncated to int).
    pub(crate) fn fn_multinomial(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<u64> = Vec::new();

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(v) => {
                    if v < 0.0 {
                        return CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "MULTINOMIAL requires non-negative values".to_string(),
                        );
                    }
                    values.push(v.trunc() as u64);
                }
                CalcResult::Range { left, right } => {
                    // A cell past the used area is empty, which counts as 0 here
                    // and changes neither the sum nor the product of factorials.
                    if let Err(error) =
                        self.for_each_cell_in_areas(&[Range { left, right }], cell, |_, value| {
                            match value {
                                CalcResult::Number(v) if v < 0.0 => Err(CalcResult::new_error(
                                    Error::NUM,
                                    cell,
                                    "MULTINOMIAL requires non-negative values".to_string(),
                                )),
                                CalcResult::Number(v) => {
                                    values.push(v.trunc() as u64);
                                    Ok(())
                                }
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {
                                    values.push(0);
                                    Ok(())
                                }
                                _ => Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "MULTINOMIAL requires numeric values".to_string(),
                                )),
                            }
                        })
                    {
                        return error;
                    }
                }
                CalcResult::Array(arr) => {
                    for row in arr {
                        for node in row {
                            match node {
                                ArrayNode::Number(v) => {
                                    if v < 0.0 {
                                        return CalcResult::new_error(
                                            Error::NUM,
                                            cell,
                                            "MULTINOMIAL requires non-negative values".to_string(),
                                        );
                                    }
                                    values.push(v.trunc() as u64);
                                }
                                ArrayNode::Empty => values.push(0),
                                ArrayNode::Error(e) => {
                                    return CalcResult::Error {
                                        error: e,
                                        origin: cell,
                                        message: String::new(),
                                    }
                                }
                                _ => {
                                    return CalcResult::new_error(
                                        Error::VALUE,
                                        cell,
                                        "MULTINOMIAL requires numeric values".to_string(),
                                    )
                                }
                            }
                        }
                    }
                }
                CalcResult::EmptyCell | CalcResult::EmptyArg => values.push(0),
                err @ CalcResult::Error { .. } => return err,
                _ => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "MULTINOMIAL requires numeric values".to_string(),
                    )
                }
            }
        }

        // Excel is #NUM! once the truncated arguments sum past 170, the largest n whose n! is
        // finite in a double (`MULTINOMIAL(170)` is a number, `(171)`, `(170,1)` and `(100,71)`
        // are #NUM!). Checking first also bounds the factorial loops below to 170 steps whatever
        // the argument (directive 5). Excel: see docs/excel-semantics/numbers.md#multinomial-of-a-huge-argument
        let sum = match values.iter().try_fold(0_u64, |acc, &v| acc.checked_add(v)) {
            Some(sum) if sum <= MAX_FACTORIAL_ARGUMENT => sum,
            _ => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "MULTINOMIAL: result overflow".to_string(),
                )
            }
        };
        // Compute (sum)! / (n1! * n2! * ... * nk!) with log-factorials for numerical stability.
        let log_result = log_factorial(sum) - values.iter().map(|&v| log_factorial(v)).sum::<f64>();
        let result = log_result.exp();
        CalcResult::Number(result.round())
    }
}
