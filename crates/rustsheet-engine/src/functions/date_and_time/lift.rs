//! Array-literal lifting for the Analysis ToolPak-era date functions (EDATE, EOMONTH, WEEKNUM,
//! WORKDAY, NETWORKDAYS, YEARFRAC and their .INTL forms): an array in a date, count, return
//! type or basis argument gives an array of per-element results, while a multi-cell range in
//! the same place stays the scalar path's `#VALUE!` (docs/excel-semantics/dates.md "Date
//! function argument modes", probes/date-argument-modes).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        types::CellReferenceIndex,
    },
    functions::util::array_node_to_calc_result,
    limits::{check_array_size, too_many_visits},
    model::Model,
};

/// The node that stands for an already evaluated argument, or `None` when `value` is one the
/// original node must stay for (a range keeps its reference).
fn value_node(value: &CalcResult) -> Option<Node> {
    match value {
        CalcResult::Number(n) => Some(Node::NumberKind(*n)),
        CalcResult::String(s) => Some(Node::StringKind(s.to_string())),
        CalcResult::Boolean(b) => Some(Node::BooleanKind(*b)),
        CalcResult::Error { error, .. } => Some(Node::ErrorKind(error.clone())),
        CalcResult::EmptyArg => Some(Node::EmptyArgKind),
        _ => None,
    }
}

fn array_element(value: &CalcResult, row: usize, column: usize) -> Option<ArrayNode> {
    let CalcResult::Array(rows) = value else {
        return None;
    };
    let (height, width) = (rows.len(), rows.first().map_or(0, Vec::len));
    // A size-1 dimension repeats; a position past a shorter array is #N/A (as the operators).
    let at = |size: usize, index: usize| match size {
        1 => Some(0),
        _ => (index < size).then_some(index),
    };
    Some(
        at(height, row)
            .zip(at(width, column))
            .and_then(|(r, c)| rows.get(r).and_then(|row| row.get(c)))
            .cloned()
            .unwrap_or(ArrayNode::Error(crate::expressions::token::Error::NA)),
    )
}

/// The days NETWORKDAYS walks between its first two (numeric) arguments.
pub(super) fn day_span(values: &[CalcResult]) -> u64 {
    match values {
        [CalcResult::Number(start), CalcResult::Number(end), ..] => {
            (end.floor() - start.floor()).abs() as u64 + 1
        }
        _ => 1,
    }
}

/// The working days WORKDAY steps over: the magnitude of its second (numeric) argument.
pub(super) fn step_days(values: &[CalcResult]) -> u64 {
    match values {
        [_, CalcResult::Number(days), ..] => days.abs().floor() as u64 + 1,
        _ => 1,
    }
}

impl Model {
    /// Runs `scalar` on `args`, once per element when an argument at one of the `lifted`
    /// positions is an array (arguments broadcast like the arithmetic operators). `visits`
    /// estimates the cells or days one element walks from its lifted arguments; the call is
    /// `#ERROR!` when the total passes `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION: Excel has no
    /// such limit). Each lifted argument is evaluated once; a multi-cell range is left to
    /// `scalar`, which gives `#VALUE!`.
    pub(crate) fn lift_array_args(
        &mut self,
        args: &[Node],
        lifted: &[usize],
        cell: CellReferenceIndex,
        visits: impl Fn(&[CalcResult]) -> u64,
        scalar: impl Fn(&mut Self, &[Node], CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        let positions: Vec<usize> = lifted.iter().copied().filter(|&p| p < args.len()).collect();
        let values: Vec<CalcResult> = positions
            .iter()
            .map(|&p| self.evaluate_node_in_context(&args[p], cell))
            .collect();
        let element_args = |values: &[CalcResult]| -> Vec<Node> {
            let mut nodes = args.to_vec();
            for (&p, value) in positions.iter().zip(values) {
                if let Some(node) = value_node(value) {
                    nodes[p] = node;
                }
            }
            nodes
        };
        if !values.iter().any(|v| matches!(v, CalcResult::Array(_))) {
            return scalar(self, &element_args(&values), cell);
        }
        let dims = |value: &CalcResult| match value {
            CalcResult::Array(rows) => (rows.len(), rows.first().map_or(0, Vec::len)),
            _ => (1, 1),
        };
        let rows = values.iter().map(|v| dims(v).0).max().unwrap_or(1);
        let columns = values.iter().map(|v| dims(v).1).max().unwrap_or(1);
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let element_values = |row: usize, column: usize| -> Vec<CalcResult> {
            values
                .iter()
                .map(|value| match array_element(value, row, column) {
                    Some(node) => array_node_to_calc_result(&node, cell),
                    None => value.clone(),
                })
                .collect()
        };
        let mut total = 0u64;
        for row in 0..rows {
            for column in 0..columns {
                total = total.saturating_add(visits(&element_values(row, column)).max(1));
            }
        }
        if let Some(error) = too_many_visits(total, cell) {
            return error;
        }
        let output = (0..rows)
            .map(|row| {
                (0..columns)
                    .map(|column| {
                        let nodes = element_args(&element_values(row, column));
                        match scalar(self, &nodes, cell) {
                            CalcResult::Number(n) => ArrayNode::Number(n),
                            CalcResult::Error { error, .. } => ArrayNode::Error(error),
                            // The weekend lift of a .INTL function inside an array element has
                            // no single value (unprobed): #VALUE!.
                            _ => ArrayNode::Error(crate::expressions::token::Error::VALUE),
                        }
                    })
                    .collect()
            })
            .collect();
        CalcResult::Array(output)
    }
}
