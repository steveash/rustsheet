use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::aggregate_args::{AggregatePolicy, ArgReading, ErrorRule, ItemSource},
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

    /// Base function for AND, OR, XOR: `fold_fn` combines the running result (`None` at first)
    /// with each logical value the arguments give, read through the SPEC-0009 walker
    /// ([`LOGICAL_ARGS`]):
    /// - a boolean is itself, a number is TRUE unless 0, in any source;
    /// - direct text "TRUE" or "FALSE" (any case) counts; other text, referenced text and array
    ///   text are skipped (probes/and-or-xor-text-arguments, T-0258);
    /// - an empty argument is FALSE when nothing came before and changes nothing after; empty
    ///   cells are skipped; a whole-column range is clipped to the used area (`OR(F:LSE)`, T-0230);
    /// - the first error in argument order wins, and every argument is evaluated: Excel does not
    ///   short-circuit (T-0215, docs/excel-semantics/coercion.md, probes/and-or-xor-error-order).
    ///
    /// No logical value at all is `#VALUE!`.
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
        let walked = self.walk_aggregate_args(args, cell, LOGICAL_ARGS, |model, source, value| {
            let logical = match value {
                CalcResult::Boolean(value) => Some(value),
                CalcResult::Number(value) => Some(value != 0.0),
                CalcResult::EmptyArg => {
                    result = Some(result.unwrap_or(false));
                    None
                }
                text @ CalcResult::String(_) if source == ItemSource::Direct => {
                    model.cast_to_bool(text, cell).ok()
                }
                // A direct LAMBDA is `#VALUE!`; the walker holds this error, so a direct error
                // beats it in either order (probes/and-or-xor-picked-text J1-J18, T-1685).
                CalcResult::Lambda(_) => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Lambdas not supported".to_string(),
                    ));
                }
                // Empty values and the rest of the text are skipped.
                _ => None,
            };
            if let Some(value) = logical {
                result = Some(fold_fn(result, value));
            }
            Ok(())
        });
        if let Err(error) = walked {
            return error;
        }
        match result {
            Some(result) => CalcResult::Boolean(result),
            None => CalcResult::new_error(
                Error::VALUE,
                cell,
                "No logical values in argument list".to_string(),
            ),
        }
    }
}

/// How AND, OR and XOR read their arguments (SPEC-0009 "Consumer mapping"): as
/// `evaluate_aggregate_arg` gives them (picked cells are references), first error wins, an
/// empty argument is an item.
const LOGICAL_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::First,
    skip_empty_args: false,
};
