use crate::calc_result::Range;
// new implementation gcd_lcm.rs
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

// Excel accepts 2^53 itself as an argument and as an LCM result; anything above is `#NUM!`
// (probes/gcd-lcm-bounds, lcm-result-bound, lcm-result-edge; docs/excel-semantics/numbers.md).
const MAX_LCM_GCD: i64 = 2_i64.pow(53);

// Euclidean gcd for i64 (non-negative inputs expected)
fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

// lcm(a, b) = a / gcd(a, b) * b
// done in i128: both inputs are at most 2^53, so the product is at most 2^106
fn lcm_i64(a: i64, b: i64) -> Option<i64> {
    if a == 0 || b == 0 {
        return Some(0);
    }
    let g = gcd_i64(a, b);
    let a_div_g = (a / g) as i128;
    // The limit applies to the rounded double: a product of 2^53+1 reads as 2^53 and is accepted
    // (probes/lcm-result-edge C1), while 2^53+2 is `#NUM!`.
    let prod = ((a_div_g * (b as i128)) as f64) as i128;
    if prod > MAX_LCM_GCD as i128 {
        None
    } else {
        Some(prod as i64)
    }
}

impl Model {
    fn gcd_lcm_impl<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        fn_name: &str,
        mut combine: F,
    ) -> CalcResult
    where
        F: FnMut(i64, i64) -> Option<i64>,
    {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut acc: Option<i64> = None;
        let mut saw_number = false;
        let mut has_range = false;

        let mut handle_number = |value: f64| -> Option<CalcResult> {
            if !value.is_finite() {
                return Some(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    format!("Non-finite number in {fn_name}"),
                ));
            }
            let n = value.trunc() as i64;

            if value < 0.0 {
                return Some(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    format!("{fn_name} only accepts non-negative integers"),
                ));
            } else if n > MAX_LCM_GCD {
                return Some(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Argument too large".to_string(),
                ));
            }
            saw_number = true;
            acc = Some(match acc {
                Some(cur) => match combine(cur, n) {
                    Some(v) => v,
                    None => {
                        return Some(CalcResult::new_error(
                            Error::NUM,
                            cell,
                            format!("{fn_name} result too large"),
                        ));
                    }
                },
                None => n,
            });
            None
        };

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    if let Some(res) = handle_number(value) {
                        return res;
                    }
                }
                CalcResult::String(s) => {
                    if let Ok(value) = self.cast_to_number(CalcResult::String(s), cell) {
                        if let Some(res) = handle_number(value) {
                            return res;
                        }
                    } else {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Non-numeric string".to_string(),
                        };
                    }
                }
                CalcResult::Boolean(_) => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Booleans not allowed in GCD".to_string(),
                    };
                }
                CalcResult::EmptyArg => {
                    return CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Empty argument".to_string(),
                    };
                }
                CalcResult::Range { left, right } => {
                    has_range = true;
                    if let Err(error) = self.for_each_cell_in_areas(
                        &[Range { left, right }],
                        cell,
                        |model, value| {
                            let number = match value {
                                CalcResult::Number(value) => value,
                                CalcResult::String(s) => model
                                    .cast_to_number(CalcResult::String(s), cell)
                                    .map_err(|_| CalcResult::Error {
                                        error: Error::VALUE,
                                        origin: cell,
                                        message: "Non-numeric string".to_string(),
                                    })?,
                                CalcResult::Boolean(_) => {
                                    return Err(CalcResult::Error {
                                        error: Error::VALUE,
                                        origin: cell,
                                        message: "Booleans not allowed in GCD".to_string(),
                                    });
                                }
                                CalcResult::EmptyCell | CalcResult::EmptyArg => 0.0,
                                _ => return Ok(()),
                            };
                            handle_number(number).map_or(Ok(()), Err)
                        },
                    ) {
                        return error;
                    }
                }
                CalcResult::Array(array) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    if let Some(res) = handle_number(value) {
                                        return res;
                                    }
                                }
                                ArrayNode::String(s) => {
                                    if let Ok(value) =
                                        self.cast_to_number(CalcResult::String(s), cell)
                                    {
                                        if let Some(res) = handle_number(value) {
                                            return res;
                                        }
                                    } else {
                                        return CalcResult::Error {
                                            error: Error::VALUE,
                                            origin: cell,
                                            message: "Non-numeric string".to_string(),
                                        };
                                    }
                                }
                                ArrayNode::Boolean(_) => {
                                    return CalcResult::Error {
                                        error: Error::VALUE,
                                        origin: cell,
                                        message: "Booleans not allowed in GCD".to_string(),
                                    };
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
                                ArrayNode::Empty => {
                                    // Excel behavior: empty cells are treated as 0 in GCD
                                    if let Some(res) = handle_number(0.0) {
                                        return res;
                                    }
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // ignore strings / booleans
                }
            }
        }

        if !saw_number && !has_range {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No valid numbers found".to_string(),
            };
        }

        CalcResult::Number(acc.unwrap_or(0) as f64)
    }

    pub(crate) fn fn_gcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.gcd_lcm_impl(args, cell, "GCD", |a, b| Some(gcd_i64(a, b)))
    }

    pub(crate) fn fn_lcm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.gcd_lcm_impl(args, cell, "LCM", lcm_i64)
    }
}
