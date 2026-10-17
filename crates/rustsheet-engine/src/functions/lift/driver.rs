//! The lifting driver's broadcasting core (SPEC-0011 "The driver"), moved from
//! `statistical/if_ifs.rs` by T-1784 with no result change.

use crate::arithmetic::{bcast_get, MISSING};
use crate::cast::calc_result_to_array_node;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::array_node_to_calc_result;
use crate::limits::{check_array_size, too_many_visits};
use crate::{
    calc_result::CalcResult,
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

/// One lifted argument as the driver holds it.
pub(crate) enum LiftedArg {
    /// One value for every element.
    Scalar(CalcResult),
    /// An array, or a multi-cell range read once into its cells.
    Grid(Vec<Vec<ArrayNode>>),
}

impl LiftedArg {
    /// Whether `value` lifts the call, that is, becomes a [`LiftedArg::Grid`]: an array or a
    /// range. IFS and SWITCH ask this before they evaluate their remaining arguments.
    pub(crate) fn is_grid(value: &CalcResult) -> bool {
        matches!(value, CalcResult::Range { .. } | CalcResult::Array(_))
    }

    fn dims(&self) -> (usize, usize) {
        match self {
            LiftedArg::Scalar(_) => (1, 1),
            LiftedArg::Grid(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
        }
    }

    /// The grid slot at output position (`row`, `column`), `None` for a scalar: a dimension of
    /// length 1 stretches, a position past a shorter array is `#N/A`
    /// (probes/ifs-array-criteria H11, H33-H37).
    fn element(&self, row: usize, column: usize) -> Option<&ArrayNode> {
        match self {
            LiftedArg::Scalar(_) => None,
            LiftedArg::Grid(grid) => Some(bcast_get(grid, row, column).unwrap_or(&MISSING)),
        }
    }
}

/// What one element costs, for the cell-visit cap (`limits::MAX_LIFTED_CELL_VISITS`). The total
/// is this cost times the number of lifted arguments times the elements computed (one per run
/// of equal elements under [`LiftOptions::reuse_equal_runs`]), as the *IF(S) lift charged it.
pub(crate) enum LiftCost<'a> {
    /// Every element visits this many cells per argument.
    Cells(u64),
    /// `Model::walk_cost` of this node per element and argument (the *IF(S) front end); the
    /// node is only measured when the call lifts.
    Walk(&'a Node),
}

/// How [`Model::lift_values`] runs the elements.
pub(crate) struct LiftOptions {
    /// Reuse the previous element's result when every grid slot equals the previous element's
    /// (the blank tail of a whole-column criteria range is walked once).
    pub(crate) reuse_equal_runs: bool,
}

impl LiftOptions {
    /// Every caller of T-1784's driver: the *IF(S) functions, AGGREGATE, IFS, SWITCH and the
    /// lookups all reuse equal runs, as `lift_with_cost` did.
    pub(crate) const REUSE_EQUAL_RUNS: Self = Self {
        reuse_equal_runs: true,
    };
}

/// A per-element kernel: the scalar function over the argument values of one element.
pub(crate) type Kernel<'k> = dyn FnMut(&mut Model, &[CalcResult]) -> CalcResult + 'k;

/// Whether every grid slot at (`row`, `column`) equals the one at `previous`.
fn same_as(args: &[LiftedArg], row: usize, column: usize, previous: (usize, usize)) -> bool {
    args.iter()
        .all(|arg| arg.element(row, column) == arg.element(previous.0, previous.1))
}

/// Elements [`Model::lift_values`] computes, in row-major order: all of them, or one per run of
/// equal consecutive elements when `reuse` is set (the rest reuse the previous result).
fn count_walks(args: &[LiftedArg], rows: usize, columns: usize, reuse: bool) -> u64 {
    let mut walks: u64 = 0;
    let mut previous: Option<(usize, usize)> = None;
    for row in 0..rows {
        for column in 0..columns {
            let same = reuse && previous.is_some_and(|p| same_as(args, row, column, p));
            if !same {
                walks = walks.saturating_add(1);
            }
            previous = Some((row, column));
        }
    }
    walks
}

impl Model {
    /// Runs `kernel` once per element when any of `args` is an array or a multi-cell range: the
    /// result is an array of the arguments broadcast against each other, as the element-wise
    /// operators do (a row and a column give a grid; a position past a shorter array is an
    /// `#N/A` element). Otherwise `kernel` runs once with the arguments as they are. Evidence:
    /// probes/ifs-array-criteria, poi/70005-countifs.xlsx (docs/excel-semantics/criteria.md
    /// "*IF(S) lift array criteria").
    ///
    /// A range across sheets is `#VALUE!`. The output is bounded by `check_array_size`, and
    /// the call is `#ERROR!` before any element runs when `cost` exceeds
    /// `limits::MAX_LIFTED_CELL_VISITS`.
    pub(crate) fn lift_values(
        &mut self,
        args: Vec<CalcResult>,
        cost: LiftCost<'_>,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        if !args.iter().any(LiftedArg::is_grid) {
            return kernel(self, &args);
        }
        let mut lifted = Vec::with_capacity(args.len());
        for arg in args {
            lifted.push(match arg {
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    match self.evaluate_range(left, right, cell) {
                        Ok(grid) => LiftedArg::Grid(grid),
                        Err(error) => return error,
                    }
                }
                CalcResult::Array(grid) => LiftedArg::Grid(grid),
                scalar => LiftedArg::Scalar(scalar),
            });
        }
        let (rows, columns) = lifted
            .iter()
            .map(LiftedArg::dims)
            .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        // Refuse before running anything when the elements would visit too many cells.
        let walks = count_walks(&lifted, rows, columns, options.reuse_equal_runs);
        let per_element = match cost {
            LiftCost::Cells(cells) => cells,
            LiftCost::Walk(node) => self.walk_cost(node, cell),
        };
        let arg_count = u64::try_from(lifted.len()).unwrap_or(u64::MAX);
        let visits = per_element.saturating_mul(arg_count).saturating_mul(walks);
        if let Some(error) = too_many_visits(visits, cell) {
            return error;
        }
        // One buffer for the whole call: scalar slots are written once, grid slots per element.
        let mut values: Vec<CalcResult> = lifted
            .iter()
            .map(|arg| match arg {
                LiftedArg::Scalar(value) => value.clone(),
                LiftedArg::Grid(_) => CalcResult::EmptyCell,
            })
            .collect();
        // The previous position and the last computed result.
        let mut previous: Option<((usize, usize), ArrayNode)> = None;
        let mut output = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut out_row = Vec::with_capacity(columns);
            for column in 0..columns {
                let reuse = options.reuse_equal_runs
                    && previous
                        .as_ref()
                        .is_some_and(|(position, _)| same_as(&lifted, row, column, *position));
                let value = match previous.as_mut() {
                    // Equal slots are transitive, so comparing with the previous position is the
                    // same as comparing with the last computed one; only the position moves.
                    Some((position, value)) if reuse => {
                        *position = (row, column);
                        value.clone()
                    }
                    _ => {
                        for (slot, arg) in values.iter_mut().zip(&lifted) {
                            if let Some(node) = arg.element(row, column) {
                                *slot = array_node_to_calc_result(node, cell);
                            }
                        }
                        let value = calc_result_to_array_node(kernel(self, &values));
                        previous = Some(((row, column), value.clone()));
                        value
                    }
                };
                out_row.push(value);
            }
            output.push(out_row);
        }
        CalcResult::Array(output)
    }
}

#[cfg(test)]
mod tests {
    use super::{count_walks, LiftedArg};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;

    fn column(values: &[f64]) -> LiftedArg {
        LiftedArg::Grid(values.iter().map(|v| vec![ArrayNode::Number(*v)]).collect())
    }

    #[test]
    fn count_walks_counts_runs_only_when_reusing() {
        let args = [column(&[1.0, 1.0, 2.0, 2.0, 1.0])];
        assert_eq!(count_walks(&args, 5, 1, true), 3);
        assert_eq!(count_walks(&args, 5, 1, false), 5);
    }

    #[test]
    fn count_walks_compares_every_grid_and_ignores_scalars() {
        // A column of runs against a row of two: each row's second element differs from its
        // first, and the first differs from the previous row's last.
        let row = LiftedArg::Grid(vec![vec![ArrayNode::Number(1.0), ArrayNode::Number(2.0)]]);
        let args = [
            column(&[7.0, 7.0]),
            row,
            LiftedArg::Scalar(CalcResult::Number(3.0)),
        ];
        assert_eq!(count_walks(&args, 2, 2, true), 4);
        // Past the shorter array every slot is #N/A, so the tail is one run.
        let args = [column(&[1.0, 2.0]), column(&[5.0, 5.0, 5.0, 5.0])];
        assert_eq!(count_walks(&args, 4, 1, true), 3);
    }

    #[test]
    fn is_grid_is_an_array_or_a_range() {
        assert!(LiftedArg::is_grid(&CalcResult::Array(vec![])));
        assert!(!LiftedArg::is_grid(&CalcResult::Number(1.0)));
        assert!(!LiftedArg::is_grid(&CalcResult::EmptyCell));
    }
}
