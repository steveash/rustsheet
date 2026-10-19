//! `lift_call` and its read phase (SPEC-0011 "The driver"): each argument is read once, by its
//! kind in the table, and the kernel runs once (nothing lifted) or once per element.

use super::arg_buf::ArgBuf;
use super::driver::{Kernel, LiftCost, LiftOptions, LiftedArg, Reuse};
use super::kinds::{ArgKind, ArgKinds, KernelWork};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::is_multi_sheet_span;
use crate::functions::Function;
use crate::limits::{check_range_size, too_many_visits, MAX_LIFTED_CELL_VISITS};
use crate::{
    calc_result::CalcResult,
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

/// The arguments of one call as the read phase leaves them: every value in an inline buffer,
/// and the grids (arrays and multi-cell ranges read into cells) apart with their positions, so
/// an unlifted call hands the kernel a borrowed slice and builds no [`LiftedArg`].
pub(super) struct ReadArgs {
    pub(super) values: ArgBuf<CalcResult>,
    pub(super) grids: Vec<(usize, Vec<Vec<ArrayNode>>)>,
}

impl ReadArgs {
    /// The arguments as the broadcast takes them: a grid in its own position, the rest scalars.
    fn into_lifted(self) -> Vec<LiftedArg> {
        let mut lifted: Vec<LiftedArg> = self.values.into_iter().map(LiftedArg::Scalar).collect();
        for (index, grid) in self.grids {
            if let Some(slot) = lifted.get_mut(index) {
                *slot = LiftedArg::Grid(grid);
            }
        }
        lifted
    }
}

/// The cells of `grid`, saturating.
fn grid_cells(grid: &[Vec<ArrayNode>]) -> u64 {
    let cells = grid.len().saturating_mul(grid.first().map_or(0, Vec::len));
    u64::try_from(cells).unwrap_or(u64::MAX)
}

/// The `#VALUE!` of a multi-cell range in a `LiftArrays` position, or of a range across sheets
/// there. Its message is empty: an element error allocates nothing.
fn refused_range(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, String::new())
}

/// One `Lift` argument, for the callers that read their first argument apart from the rest
/// (the offline functions).
const LIFT_ONE: ArgKinds = ArgKinds::fixed(1, &[ArgKind::Lift], KernelWork::Constant);

impl Model {
    /// A `LiftArrays` argument: a call that returns a reference when its source is one (INDEX,
    /// XLOOKUP, TAKE, DROP, TRIMRANGE) is read as that reference, so a multi-cell one is refused
    /// like a written range: `EOMONTH(..,TAKE(Data!B9:E12,-2,-1))` is `#VALUE!`
    /// (gen-m9/b58b6d7cd39d195f A9), as `EDATE(INDEX(A1:A60,0,1),1)` is (-b K3).
    /// A 3-D range there is `#VALUE!`, where a `Lift` argument reads it as `#REF!`
    /// (probes/scalar-lift-open-b W2-W11, -d W17-W29; T-1777), also a one-cell one,
    /// `EDATE(Sheet1:Sheet2!A1,1)` (scalar-lift-open-e W1-W9). A DROP, TRIMRANGE or XLOOKUP that
    /// returns several cells is refused like TAKE, INDEX and IF (scalar-lift-open-e W10-W15, W20,
    /// W22), and one that returns one cell is its value (W16-W19, W21; T-1801).
    fn evaluate_as_reference(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        match node.ungrouped() {
            Node::FunctionKind { kind, args } => {
                if let Some(result) = self.evaluate_reference_form(kind, args, cell) {
                    return result;
                }
            }
            node if is_multi_sheet_span(node) => return refused_range(cell),
            _ => {}
        }
        self.evaluate_node_in_context(node, cell)
    }

    /// Reads one `Lift` argument: an array or a multi-cell range is a grid, anything else (a
    /// one-cell range included) its value. The read is bounded like `lift_call`'s.
    pub(crate) fn read_lift_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LiftedArg, CalcResult> {
        let read = self.read_lift_args(
            &LIFT_ONE,
            std::slice::from_ref(node),
            MAX_LIFTED_CELL_VISITS,
            cell,
        )?;
        let mut lifted = read.into_lifted();
        Ok(lifted
            .pop()
            .unwrap_or(LiftedArg::Scalar(CalcResult::EmptyArg)))
    }

    /// Reads `args` by `kind.arg_kinds()`, then runs `kernel` once (nothing lifted) or once per
    /// element of the broadcast arguments. A wrong argument count is the count error, before
    /// anything is evaluated.
    pub(crate) fn lift_call(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        self.lift_call_costed(kind, args, LiftCost::Unit, cell, kernel)
    }

    /// [`Model::lift_call`] for a kernel whose elements cost `cost` rather than one unit each:
    /// `LiftCost::Prepared` for one that prepares a whole argument once (LARGE sorts its data,
    /// T-1772), `LiftCost::Cells` for one whose every element walks it.
    pub(crate) fn lift_call_costed(
        &mut self,
        kind: Function,
        args: &[Node],
        cost: LiftCost,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let Some(kinds) = kind.arg_kinds() else {
            // A programming error, not an input condition: `lift_call` is only called with the
            // constant `Function` of the `fn_x` that calls it (SPEC-0011 "Error handling").
            debug_assert!(false, "{kind:?} has no lifting table entry");
            let values: Vec<CalcResult> = args
                .iter()
                .map(|arg| self.evaluate_node_in_context(arg, cell))
                .collect();
            return kernel(self, &values);
        };
        self.lift_call_with(kinds, args, cost, cell, kernel)
    }

    /// [`Model::lift_call`] for the kinds `kinds` and the element cost `cost`. Not generic and
    /// not inlined, so the read and broadcast code is one frame of fixed size, not copied into
    /// every caller's (T-1110).
    #[inline(never)]
    pub(super) fn lift_call_with(
        &mut self,
        kinds: &ArgKinds,
        args: &[Node],
        cost: LiftCost,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        if !kinds.accepts(args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let options = LiftOptions {
            reuse: Reuse::Never,
            work: kinds.work,
            result: kinds.result,
        };
        // One- and two-argument calls (most of the math functions) read into locals: building,
        // moving and dropping a whole `ArgBuf` cost about as much as a scalar kernel, and was
        // most of the per-call overhead over the old math macros (T-1773 bench).
        // Each argument is read by `read_lift_arg_with`, as in `read_lift_args`: a change to how
        // the arguments are read or counted against the cap goes there, not in these arms.
        let lifted = match args {
            [only] => match self.read_lift_arg_at(kinds, 0, only, &mut 0, cell) {
                Ok(LiftedArg::Scalar(value)) => return kernel(self, std::slice::from_ref(&value)),
                Ok(grid) => vec![grid],
                Err(error) => return error,
            },
            [first, second] => {
                let mut input_cells = 0;
                let read = self
                    .read_lift_arg_at(kinds, 0, first, &mut input_cells, cell)
                    .and_then(|first| {
                        let second =
                            self.read_lift_arg_at(kinds, 1, second, &mut input_cells, cell)?;
                        Ok((first, second))
                    });
                match read {
                    Ok((LiftedArg::Scalar(first), LiftedArg::Scalar(second))) => {
                        return kernel(self, &[first, second]);
                    }
                    Ok((first, second)) => vec![first, second],
                    Err(error) => return error,
                }
            }
            _ => {
                let read = match self.read_lift_args(kinds, args, MAX_LIFTED_CELL_VISITS, cell) {
                    Ok(read) => read,
                    Err(error) => return error,
                };
                if read.grids.is_empty() {
                    // Nothing lifts: exactly the scalar call, on the values read.
                    return kernel(self, read.values.as_slice());
                }
                read.into_lifted()
            }
        };
        self.lift_grids(lifted, cost, &options, cell, kernel)
    }

    /// [`Model::lift_call_costed`] for a call whose `ReferenceByCaller` argument at `index` was
    /// not a reference: its caller read it into `grid` (one error per element), which broadcasts
    /// with the lifted arguments as a grid in that position. `grid` counts toward the read cap.
    pub(crate) fn lift_call_beside(
        &mut self,
        kind: Function,
        args: &[Node],
        (index, grid): (usize, Vec<Vec<ArrayNode>>),
        cost: LiftCost,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let Some(kinds) = kind.arg_kinds() else {
            // As in `lift_call_costed`: only called with the caller's own constant `Function`.
            debug_assert!(false, "{kind:?} has no lifting table entry");
            return CalcResult::new_args_number_error(cell);
        };
        debug_assert_eq!(kinds.kind(index), Some(ArgKind::ReferenceByCaller));
        if !kinds.accepts(args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let input_cap = MAX_LIFTED_CELL_VISITS.saturating_sub(grid_cells(&grid));
        let mut read = match self.read_lift_args(kinds, args, input_cap, cell) {
            Ok(read) => read,
            Err(error) => return error,
        };
        read.grids.push((index, grid));
        let options = LiftOptions {
            reuse: Reuse::Never,
            work: kinds.work,
            result: kinds.result,
        };
        self.lift_grids(read.into_lifted(), cost, &options, cell, kernel)
    }

    /// [`Model::read_lift_arg_with`] for argument `index` of a call read by `kinds`, under
    /// `limits::MAX_LIFTED_CELL_VISITS`.
    fn read_lift_arg_at(
        &mut self,
        kinds: &ArgKinds,
        index: usize,
        node: &Node,
        input_cells: &mut u64,
        cell: CellReferenceIndex,
    ) -> Result<LiftedArg, CalcResult> {
        let kind = kinds.kind(index).unwrap_or(ArgKind::LiftArrays);
        self.read_lift_arg_with(kind, node, input_cells, MAX_LIFTED_CELL_VISITS, cell)
    }

    /// The read phase for calls of three or more arguments (`lift_call_with` reads shorter ones
    /// into locals): each argument once, by its kind (SPEC-0011's read table). An array is a
    /// grid; a multi-cell range is a grid for `Lift` (read through `evaluate_range`, bounded
    /// before it is read) and `#VALUE!` for `LiftArrays`, with no intersection; a one-cell range
    /// is its cell's value. The grids' cells add up, and the call is `#ERROR!` before the next
    /// range is read once the total would pass `input_cap` (`limits::MAX_LIFTED_CELL_VISITS`).
    #[inline(never)]
    pub(super) fn read_lift_args(
        &mut self,
        kinds: &ArgKinds,
        args: &[Node],
        input_cap: u64,
        cell: CellReferenceIndex,
    ) -> Result<ReadArgs, CalcResult> {
        let mut values = ArgBuf::filled(args.len(), || CalcResult::EmptyArg);
        let mut grids = Vec::new();
        let mut input_cells: u64 = 0;
        for (index, (node, slot)) in args.iter().zip(values.as_mut_slice()).enumerate() {
            let kind = kinds.kind(index).unwrap_or(ArgKind::LiftArrays);
            match self.read_lift_arg_with(kind, node, &mut input_cells, input_cap, cell)? {
                LiftedArg::Scalar(value) => *slot = value,
                LiftedArg::Grid(grid) => grids.push((index, grid)),
            }
        }
        Ok(ReadArgs { values, grids })
    }

    /// One argument of kind `kind`, read once: a grid, or a value (`EmptyArg` for a
    /// `WholeByCaller` argument, which its caller reads). A grid's cells add to `input_cells`, and
    /// the call is `#ERROR!` before a range is read once the total would pass `input_cap`.
    fn read_lift_arg_with(
        &mut self,
        kind: ArgKind,
        node: &Node,
        input_cells: &mut u64,
        input_cap: u64,
        cell: CellReferenceIndex,
    ) -> Result<LiftedArg, CalcResult> {
        // The `#ERROR!` of `too_many_visits`, past the cap.
        let over_cap = |cells: u64| (cells > input_cap).then(|| too_many_visits(u64::MAX, cell));
        let value = match kind {
            ArgKind::WholeByCaller | ArgKind::ReferenceByCaller => {
                return Ok(LiftedArg::Scalar(CalcResult::EmptyArg))
            }
            ArgKind::Whole => {
                return Ok(LiftedArg::Scalar(
                    self.evaluate_node_with_reference(node, cell),
                ))
            }
            ArgKind::LiftArrays => self.evaluate_as_reference(node, cell),
            ArgKind::Lift => self.evaluate_node_in_context(node, cell),
            // A reference stays one, also a one-cell `E1`: the criteria table is read whole
            // (`DSUM(A1:C3,"Price",E1)` is a one-cell table, not the cell's value).
            ArgKind::LiftArraysRangeWhole => self.evaluate_node_with_reference(node, cell),
        };
        let grid = match value {
            CalcResult::Array(grid) => grid,
            range @ CalcResult::Range { .. } if kind == ArgKind::LiftArraysRangeWhole => {
                return Ok(LiftedArg::Scalar(range));
            }
            // A 3-D range is `#REF!` in a `Lift` argument and `#VALUE!` in a `LiftArrays`
            // one (probes/scalar-lift-open-b W2-W11, -d W17-W29; T-1777).
            CalcResult::Range { left, right } if left.sheet != right.sheet => {
                return Ok(LiftedArg::Scalar(if kind == ArgKind::Lift {
                    CalcResult::new_error(Error::REF, cell, String::new())
                } else {
                    refused_range(cell)
                }));
            }
            CalcResult::Range { left, right } if left == right => {
                return Ok(LiftedArg::Scalar(self.evaluate_cell(left)));
            }
            CalcResult::Range { .. } if kind == ArgKind::LiftArrays => {
                return Ok(LiftedArg::Scalar(refused_range(cell)));
            }
            CalcResult::Range { left, right } => {
                if let Some((error, message)) = check_range_size(&left, &right) {
                    return Err(CalcResult::new_error(error, cell, message));
                }
                let rows = u64::try_from(right.row - left.row + 1).unwrap_or(0);
                let columns = u64::try_from(right.column - left.column + 1).unwrap_or(0);
                let total = input_cells.saturating_add(rows.saturating_mul(columns));
                if let Some(Some(error)) = over_cap(total) {
                    return Err(error);
                }
                self.evaluate_range(left, right, cell)?
            }
            scalar => return Ok(LiftedArg::Scalar(scalar)),
        };
        // An array is already built; it counts toward the total all the same.
        *input_cells = input_cells.saturating_add(grid_cells(&grid));
        if let Some(Some(error)) = over_cap(*input_cells) {
            return Err(error);
        }
        Ok(LiftedArg::Grid(grid))
    }
}
