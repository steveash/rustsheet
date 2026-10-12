use crate::calc_result::Range;
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::array_node_to_calc_result,
    model::Model,
};

impl Model {
    /// NOT lifts over ranges and arrays, one result per element (T-0225,
    /// docs/excel-semantics/arrays.md). Each element is cast like a scalar argument:
    /// NOT of text "TRUE" is FALSE, of other text `#VALUE!` and of an empty cell TRUE
    /// (probes/not-array-cast, T-0260).
    pub(crate) fn fn_not(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let array = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => array,
                Err(error) => return error,
            },
            CalcResult::Array(array) => array,
            error @ CalcResult::Error { .. } => return error,
            value => {
                return match self.cast_to_bool(value, cell) {
                    Ok(f) => CalcResult::Boolean(!f),
                    Err(s) => s,
                };
            }
        };
        CalcResult::Array(
            array
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|node| {
                            match self.cast_to_bool(array_node_to_calc_result(&node, cell), cell) {
                                Ok(f) => ArrayNode::Boolean(!f),
                                Err(CalcResult::Error { error, .. }) => ArrayNode::Error(error),
                                // An ArrayNode element is never a range or array, so this is unreachable.
                                Err(_) => ArrayNode::Error(Error::VALUE),
                            }
                        })
                        .collect()
                })
                .collect(),
        )
    }

    pub(crate) fn fn_and(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.logical_nary(args, cell, |acc, value| acc.unwrap_or(true) && value)
    }

    pub(crate) fn fn_or(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.logical_nary(args, cell, |acc, value| acc.unwrap_or(false) || value)
    }

    pub(crate) fn fn_xor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.logical_nary(args, cell, |acc, value| acc.unwrap_or(false) ^ value)
    }

    /// Base function for AND, OR, XOR. These are all n-ary functions that perform a boolean operation on a series of
    /// boolean values. These boolean values are sourced from `args`. Note that there is not a 1-1 relationship between
    /// arguments and boolean values evaluated (see how Ranges are handled for example).
    ///
    /// Each argument in `args` is evaluated and the resulting value is interpreted as a boolean as follows:
    /// - Boolean: The value is used directly.
    /// - Number: 0 is FALSE, all other values are TRUE.
    /// - Range: Each cell in the range is evaluated as if they were individual arguments with some caveats
    /// - Empty arg: FALSE
    /// - Empty cell & String: Ignored, behaves exactly like the argument wasn't passed in at all
    /// - Error: Propagated
    ///
    /// If no arguments are provided, or all arguments are ignored, the function returns a #VALUE! error
    ///
    /// **`fold_fn`:** The function that combines the running result with the next value boolean value. The running result
    /// starts as `None`.
    ///
    /// Every argument is evaluated: Excel does not short-circuit, so an error after a decisive
    /// FALSE (AND) or TRUE (OR) still wins (T-0215, docs/excel-semantics/coercion.md).
    fn logical_nary(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        fold_fn: fn(Option<bool>, bool) -> bool,
    ) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = None;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Boolean(value) => result = Some(fold_fn(result, value)),
                CalcResult::Number(value) => result = Some(fold_fn(result, value != 0.0)),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    // The walk is clipped to the used area: a whole-column argument such as
                    // `OR(F:LSE)` must not visit (and record a read of) billions of empty
                    // cells (CLAUDE.md directive 5, T-0230). Empty cells are ignored anyway.
                    let walk = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |_, value| {
                            match value {
                                CalcResult::Boolean(value) => result = Some(fold_fn(result, value)),
                                CalcResult::Number(value) => {
                                    result = Some(fold_fn(result, value != 0.0));
                                }
                                error @ CalcResult::Error { .. } => return Err(error),
                                // A cell's value is never an empty argument or a range.
                                CalcResult::EmptyArg
                                | CalcResult::Range { .. }
                                | CalcResult::String { .. }
                                | CalcResult::EmptyCell => {}
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    return Err(CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    });
                                }
                            }
                            Ok(())
                        },
                    );
                    if let Err(error) = walk {
                        return error;
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::EmptyArg => result = Some(result.unwrap_or(false)),
                // Strings are ignored unless they are "TRUE" or "FALSE" (case insensitive). EXCEPT if the string value
                // comes from a reference, in which case it is always ignored regardless of its value.
                CalcResult::String(..) => {
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) {
                        if let Ok(f) = self.get_boolean(arg, cell) {
                            result = Some(fold_fn(result, f));
                        }
                    }
                }
                // References to empty cells are ignored. If all args are ignored the result is #VALUE!
                CalcResult::EmptyCell => {}
                // Each element of the array is treated as an individual value, exactly like the
                // cells of a range. Booleans and numbers are evaluated, strings and empties are
                // ignored and errors are propagated.
                CalcResult::Array(array) => {
                    for array_row in &array {
                        for node in array_row {
                            match node {
                                ArrayNode::Boolean(value) => result = Some(fold_fn(result, *value)),
                                ArrayNode::Number(value) => {
                                    result = Some(fold_fn(result, *value != 0.0));
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::new_error(
                                        error.clone(),
                                        cell,
                                        "Error in array".to_string(),
                                    )
                                }
                                ArrayNode::String(_) | ArrayNode::Empty => {}
                            }
                        }
                    }
                }
                CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Lambdas not supported".to_string(),
                    }
                }
            }
        }

        if let Some(result) = result {
            CalcResult::Boolean(result)
        } else {
            CalcResult::new_error(
                Error::VALUE,
                cell,
                "No logical values in argument list".to_string(),
            )
        }
    }
}
