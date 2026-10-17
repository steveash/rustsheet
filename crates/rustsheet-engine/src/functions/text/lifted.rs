//! Lifting for the text functions that take lists (TEXTSPLIT, TEXTBEFORE, TEXTAFTER) or a
//! pattern (REGEX*) next to scalar arguments: the scalar arguments lift over a range or an
//! array, one call per element, while the list arguments are taken whole (docs/excel-semantics/
//! arrays.md "Dynamic-array and LAMBDA function argument modes"; probes/m7-argument-modes).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{
        lift::{LiftCost, LiftOptions, LiftedArg},
        util::array_node_to_calc_result,
    },
    model::Model,
    number_format::number_to_text,
};

/// Cells one lifted element counts for against `limits::MAX_LIFTED_CELL_VISITS`: an element
/// scans a text of up to 32,767 characters, so a lift of this many elements stays bounded
/// (DEVIATION, decision 0014: Excel has no such limit).
const ELEMENT_COST: u64 = 256;

impl Model {
    /// Evaluates the arguments of a lifting text call. Those at the indices in `lists` are
    /// returned whole, in order, and left as `EmptyArg` in the first vector, which holds the
    /// operands that lift (a single-cell reference is its value).
    pub(super) fn lift_text_operands(
        &mut self,
        args: &[Node],
        lists: &[usize],
        cell: CellReferenceIndex,
    ) -> (Vec<CalcResult>, Vec<CalcResult>) {
        let mut operands = Vec::with_capacity(args.len());
        let mut whole = Vec::with_capacity(lists.len());
        for (index, node) in args.iter().enumerate() {
            let value = self.evaluate_node_in_context(node, cell);
            if lists.contains(&index) {
                whole.push(value);
                operands.push(CalcResult::EmptyArg);
                continue;
            }
            operands.push(match value {
                CalcResult::Range { left, right } if left == right => self.evaluate_cell(left),
                other => other,
            });
        }
        (operands, whole)
    }

    /// Runs `element` once on `operands`, or once per element when one lifts. A lifted element
    /// keeps its first value when the call gives an array (UNVERIFIED, text.md "TEXTSPLIT,
    /// TEXTBEFORE and TEXTAFTER lift").
    pub(super) fn lift_text_call(
        &mut self,
        operands: Vec<CalcResult>,
        cell: CellReferenceIndex,
        mut element: impl FnMut(&mut Model, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        let lifts = operands.iter().any(LiftedArg::is_grid);
        self.lift_values(
            operands,
            LiftCost::Cells(ELEMENT_COST),
            &LiftOptions {
                reuse_equal_runs: false,
            },
            cell,
            &mut |model, values| match element(model, values) {
                CalcResult::Array(grid) if lifts => match grid.first().and_then(|r| r.first()) {
                    Some(node) => array_node_to_calc_result(node, cell),
                    None => CalcResult::new_error(Error::VALUE, cell, String::new()),
                },
                other => other,
            },
        )
    }

    /// The delimiters a list argument holds: the strings and numbers (as text) of an array or a
    /// range, blanks skipped; any other value is `#VALUE!`.
    pub(super) fn delimiter_list(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let grid = match value {
            CalcResult::Range { left, right } => self.evaluate_range(left, right, cell)?,
            CalcResult::Array(grid) => grid,
            CalcResult::String(s) => return Ok(vec![s.into_lossy_string()]),
            CalcResult::Number(n) => return Ok(vec![number_to_text(n)]),
            CalcResult::EmptyArg | CalcResult::EmptyCell => return Ok(vec![]),
            err @ CalcResult::Error { .. } => return Err(err),
            _ => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "delimiter must be a string".to_string(),
                ))
            }
        };
        let mut delimiters = Vec::new();
        for item in grid.into_iter().flatten() {
            match item {
                ArrayNode::String(s) => delimiters.push(s.into_lossy_string()),
                ArrayNode::Number(n) => delimiters.push(number_to_text(n)),
                ArrayNode::Empty => {}
                _ => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "delimiter must be a string".to_string(),
                    ))
                }
            }
        }
        Ok(delimiters)
    }
}
