//! Lifting for the text functions that take lists (TEXTSPLIT, TEXTBEFORE, TEXTAFTER) or a
//! pattern (REGEX*) next to scalar arguments: the scalar arguments lift over a range or an
//! array, one call per element, while the list arguments are taken whole (docs/excel-semantics/
//! arrays.md "Dynamic-array and LAMBDA function argument modes"; probes/m7-argument-modes).

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
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

/// Cells a lifted call counts for compiling one pattern (`Regex::new` on a pattern of up to
/// 32,767 characters can build a program of megabytes), charged per element when the pattern
/// lifts. DEVIATION, decision 0014: Excel has no such limit.
const COMPILE_COST: u64 = 4096;

impl Model {
    /// A one-cell reference as the value of its cell, any other value as it is.
    pub(super) fn collapse_single_cell(&mut self, value: CalcResult) -> CalcResult {
        match value {
            CalcResult::Range { left, right } if left == right => self.evaluate_cell(left),
            other => other,
        }
    }

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
            operands.push(match self.collapse_single_cell(value) {
                // Read once here: the cost of a lifted call depends on the text it holds.
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        )
                    } else {
                        match self.evaluate_range(left, right, cell) {
                            Ok(grid) => CalcResult::Array(grid),
                            Err(error) => error,
                        }
                    }
                }
                other => other,
            });
        }
        (operands, whole)
    }

    /// Runs `element` once on `operands`, or once per element when one lifts. A lifted element
    /// keeps its first value when the call gives an array (UNVERIFIED, text.md "TEXTSPLIT,
    /// TEXTBEFORE and TEXTAFTER lift").
    ///
    /// An element scans its text once per delimiter (`delimiters`, at least 1) and, when
    /// `compiles_patterns`, compiles its pattern (operand 1) if that lifts; the call is
    /// `#ERROR!` when that work exceeds `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION, decision
    /// 0014: Excel has no such limit).
    pub(super) fn lift_text_call(
        &mut self,
        operands: Vec<CalcResult>,
        cell: CellReferenceIndex,
        delimiters: usize,
        compiles_patterns: bool,
        mut element: impl FnMut(&mut Model, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        let lifts = operands.iter().any(LiftedArg::is_grid);
        let longest = match operands.first() {
            Some(CalcResult::String(s)) => s.len(),
            Some(CalcResult::Array(grid)) => grid
                .iter()
                .flatten()
                .map(|node| match node {
                    ArrayNode::String(s) => s.len(),
                    _ => 0,
                })
                .max()
                .unwrap_or(0),
            _ => 0,
        };
        let scans = u64::try_from(delimiters.max(1)).unwrap_or(u64::MAX);
        let mut cost = u64::try_from(longest)
            .unwrap_or(u64::MAX)
            .saturating_add(1)
            .saturating_mul(scans);
        if compiles_patterns && operands.get(1).is_some_and(LiftedArg::is_grid) {
            cost = cost.saturating_add(COMPILE_COST);
        }
        self.lift_values(
            operands,
            LiftCost::Cells(cost),
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
            CalcResult::EmptyArg | CalcResult::EmptyCell => return Ok(vec![]),
            err @ CalcResult::Error { .. } => return Err(err),
            scalar @ (CalcResult::String(_) | CalcResult::Number(_)) => {
                return Ok(vec![delimiter_text(
                    &calc_result_to_array_node(scalar),
                    cell,
                )?])
            }
            _ => return Err(not_a_string(cell)),
        };
        let mut delimiters = Vec::new();
        for item in grid.into_iter().flatten() {
            match item {
                ArrayNode::Empty => {}
                other => delimiters.push(delimiter_text(&other, cell)?),
            }
        }
        Ok(delimiters)
    }
}

/// The text of one delimiter: a string, or a number as its text.
fn delimiter_text(item: &ArrayNode, cell: CellReferenceIndex) -> Result<String, CalcResult> {
    match item {
        ArrayNode::String(s) => Ok(s.clone().into_lossy_string()),
        ArrayNode::Number(n) => Ok(number_to_text(*n)),
        _ => Err(not_a_string(cell)),
    }
}

fn not_a_string(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "delimiter must be a string".to_string())
}
