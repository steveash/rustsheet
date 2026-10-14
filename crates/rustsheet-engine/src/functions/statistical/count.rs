use crate::calc_result::Range;
use crate::expressions::parser::ArrayNode;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::is_span_or_if_of_span;
use crate::reference_operators::{not_a_reference_array, RefArg};
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// COUNTBLANK's `#VALUE!` for an argument that is not a reference.
fn countblank_needs_reference(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "COUNTBLANK needs a reference".to_string(),
    )
}

impl Model {
    pub(crate) fn fn_count(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            // References (also via IF/INDEX) arrive as a Range; the scalar
            // arms below only see values.
            match self.evaluate_picked_reference_arg(arg, cell) {
                RefArg::Value(CalcResult::Number(_)) => {
                    result += 1.0;
                }
                RefArg::Value(CalcResult::Boolean(_))
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) =>
                {
                    result += 1.0;
                }
                // COUNT(5,) is 2 (probes/average-value-args F16).
                RefArg::Value(CalcResult::EmptyArg) => result += 1.0,
                // Direct text counts when it coerces to a number as SUM's does, clock, date,
                // currency and percent text included (probes/count-direct-text; T-0252). Only
                // number-ness matters, so the 15-digit cut cannot change the answer
                // (docs/excel-semantics/numbers.md "15-digit literal parsing").
                RefArg::Value(CalcResult::String(s))
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. })
                        && self.cast_number(&s).is_some_and(f64::is_finite) =>
                {
                    result += 1.0;
                }
                RefArg::Ranges(areas) => {
                    if let Err(error) =
                        self.for_each_cell_or_error_in_areas(&areas, cell, |_, value| {
                            if let CalcResult::Number(_) = value {
                                result += 1.0;
                            }
                            Ok(())
                        })
                    {
                        return error;
                    }
                }
                // Only an array's numbers count (probes/average-value-args F32).
                RefArg::Value(CalcResult::Array(array)) => {
                    let numbers = array.iter().flatten();
                    result += numbers
                        .filter(|v| matches!(v, ArrayNode::Number(_)))
                        .count() as f64;
                }
                RefArg::Value(_) => {
                    // Ignore everything else
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_counta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            // A reference, 3-D ones included, arrives as a Range: its non-empty cells count.
            // TAKE, DROP, INDEX, OFFSET and the like over a range pick a reference too, so
            // `COUNTA(TAKE(B3:B3,1))` of a blank is 0 (probes/subtotal-reference-operands-c H13).
            match self.evaluate_picked_reference_arg(arg, cell) {
                RefArg::Value(CalcResult::EmptyCell) => {}
                // Excel: an omitted argument and the empty value IFERROR/IFNA give for a blank
                // cell both count (COUNTA(,) = 2, COUNTA(IFERROR(B1,1)) = 1), unlike a blank
                // reference; see docs/excel-semantics/logic.md "IFERROR / IFNA of a blank cell is an empty value".
                RefArg::Value(CalcResult::EmptyArg) => result += 1.0,
                RefArg::Ranges(areas) => {
                    if let Err(error) =
                        self.for_each_cell_or_error_in_areas(&areas, cell, |_, value| {
                            if !matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                                result += 1.0;
                            }
                            Ok(())
                        })
                    {
                        return error;
                    }
                }
                // Every element of an array value counts, the empty values IFERROR gives for
                // blank cells included: `COUNTA(IFERROR(B1:B2,1))` with B1 blank is 2
                // (probes/counta-empty-value D18).
                RefArg::Value(CalcResult::Array(a)) => {
                    result += a.iter().map(Vec::len).sum::<usize>() as f64;
                }
                RefArg::Value(_) => {
                    result += 1.0;
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_countblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // COUNTBLANK requires only one argument
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            let value = if self.is_multi_area(arg) && !is_span_or_if_of_span(arg) {
                // A union is #VALUE! here (probes/multiarea-count H1), and so is a 3-D span bound
                // by LET (probes/3d-ref-let-bound-span-readers C11); one area is a range.
                self.evaluate_reference_arg(arg, cell)
                    .into_single_sheet_area(cell)
            } else if is_span_or_if_of_span(arg) {
                self.evaluate_single_sheet_arg(arg, cell)
            } else {
                // Read once in reference context, so a reference to one error cell (also through
                // CHOOSE, INDEX, OFFSET, XLOOKUP or TAKE) is a range that counts 0, while an error
                // value is the result (probes/countblank-error-cell-refs, countblank-error-forms-b,
                // countblank-choose-errors).
                self.evaluate_picked_reference_arg(arg, cell)
                    .into_single_sheet_area(cell)
            };
            // A name that is not a reference is an error, never a count: an error literal or a
            // formula body gives its own error, any other formula value `#VALUE!`, `=""`
            // included. A body is a reference when it reads as one in reference context, a
            // single cell and INDEX too (probes/countblank-formula-names, excel-graph
            // 2026-10-09, T-1199); cell and range names are references as before.
            if let Some(error) = self.defined_name_error_literal(arg) {
                return CalcResult::new_error(error, cell, "Defined name is an error".to_string());
            }
            let value = if self.defined_name_formula(arg).is_none() {
                value
            } else if self.is_multi_area(arg) {
                // A body that may stand for a union keeps the union path; its error is the result,
                // and a scalar it picks (`IF(TRUE,5,6)`, `CHOOSE(1,5,6)`, `IF(TRUE,"",1)`) is
                // `#VALUE!` like any other name that is not a reference
                // (probes/countblank-formula-names-b, excel-graph 2026-10-09, T-1207).
                match value {
                    error @ CalcResult::Error { .. } => return error,
                    range @ CalcResult::Range { .. } => range,
                    _ => return countblank_needs_reference(cell),
                }
            } else {
                match self.defined_name_range_operand(arg, cell) {
                    range @ CalcResult::Range { .. } => range,
                    error @ CalcResult::Error { .. } => return error,
                    _ => return countblank_needs_reference(cell),
                }
            };
            match value {
                CalcResult::Range { left, right } => {
                    // Only the used area is walked: the cells past it are empty,
                    // so every cell that is not blank is found in the walk.
                    let mut filled = 0.0;
                    if let Err(error) = self.for_each_cell_or_error_in_areas(
                        &[Range { left, right }],
                        cell,
                        |_, value| {
                            match value {
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::String(s) if s.is_empty() => {}
                                _ => filled += 1.0,
                            }
                            Ok(())
                        },
                    ) {
                        return error;
                    }
                    let side = |first: i32, last: i32| f64::from(last - first + 1);
                    result += side(left.row, right.row) * side(left.column, right.column) - filled;
                }
                // An argument that is an error gives that error: a span with a missing end
                // (`#NAME?`), `CHOOSE(1,NA())`, an empty intersection (`#NULL!`), a 3-D or union
                // `#VALUE!` (probes/countblank-choose-errors, excel-graph 2026-10-10, T-1181).
                error @ CalcResult::Error { .. } => return error,
                // Any other value is not a reference, so COUNTBLANK is `#VALUE!`; an array lifts it
                // to one `#VALUE!` per element: a lifted INDEX, CHOOSE or IF with an array
                // condition, and `CHOOSE(1,5)` or `CHOOSE(1,"")` (probes/countblank-array-values,
                // probes/index-lifted-edge-cases H131, excel-graph 2026-10-13, T-1437).
                CalcResult::Array(values) => return not_a_reference_array(values),
                _ => return countblank_needs_reference(cell),
            };
        }
        CalcResult::Number(result)
    }
}
