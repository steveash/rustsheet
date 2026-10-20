use crate::calc_result::Range;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::external_reference::PickedReference;
use crate::functions::util::aggregate_args::{
    AggregatePolicy, AggregateWalk, ArgReading, ErrorRule, GridReading, ItemSource,
};
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

/// Whether COUNTBLANK counts a cell's value as blank: empty, or the empty string.
fn is_countblank_blank(value: &CalcResult) -> bool {
    match value {
        CalcResult::EmptyCell | CalcResult::EmptyArg => true,
        CalcResult::String(s) => s.is_empty(),
        _ => false,
    }
}

/// COUNT: errors are ignored (syntax.md), so the walker hands them on as items.
const COUNT_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::Items,
    skip_empty_args: false,
    grids: GridReading::Values,
};

/// COUNTA counts errors; it reads a picked reference keeping the cached blanks of a linked
/// workbook.
const COUNTA_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::PickedKeepingBlanks,
    errors: ErrorRule::Items,
    skip_empty_args: false,
    grids: GridReading::Values,
};

impl Model {
    pub(crate) fn fn_count(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        if let Err(error) =
            self.walk_aggregate_args(args, cell, COUNT_ARGS, |model, source, value| {
                let counts = match (source, &value) {
                    // Only the numbers of a reference or an array count (probes/average-value-args
                    // F32).
                    (_, CalcResult::Number(_)) => true,
                    // COUNT(5,) is 2 (probes/average-value-args F16).
                    (ItemSource::Direct | ItemSource::ReferenceValue, CalcResult::EmptyArg) => true,
                    (ItemSource::Direct, CalcResult::Boolean(_)) => true,
                    // Direct text counts when it coerces to a number as SUM's does, clock, date,
                    // currency and percent text included (probes/count-direct-text; T-0252). Only
                    // number-ness matters, so the 15-digit cut cannot change the answer
                    // (docs/excel-semantics/numbers.md "15-digit literal parsing").
                    (ItemSource::Direct, CalcResult::String(text)) => {
                        model.cast_number(text).is_some_and(f64::is_finite)
                    }
                    _ => false,
                };
                if counts {
                    result += 1.0;
                }
                Ok(())
            })
        {
            return error;
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_counta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        // The cells of another workbook, counted from the externalLink cache.
        let mut external = 0.0;
        let mut result = 0.0;
        let mut walk = AggregateWalk::new(COUNTA_ARGS);
        let mut visit = |_: &mut Self, source: ItemSource, value: CalcResult| {
            let counts = match (source, &value) {
                // Excel: an omitted argument and the empty value IFERROR/IFNA give for a blank
                // cell both count (COUNTA(,) = 2, COUNTA(IFERROR(B1,1)) = 1), unlike a blank
                // reference; see docs/excel-semantics/logic.md "IFERROR / IFNA of a blank cell is
                // an empty value".
                (ItemSource::Direct | ItemSource::ReferenceValue, CalcResult::EmptyCell) => false,
                // Every element of an array value counts, the empty values IFERROR gives for blank
                // cells included: `COUNTA(IFERROR(B1:B2,1))` with B1 blank is 2
                // (probes/counta-empty-value D18). The walker never hands on a blank cell.
                _ => true,
            };
            if counts {
                result += 1.0;
            }
            Ok(())
        };
        for arg in args {
            // The cells of another workbook count the cells its externalLink cache holds
            // (`COUNTA([1]Uses!$A$1:$B$3)` is 5; fixtures/external-link-cached-ranges), and none
            // without a cache, written or reached through INDEX, a union or an intersection,
            // although `='[other.xlsx]Sheet1'!A1` is `#BLOCKED!` (`COUNTA(…!A1)` is 0,
            // `COUNTA(…!A1:B2,1)` 1; probes/external-ref-no-link-shapes A16,
            // external-ref-no-link-index A8, A10, A12, A23, A30). An error instead of a
            // reference is a value, counted. So for one IF or CHOOSE picks (`COUNTA(IF(TRUE,
            // …!A1:B2))` is 0; probes/external-ref-no-link-ifchoose A4, A18, A30).
            match self.pick_reference(arg, cell) {
                PickedReference::External(areas) => match areas {
                    Ok(areas) => {
                        external += self.external_non_empty_count(&areas).unwrap_or(0) as f64;
                    }
                    Err(_) => external += 1.0,
                },
                PickedReference::Node(arg) => {
                    // A reference, 3-D ones included, arrives as a Range: its non-empty cells
                    // count. TAKE, DROP, INDEX, OFFSET and the like over a range pick a reference
                    // too, so `COUNTA(TAKE(B3:B3,1))` of a blank is 0
                    // (probes/subtotal-reference-operands-c H13).
                    if let Err(error) = self.walk_aggregate_arg(arg, cell, &mut walk, &mut visit) {
                        return error;
                    }
                }
                PickedReference::Value(value) => {
                    let read = RefArg::Value(value);
                    if let Err(error) =
                        self.walk_aggregate_read(read, false, cell, &mut walk, &mut visit)
                    {
                        return error;
                    }
                }
            }
        }
        CalcResult::Number(result + external)
    }

    pub(crate) fn fn_countblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // COUNTBLANK requires only one argument
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            // COUNTBLANK over the cells of another workbook is `#VALUE!`, cached or not
            // (probes/external-ref-no-link-index A9; `COUNTBLANK([1]Uses!$B$3)`,
            // fixtures/external-link-cached-ranges), as COUNTIF and SUMIF are, and so is one IF
            // picks (probes/external-ref-no-link-ifchoose A24).
            let (arg, picked_value) = match self.pick_reference(arg, cell) {
                PickedReference::Node(node) => (node, None),
                PickedReference::Value(value) => (arg, Some(value)),
                PickedReference::External(areas) => {
                    return match areas {
                        Ok(_) => CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "COUNTBLANK of an external reference".to_string(),
                        ),
                        Err(error) => error,
                    };
                }
            };
            let value = if let Some(value) = picked_value {
                value
            } else if self.is_multi_area(arg) && !is_span_or_if_of_span(arg) {
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
                            if !is_countblank_blank(&value) {
                                filled += 1.0;
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
