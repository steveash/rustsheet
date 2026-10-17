//! Array-literal lifting for the Analysis ToolPak-era date functions (EDATE, EOMONTH, WEEKNUM,
//! WORKDAY, NETWORKDAYS, YEARFRAC and their .INTL forms): an array in a date, count, return
//! type or basis argument gives an array of per-element results, while a multi-cell range in
//! the same place stays the scalar path's `#VALUE!` (docs/excel-semantics/dates.md "Date
//! function argument modes", probes/date-argument-modes).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::value_node,
    model::Model,
};

/// No element walks more days than the calendar has: a serial past it is `#NUM!` before any walk,
/// so a larger estimate would refuse inputs that cost nothing.
const MAX_WALK: u64 = 2_958_466;

/// The numbers of an evaluated argument: the number itself, or the numeric elements of an array.
fn numbers(value: &CalcResult) -> Vec<f64> {
    match value {
        CalcResult::Number(n) => vec![*n],
        CalcResult::Array(rows) => rows
            .iter()
            .flatten()
            .filter_map(|node| match node {
                ArrayNode::Number(n) => Some(*n),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The days NETWORKDAYS walks at most between its first two arguments: the spread of every
/// number either one holds. (`as u64` saturates, so a huge spread stays huge, never wraps.)
pub(super) fn day_span(values: &[CalcResult]) -> u64 {
    let all: Vec<f64> = values.iter().take(2).flat_map(numbers).collect();
    let (min, max) = all.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &n| {
        (lo.min(n.floor()), hi.max(n.floor()))
    });
    if all.is_empty() {
        return 1;
    }
    ((max - min) as u64).saturating_add(1).min(MAX_WALK)
}

/// The working days WORKDAY steps over at most: the largest magnitude of its second argument.
pub(super) fn step_days(values: &[CalcResult]) -> u64 {
    let largest = values.get(1).map_or(0.0, |days| {
        numbers(days)
            .iter()
            .fold(0.0_f64, |m, d| m.max(d.abs().floor()))
    });
    (largest as u64).saturating_add(1).min(MAX_WALK)
}

impl Model {
    /// Runs `scalar` on `args`, once per element when an argument at one of the `lifted`
    /// positions is an array (broadcast by [`Model::lift_with_cost`]). `visits` bounds the cells
    /// or days one element walks from the lifted arguments (evaluated, ranges excluded); with
    /// the cells of the arguments that do not lift (the holidays, read again by every element)
    /// the call is `#ERROR!` past `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION: Excel has no such
    /// limit). A multi-cell range at a lifted position stays a node, so `scalar` gives `#VALUE!`.
    pub(crate) fn lift_array_args(
        &mut self,
        args: &[Node],
        lifted: &[usize],
        cell: CellReferenceIndex,
        visits: impl Fn(&[CalcResult]) -> u64,
        scalar: impl Fn(&mut Self, &[Node], CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        let positions: Vec<usize> = lifted.iter().copied().filter(|&p| p < args.len()).collect();
        let mut values: Vec<CalcResult> = positions
            .iter()
            .map(|&p| self.evaluate_node_with_reference(&args[p], cell))
            .collect();
        // A range is left to `scalar` as its node: `lift_with_cost` would read it as an array.
        let ranges: Vec<bool> = values
            .iter_mut()
            .map(|value| match value {
                CalcResult::Range { .. } => {
                    *value = CalcResult::EmptyCell;
                    true
                }
                _ => false,
            })
            .collect();
        let lifting = values.iter().any(|v| matches!(v, CalcResult::Array(_)));
        let cost = visits(&values);
        self.lift_with_cost(
            values,
            |model| {
                let rest = (0..args.len())
                    .filter(|p| !positions.contains(p))
                    .fold(0u64, |sum, p| {
                        sum.saturating_add(model.walk_cost(&args[p], cell))
                    });
                cost.max(1).saturating_add(rest)
            },
            cell,
            |model, values| {
                let mut nodes = args.to_vec();
                for ((&p, value), is_range) in positions.iter().zip(values).zip(&ranges) {
                    if !is_range {
                        if let Some(node) = value_node(value) {
                            nodes[p] = node;
                        }
                    }
                }
                match scalar(model, &nodes, cell) {
                    // A .INTL element with a lifted weekend has no single value (unprobed, TODO(T-1784)); without a
                    // lifted date the weekend array is the result.
                    CalcResult::Array(_) if lifting => CalcResult::new_error(
                        Error::NIMPL,
                        cell,
                        "A lifted weekend inside a lifted date".to_string(),
                    ),
                    result => result,
                }
            },
        )
    }

    pub(crate) fn fn_eomonth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_array_args(args, &[0, 1], cell, |_| 1, Self::eomonth_scalar)
    }

    pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_array_args(args, &[0, 1], cell, |_| 1, Self::edate_scalar)
    }

    pub(crate) fn fn_networkdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_array_args(args, &[0, 1], cell, day_span, Self::networkdays_scalar)
    }

    pub(crate) fn fn_networkdays_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_array_args(args, &[0, 1], cell, day_span, Self::networkdays_intl_scalar)
    }
}
