use crate::calc_result::Range;
use crate::x87_log10::excel_log10;
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

/// MULTINOMIAL of `values`, whose sum is at most 170, as Excel computes it. One double running
/// sum adds `log10(k)` for k = 1..=sum, then subtracts `log10(k)` for k = 1..=n for each value
/// from the last to the first, and the result is `exp(sum * ln 10)`, or `1 / exp(-sum * ln 10)`
/// when the sum is negative (only a lone argument's residue is: `(5)` 0.9999999999999996, not the
/// nearer 0.9999999999999994). Its rounding is Excel's: `MULTINOMIAL(2,3)` 9.999999999999998,
/// `(170)` 1.0000000000001974, and `(20,150)` differs from `(150,20)`
/// (docs/excel-semantics/numbers.md "MULTINOMIAL is inexact"; fit on
/// probes/multinomial-algorithm-sweep, multinomial-single-arguments, multinomial-large-arguments
/// and math-argument-modes, T-1190 and T-1600: all 240 cells bit-exact). The terms are
/// Excel's x87 log10 (`excel_log10`, T-2004; on 1..=170 it is the correctly rounded log10 the fit
/// found), not the platform's, which is one ulp off at 11, 40, 43, 53, ...; the last step is the
/// platform `exp` (glibc's gives the fitted values).
fn multinomial_value(values: &[u64]) -> f64 {
    let sum: u64 = values.iter().sum();
    let mut log = 0.0;
    for k in 1..=sum {
        log += excel_log10(k as f64);
    }
    for &n in values.iter().rev() {
        for k in 1..=n {
            log -= excel_log10(k as f64);
        }
    }
    let exponent = log * std::f64::consts::LN_10;
    if exponent < 0.0 {
        1.0 / (-exponent).exp()
    } else {
        exponent.exp()
    }
}

/// Numeric text as its number: typed (`MULTINOMIAL("2",3)` is `MULTINOMIAL(2,3)`) and in a range
/// cell (4, "2", 6 reads as 4, 2, 6), as GCD and LCM read it (probes/math-argument-modes C9, C47,
/// C52; docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells"). Other text stays
/// text, which is `#VALUE!`.
fn numeric_text_as_number(model: &Model, value: CalcResult) -> CalcResult {
    match value {
        CalcResult::String(text) => match model.cast_number(&text) {
            Some(number) => CalcResult::Number(number),
            None => CalcResult::String(text),
        },
        other => other,
    }
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

        for (index, arg) in args.iter().enumerate() {
            let evaluated = self.evaluate_node_in_context(arg, cell);
            let evaluated = numeric_text_as_number(self, evaluated);
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
                        |model, value| match numeric_text_as_number(model, value) {
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
                        },
                    ) {
                        return error;
                    }
                }
                CalcResult::Array(arr) => {
                    for row in arr {
                        for node in row {
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
                // Excel: an omitted first argument is `#N/A`, a later one is skipped
                // (probes/gcd-lcm-multinomial-negative-precedence E51:E71).
                CalcResult::EmptyArg if index == 0 => {
                    return CalcResult::new_error(
                        Error::NA,
                        cell,
                        "MULTINOMIAL: empty first argument".to_string(),
                    )
                }
                CalcResult::EmptyArg => {}
                CalcResult::EmptyCell => values.push(0),
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
        let result = multinomial_value(&values);
        if !result.is_finite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "MULTINOMIAL: result overflow".to_string(),
            );
        }
        CalcResult::Number(result)
    }
}
