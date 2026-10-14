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

/// The largest `n` whose `n!` is finite in a double (170! is about 7.3e306).
const MAX_FACTORIAL_ARGUMENT: u64 = 170;

fn log_factorial(n: u64) -> f64 {
    (1..=n).map(|i| (i as f64).ln()).sum()
}

fn negative_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::NUM,
        cell,
        "MULTINOMIAL requires non-negative values".to_string(),
    )
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
        // The type check over every argument wins over a negative number whatever the order
        // (gen-m6/2c1a6e93a1974f48 Calc!A35, docs/excel-semantics/coercion.md), so a negative
        // only sets this flag and `#NUM!` is returned after the last argument.
        let negative_seen = std::cell::Cell::new(false);

        for arg in args {
            let evaluated = self.evaluate_node_in_context(arg, cell);
            // Only a text or boolean range cell is evidenced to beat the negative; typed
            // arguments, errors and array elements after one keep the immediate `#NUM!`.
            if negative_seen.get() && !matches!(evaluated, CalcResult::Range { .. }) {
                return negative_error(cell);
            }
            match evaluated {
                CalcResult::Number(v) => {
                    if v < 0.0 {
                        negative_seen.set(true);
                    } else {
                        values.push(v.trunc() as u64);
                    }
                }
                CalcResult::Range { left, right } => {
                    // A cell past the used area is empty, which counts as 0 here
                    // and changes neither the sum nor the product of factorials.
                    if let Err(error) = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |_, value| {
                            match value {
                                // An error cell after a negative keeps the immediate `#NUM!`.
                                CalcResult::Error { .. } if negative_seen.get() => {
                                    Err(negative_error(cell))
                                }
                                error @ CalcResult::Error { .. } => Err(error),
                                CalcResult::Number(v) if v < 0.0 => {
                                    negative_seen.set(true);
                                    Ok(())
                                }
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
                        },
                    ) {
                        return error;
                    }
                }
                CalcResult::Array(arr) => {
                    for row in arr {
                        for node in row {
                            if negative_seen.get() {
                                return negative_error(cell);
                            }
                            match node {
                                ArrayNode::Number(v) => {
                                    if v < 0.0 {
                                        negative_seen.set(true);
                                    } else {
                                        values.push(v.trunc() as u64);
                                    }
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

        if negative_seen.get() {
            return negative_error(cell);
        }
        // Excel: `#NUM!` once the arguments add up to more than 170, where `sum!` overflows a
        // double: `MULTINOMIAL(171)`, `(170,1)`, `(86,86)` and `(1E+15)` are `#NUM!`, while
        // `(170)`, `(169,1)` and `(85,85)` have values (probes/multinomial-large-arguments,
        // T-1197). Checking first also bounds the log-factorial loops below to 170 steps
        // (CLAUDE.md directive 5); the sum saturates, so huge arguments cannot overflow it.
        let sum = values.iter().fold(0u64, |acc, &v| acc.saturating_add(v));
        if sum > MAX_FACTORIAL_ARGUMENT {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "MULTINOMIAL: result overflow".to_string(),
            );
        }
        // Compute (sum)! / (n1! * n2! * ... * nk!) using f64 to handle large values.
        // Use the log-factorial approach for numerical stability.
        let log_result = log_factorial(sum) - values.iter().map(|&v| log_factorial(v)).sum::<f64>();
        let result = log_result.exp();
        if !result.is_finite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "MULTINOMIAL: result overflow".to_string(),
            );
        }
        CalcResult::Number(result.round())
    }
}
