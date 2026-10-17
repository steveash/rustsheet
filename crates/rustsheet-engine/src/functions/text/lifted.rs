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
    limits::too_many_visits,
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
    /// keeps its first value when the call gives an array (TEXTSPLIT: probes/m7-argument-modes
    /// F13:H13; REGEXEXTRACT's `return_all` UNVERIFIED, text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER
    /// and REGEX* lift their text and scalar arguments").
    ///
    /// An element scans its own text once per delimiter (`delimiters`, at least 1) and, when
    /// `compiles_patterns`, compiles its pattern (operand 1) if that lifts. The work is the sum
    /// over the text's elements, each counted as often as the broadcast repeats it, so one long
    /// cell in a mostly blank column costs its own length once. The call is `#ERROR!` when the
    /// work exceeds `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION, decision 0014: Excel has no such
    /// limit), lifted or not.
    pub(super) fn lift_text_call(
        &mut self,
        operands: Vec<CalcResult>,
        cell: CellReferenceIndex,
        delimiters: usize,
        compiles_patterns: bool,
        mut element: impl FnMut(&mut Model, &[CalcResult]) -> CalcResult,
    ) -> CalcResult {
        let lifts = operands.iter().any(LiftedArg::is_grid);
        if let Some(error) = too_many_visits(
            text_call_cost(&operands, delimiters, compiles_patterns),
            cell,
        ) {
            return error;
        }
        self.lift_values(
            operands,
            LiftCost::Unit,
            &LiftOptions::EVERY_ELEMENT,
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

/// The shape of an operand: an array's rows and columns, 1x1 for a scalar.
fn operand_dims(operand: &CalcResult) -> (usize, usize) {
    match operand {
        CalcResult::Array(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
        _ => (1, 1),
    }
}

/// The work of one text call ([`Model::lift_text_call`]): every text element's length plus 1,
/// times the delimiters, times how often the broadcast repeats it (a dimension of 1 stretches
/// over the output's), plus [`COMPILE_COST`] per element when the pattern lifts. Saturating.
fn text_call_cost(operands: &[CalcResult], delimiters: usize, compiles_patterns: bool) -> u64 {
    let (rows, columns) = operands
        .iter()
        .map(operand_dims)
        .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
    let to_u64 = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    let length = |len: usize| to_u64(len).saturating_add(1);
    let text_work = match operands.first() {
        Some(CalcResult::Array(grid)) => {
            let (text_rows, text_columns) = operand_dims(&operands[0]);
            let repeats = to_u64(if text_rows == 1 { rows } else { 1 })
                .saturating_mul(to_u64(if text_columns == 1 { columns } else { 1 }));
            grid.iter()
                .flatten()
                .map(|node| match node {
                    ArrayNode::String(s) => length(s.len()),
                    _ => 1,
                })
                .fold(0u64, u64::saturating_add)
                .saturating_mul(repeats)
        }
        Some(CalcResult::String(s)) => {
            length(s.len()).saturating_mul(to_u64(rows.saturating_mul(columns)))
        }
        _ => to_u64(rows.saturating_mul(columns)),
    };
    let mut cost = text_work.saturating_mul(to_u64(delimiters.max(1)));
    if compiles_patterns && operands.get(1).is_some_and(LiftedArg::is_grid) {
        cost =
            cost.saturating_add(COMPILE_COST.saturating_mul(to_u64(rows.saturating_mul(columns))));
    }
    cost
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
