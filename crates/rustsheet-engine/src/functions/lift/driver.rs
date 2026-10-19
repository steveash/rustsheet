//! The lifting driver's broadcasting core (SPEC-0011 "The driver"), moved from
//! `statistical/if_ifs.rs` by T-1784 with no result change; T-1770 adds the spec's cost model,
//! the in-place single-grid path and the text budget.

use super::arg_buf::ArgBuf;
use super::kinds::KernelWork;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::arithmetic::{bcast_get, MISSING};
use crate::cast::calc_result_to_array_node;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::text::util::within_text_budget;
use crate::functions::util::array_node_to_calc_result;
use crate::limits::{check_array_size, too_many_visits};
use crate::{
    calc_result::CalcResult, expressions::parser::ArrayNode, expressions::token::Error,
    model::Model,
};

/// One lifted argument as the driver holds it.
pub(crate) enum LiftedArg {
    /// One value for every element: a scalar, a one-cell reference's value, an error, a refused
    /// range (`#VALUE!`), an omitted argument (`EmptyArg`), or a whole argument.
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

    pub(crate) fn dims(&self) -> (usize, usize) {
        match self {
            LiftedArg::Scalar(_) => (1, 1),
            LiftedArg::Grid(grid) => grid_dims(grid),
        }
    }

    /// An argument value as the driver holds it: a range read once into its cells (`#VALUE!`
    /// across sheets), an array as its grid, anything else a scalar.
    pub(crate) fn read(
        model: &mut Model,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<LiftedArg, CalcResult> {
        match value {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                model.evaluate_range(left, right, cell).map(LiftedArg::Grid)
            }
            CalcResult::Array(grid) => Ok(LiftedArg::Grid(grid)),
            scalar => Ok(LiftedArg::Scalar(scalar)),
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

/// The rows and columns of a grid.
pub(crate) fn grid_dims(grid: &[Vec<ArrayNode>]) -> (usize, usize) {
    (grid.len(), grid.first().map_or(0, Vec::len))
}

/// What one element costs, for the cell-visit cap (`limits::MAX_LIFTED_CELL_VISITS`). The total
/// is checked before the first element runs, with saturating arithmetic. "Elements" are the
/// elements computed: all of them, or those [`LiftOptions::reuse`] does not reuse. The table's
/// [`KernelWork::Weighted`] weight multiplies the per-element term.
pub(crate) enum LiftCost {
    /// `KernelWork::Constant` and no whole data: elements × 1.
    Unit,
    /// Every element visits this many cells: elements × cells.
    Cells(u64),
    /// The kernel prepares its whole data once (LARGE sorts it): cells + elements.
    Prepared(u64),
}

/// Which elements reuse an earlier element's result instead of running the kernel.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reuse {
    /// Every element runs.
    Never,
    /// An element whose every grid slot equals the previous element's reuses its result (the
    /// blank tail of a whole-column criteria range is walked once).
    EqualRuns,
    /// An element whose every grid slot equals any earlier element's reuses that result: the
    /// occurrence-count idiom `COUNTIF(A2:A10000,A2:A10000)` walks once per distinct value
    /// (T-0408). Only for a kernel whose result depends on nothing but its argument values.
    AnyEqual,
}

/// How [`Model::lift_values`] runs the elements.
pub(crate) struct LiftOptions {
    /// Which elements reuse an earlier result. `ArrayNode`'s equality treats 0 and -0 as equal,
    /// so a 0 element may reuse a -0 element's result: harmless for the callers that reuse
    /// (*IF(S) criteria, IFS, SWITCH and lookups match 0 and -0 alike). [`Reuse::Never`] for the
    /// scalar kernels of `lift_call`.
    pub(crate) reuse: Reuse,
    /// What one kernel call may cost: the table's entry for `lift_call`.
    pub(crate) work: KernelWork,
}

impl LiftOptions {
    /// AGGREGATE, IFS, SWITCH and the lookups reuse equal runs, as T-1784's `lift_with_cost` did.
    pub(crate) const REUSE_EQUAL_RUNS: Self = Self {
        reuse: Reuse::EqualRuns,
        work: KernelWork::Constant,
    };

    /// The *IF(S) functions, whose kernel only reads the criteria and the ranges resolved before
    /// the lift: any repeated element reuses its first result (T-0408).
    pub(crate) const REUSE_ANY_EQUAL: Self = Self {
        reuse: Reuse::AnyEqual,
        work: KernelWork::Constant,
    };

    /// Every element runs, and the caller's [`LiftCost`] already counts its work: the lifted
    /// text functions (TEXTSPLIT, TEXTBEFORE, TEXTAFTER, REGEX*; T-1272).
    pub(crate) const EVERY_ELEMENT: Self = Self {
        reuse: Reuse::Never,
        work: KernelWork::Constant,
    };
}

/// A per-element kernel: the scalar function over the argument values of one element.
pub(crate) type Kernel<'k> = dyn FnMut(&mut Model, &[CalcResult]) -> CalcResult + 'k;

/// Whether every grid slot at (`row`, `column`) equals the one at `previous`.
fn same_as(args: &[LiftedArg], row: usize, column: usize, previous: (usize, usize)) -> bool {
    args.iter()
        .all(|arg| arg.element(row, column) == arg.element(previous.0, previous.1))
}

/// Elements [`Model::lift_values`] computes under [`Reuse::Never`] or [`Reuse::EqualRuns`], in
/// row-major order: all of them, or one per run of equal consecutive elements (the rest reuse the
/// previous result).
fn count_walks(args: &[LiftedArg], rows: usize, columns: usize, reuse: Reuse) -> u64 {
    if reuse == Reuse::Never {
        return u64::try_from(rows.saturating_mul(columns)).unwrap_or(u64::MAX);
    }
    let mut walks: u64 = 0;
    let mut previous: Option<(usize, usize)> = None;
    for row in 0..rows {
        for column in 0..columns {
            let same = previous.is_some_and(|p| same_as(args, row, column, p));
            if !same {
                walks = walks.saturating_add(1);
            }
            previous = Some((row, column));
        }
    }
    walks
}

/// One element's grid slots, as a hash key: equal when [`same_as`] is, without copying a slot.
struct ElementKey<'a> {
    args: &'a [LiftedArg],
    row: usize,
    column: usize,
}

impl PartialEq for ElementKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        same_as(self.args, self.row, self.column, (other.row, other.column))
    }
}

impl Eq for ElementKey<'_> {}

impl Hash for ElementKey<'_> {
    /// Hashes what `ArrayNode`'s equality compares: 0 and -0 alike, a text by its lossy view
    /// (equal texts have equal views; the rare lossy collision is told apart by `eq`).
    fn hash<H: Hasher>(&self, state: &mut H) {
        for arg in self.args {
            let Some(node) = arg.element(self.row, self.column) else {
                continue;
            };
            std::mem::discriminant(node).hash(state);
            match node {
                ArrayNode::Boolean(value) => value.hash(state),
                ArrayNode::Number(value) => {
                    (if *value == 0.0 { 0 } else { value.to_bits() }).hash(state);
                }
                ArrayNode::String(text) => {
                    let view: &str = text;
                    view.hash(state);
                }
                ArrayNode::Error(error) => std::mem::discriminant(error).hash(state),
                ArrayNode::Empty => {}
            }
        }
    }
}

/// How [`Model::lift_broadcast`] reuses results, worked out from [`LiftOptions::reuse`].
enum ReusePlan {
    Never,
    EqualRuns,
    /// For each element in row-major order, the row-major index of the first element equal to it
    /// ([`first_equal_elements`]).
    FirstEqual(Vec<usize>),
}

/// For each element in row-major order, the row-major index of the first element whose every
/// grid slot equals its own ([`Reuse::AnyEqual`]): itself when it is computed, an earlier one
/// whose result it reuses otherwise. Also returns how many elements are computed. An element
/// equal to the previous one takes its entry without hashing, so a run (the blank tail of a
/// whole-column criteria range) costs one slot comparison per element, as under
/// [`Reuse::EqualRuns`].
fn first_equal_elements(args: &[LiftedArg], rows: usize, columns: usize) -> (Vec<usize>, u64) {
    let mut first: HashMap<ElementKey<'_>, usize> = HashMap::new();
    let mut plan: Vec<usize> = Vec::with_capacity(rows.saturating_mul(columns));
    let mut previous: Option<(usize, usize)> = None;
    for row in 0..rows {
        for column in 0..columns {
            let index = plan.len();
            let entry = match (previous, plan.last()) {
                (Some(p), Some(&last)) if same_as(args, row, column, p) => last,
                _ => *first
                    .entry(ElementKey { args, row, column })
                    .or_insert(index),
            };
            plan.push(entry);
            previous = Some((row, column));
        }
    }
    let computed = u64::try_from(first.len()).unwrap_or(u64::MAX);
    (plan, computed)
}

/// The `#VALUE!` of a call whose element texts exceed the broadcast text budget, or `None`.
fn over_text_budget(
    value: &ArrayNode,
    total: &mut usize,
    cell: CellReferenceIndex,
) -> Option<CalcResult> {
    match value {
        ArrayNode::String(text) if !within_text_budget(total, text.len()) => {
            Some(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Total text result is too large".to_string(),
            ))
        }
        _ => None,
    }
}

/// An array element as a scalar value, by value: a string element moves, not copied.
fn array_node_into_calc_result(node: ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::String(text) => CalcResult::String(text),
        other => array_node_to_calc_result(&other, cell),
    }
}

impl Model {
    /// Runs `kernel` once per element when any of `args` is an array or a multi-cell range: the
    /// result is an array of the arguments broadcast against each other, as the element-wise
    /// operators do (a row and a column give a grid; a position past a shorter array is an
    /// `#N/A` element). Otherwise `kernel` runs once with the arguments as they are. Evidence:
    /// probes/ifs-array-criteria, poi/70005-countifs.xlsx (docs/excel-semantics/criteria.md
    /// "*IF(S) lift array criteria").
    ///
    /// A range here lifts as a grid, a one-cell range too (the *IF(S) contract); `lift_call`
    /// reads its arguments by the table instead. A range across sheets is `#VALUE!`.
    pub(crate) fn lift_values(
        &mut self,
        args: Vec<CalcResult>,
        cost: LiftCost,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        if !args.iter().any(LiftedArg::is_grid) {
            return kernel(self, &args);
        }
        let mut lifted = Vec::with_capacity(args.len());
        for arg in args {
            match LiftedArg::read(self, arg, cell) {
                Ok(arg) => lifted.push(arg),
                Err(error) => return error,
            }
        }
        self.lift_grids(lifted, cost, options, cell, kernel)
    }

    /// The broadcast over arguments already read, at least one of them a grid (SPEC-0011
    /// `lift_values` steps 3-7): the output bound (`check_array_size`, `#ERROR!` past it) and the
    /// work bound (`cost`, `too_many_visits`) are checked before any element is computed; then
    /// the elements run in row-major order, in place when one grid is the only lifted argument.
    pub(super) fn lift_grids(
        &mut self,
        lifted: Vec<LiftedArg>,
        cost: LiftCost,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let (rows, columns) = lifted
            .iter()
            .map(LiftedArg::dims)
            .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        // Refuse before running anything when the elements would cost too much.
        let (plan, elements) = match options.reuse {
            Reuse::AnyEqual => {
                let (first, elements) = first_equal_elements(&lifted, rows, columns);
                (ReusePlan::FirstEqual(first), elements)
            }
            reuse => {
                let plan = if reuse == Reuse::EqualRuns {
                    ReusePlan::EqualRuns
                } else {
                    ReusePlan::Never
                };
                (plan, count_walks(&lifted, rows, columns, reuse))
            }
        };
        let weight = options.work.weight();
        let visits = match cost {
            LiftCost::Unit => elements.saturating_mul(weight),
            LiftCost::Cells(cells) => cells.saturating_mul(weight).saturating_mul(elements),
            LiftCost::Prepared(cells) => cells.saturating_add(elements.saturating_mul(weight)),
        };
        if let Some(error) = too_many_visits(visits, cell) {
            return error;
        }
        let mut grids = lifted
            .iter()
            .enumerate()
            .filter_map(|(index, arg)| match arg {
                LiftedArg::Grid(grid) => Some((index, grid)),
                LiftedArg::Scalar(_) => None,
            });
        // One grid with the output's exact shape (every row as wide), and nothing to reuse.
        let in_place = match (grids.next(), grids.next()) {
            (Some((index, grid)), None) => (options.reuse == Reuse::Never
                && grid.len() == rows
                && grid.iter().all(|row| row.len() == columns))
            .then_some(index),
            _ => None,
        };
        match in_place {
            Some(index) => self.lift_in_place(lifted, index, cell, kernel),
            None => self.lift_broadcast(&lifted, rows, columns, &plan, cell, kernel),
        }
    }

    /// One grid at `grid_index`, the rest scalars: each element is moved out of the grid, given
    /// to the kernel, and its result written back into the same slot, so the output is the input
    /// grid (T-1078: at most one allocation per output element, none for a number).
    fn lift_in_place(
        &mut self,
        lifted: Vec<LiftedArg>,
        grid_index: usize,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        // The values live inline for up to `INLINE_ARGS` arguments, as in the read phase.
        let mut values = ArgBuf::filled(lifted.len(), || CalcResult::EmptyCell);
        let mut output = Vec::new();
        for (slot, arg) in values.as_mut_slice().iter_mut().zip(lifted) {
            match arg {
                LiftedArg::Scalar(value) => *slot = value,
                LiftedArg::Grid(grid) => output = grid,
            }
        }
        let mut text_bytes = 0;
        for row in &mut output {
            for node in row {
                let element = std::mem::replace(node, ArrayNode::Empty);
                if let Some(slot) = values.as_mut_slice().get_mut(grid_index) {
                    *slot = array_node_into_calc_result(element, cell);
                }
                *node = calc_result_to_array_node(kernel(self, values.as_slice()));
                if let Some(error) = over_text_budget(node, &mut text_bytes, cell) {
                    return error;
                }
            }
        }
        CalcResult::Array(output)
    }

    /// The general broadcast: several grids (or one that a dimension of length 1 stretches), one
    /// value buffer for the whole call (scalar slots are written once, grid slots per element),
    /// and an earlier result reused as `plan` says: the previous one for a run of equal elements,
    /// or the first equal element's.
    fn lift_broadcast(
        &mut self,
        lifted: &[LiftedArg],
        rows: usize,
        columns: usize,
        plan: &ReusePlan,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let mut values: Vec<CalcResult> = lifted
            .iter()
            .map(|arg| match arg {
                LiftedArg::Scalar(value) => value.clone(),
                LiftedArg::Grid(_) => CalcResult::EmptyCell,
            })
            .collect();
        // The previous position and the last computed result.
        let mut previous: Option<((usize, usize), ArrayNode)> = None;
        let mut text_bytes = 0;
        let mut output: Vec<Vec<ArrayNode>> = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut out_row: Vec<ArrayNode> = Vec::with_capacity(columns);
            for column in 0..columns {
                // The first equal element's result, already in the output.
                let first_equal = match plan {
                    ReusePlan::FirstEqual(first) => first
                        .get(row * columns + column)
                        .map(|first| (first / columns, first % columns))
                        .filter(|&first| first != (row, column))
                        .and_then(|(first_row, first_column)| {
                            if first_row == row {
                                out_row.get(first_column)
                            } else {
                                output.get(first_row).and_then(|r| r.get(first_column))
                            }
                        })
                        .cloned(),
                    ReusePlan::Never | ReusePlan::EqualRuns => None,
                };
                let reuse = matches!(plan, ReusePlan::EqualRuns)
                    && previous
                        .as_ref()
                        .is_some_and(|(position, _)| same_as(lifted, row, column, *position));
                let value = match (first_equal, previous.as_mut()) {
                    (Some(value), _) => value,
                    // Equal slots are transitive, so comparing with the previous position is the
                    // same as comparing with the last computed one; only the position moves.
                    (None, Some((position, value))) if reuse => {
                        *position = (row, column);
                        value.clone()
                    }
                    (None, _) => {
                        for (slot, arg) in values.iter_mut().zip(lifted) {
                            if let Some(node) = arg.element(row, column) {
                                *slot = array_node_to_calc_result(node, cell);
                            }
                        }
                        let value = calc_result_to_array_node(kernel(self, &values));
                        if matches!(plan, ReusePlan::EqualRuns) {
                            previous = Some(((row, column), value.clone()));
                        }
                        value
                    }
                };
                if let Some(error) = over_text_budget(&value, &mut text_bytes, cell) {
                    return error;
                }
                out_row.push(value);
            }
            output.push(out_row);
        }
        CalcResult::Array(output)
    }
}

#[cfg(test)]
mod tests {
    use super::{count_walks, first_equal_elements, LiftedArg, Reuse};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;

    fn column(values: &[f64]) -> LiftedArg {
        LiftedArg::Grid(values.iter().map(|v| vec![ArrayNode::Number(*v)]).collect())
    }

    #[test]
    fn count_walks_counts_runs_only_when_reusing() {
        let args = [column(&[1.0, 1.0, 2.0, 2.0, 1.0])];
        assert_eq!(count_walks(&args, 5, 1, Reuse::EqualRuns), 3);
        assert_eq!(count_walks(&args, 5, 1, Reuse::Never), 5);
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
        assert_eq!(count_walks(&args, 2, 2, Reuse::EqualRuns), 4);
        // Past the shorter array every slot is #N/A, so the tail is one run.
        let args = [column(&[1.0, 2.0]), column(&[5.0, 5.0, 5.0, 5.0])];
        assert_eq!(count_walks(&args, 4, 1, Reuse::EqualRuns), 3);
    }

    #[test]
    fn first_equal_elements_points_repeats_at_their_first_occurrence() {
        // 0 and -0 are one element, as `ArrayNode`'s equality has them; a text and an equal
        // number are not.
        let texts = LiftedArg::Grid(vec![
            vec![ArrayNode::String("a".into())],
            vec![ArrayNode::Number(1.0)],
            vec![ArrayNode::String("a".into())],
            vec![ArrayNode::Number(0.0)],
            vec![ArrayNode::Number(-0.0)],
            vec![ArrayNode::Number(1.0)],
        ]);
        let (plan, computed) = first_equal_elements(&[texts], 6, 1);
        assert_eq!(plan, vec![0, 1, 0, 3, 3, 1]);
        assert_eq!(computed, 3);
        // Every grid slot takes part; a scalar does not.
        let args = [
            column(&[1.0, 1.0, 1.0]),
            column(&[2.0, 3.0, 2.0]),
            LiftedArg::Scalar(CalcResult::Number(9.0)),
        ];
        assert_eq!(first_equal_elements(&args, 3, 1), (vec![0, 1, 0], 2));
    }

    #[test]
    fn is_grid_is_an_array_or_a_range() {
        assert!(LiftedArg::is_grid(&CalcResult::Array(vec![])));
        assert!(!LiftedArg::is_grid(&CalcResult::Number(1.0)));
        assert!(!LiftedArg::is_grid(&CalcResult::EmptyCell));
    }
}
