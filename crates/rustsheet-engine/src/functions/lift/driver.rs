//! The lifting driver's broadcasting core (SPEC-0011 "The driver"), moved from
//! `statistical/if_ifs.rs` by T-1784 with no result change; T-1770 adds the spec's cost model,
//! the in-place single-grid path and the text budget.

use super::arg_buf::ArgBuf;
use super::kinds::{ArgKinds, ArgValue, CallResult, KernelWork, ValueWork};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::keyed::{tally_run, tally_visits};
use crate::area_budget::{text_units, visit_units};
use crate::arithmetic::{bcast_get, MISSING};
use crate::cast::{calc_result_to_array_node, MAX_NUMBER_TEXT_LEN};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::text::util::within_text_budget;
use crate::functions::util::{array_element_value, top_left_of_array};
use crate::limits::{check_array_cells, too_many_visits};
use crate::reference_operators::{RefArg, RefElement, RefGrid};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::ArrayNode,
    expressions::token::Error,
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

    /// The argument as a value again: a grid as its array, a scalar as itself. A function that
    /// reads an argument before the lift (to give its error first) hands it to
    /// [`Model::lift_values`] this way.
    pub(crate) fn into_value(self) -> CalcResult {
        match self {
            LiftedArg::Scalar(value) => value,
            LiftedArg::Grid(grid) => CalcResult::Array(grid),
        }
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

    /// Every argument value read by [`LiftedArg::read`], or the first one's error.
    pub(crate) fn read_all(
        model: &mut Model,
        args: Vec<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Vec<LiftedArg>, CalcResult> {
        let mut lifted = Vec::with_capacity(args.len());
        for arg in args {
            lifted.push(LiftedArg::read(model, arg, cell)?);
        }
        Ok(lifted)
    }

    /// The grid slot at output position (`row`, `column`), `None` for a scalar: a dimension of
    /// length 1 stretches, a position past a shorter array is `#N/A`
    /// (probes/ifs-array-criteria H11, H33-H37). A nested GROUPBY cell is read as its top-left
    /// (`ArrayNode::scalar`, SPEC-0018 rule 4).
    pub(crate) fn element(&self, row: usize, column: usize) -> Option<&ArrayNode> {
        match self {
            LiftedArg::Scalar(_) => None,
            LiftedArg::Grid(grid) => {
                Some(bcast_get(grid, row, column).unwrap_or(&MISSING).scalar())
            }
        }
    }
}

/// The rows and columns of a grid.
pub(crate) fn grid_dims(grid: &[Vec<ArrayNode>]) -> (usize, usize) {
    (grid.len(), grid.first().map_or(0, Vec::len))
}

/// The broadcast shape of `lifted`, or `#ERROR!` past the cell bound (`check_array_cells`). Its
/// arrays are built already, so one wider or taller than the grid lifts like any other:
/// `ERROR.TYPE(MAKEARRAY(1,16385,..))` is a 1 x 16385 array, `#SPILL!` in a cell
/// (probes/t2579-array-value-edges B26; arrays.md "Arrays past the grid"; T-2600).
pub(crate) fn broadcast_shape(
    lifted: &[LiftedArg],
    cell: CellReferenceIndex,
) -> Result<(usize, usize), CalcResult> {
    let (rows, columns) = lifted
        .iter()
        .map(LiftedArg::dims)
        .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
    match check_array_cells(rows, columns) {
        Some((error, message)) => Err(CalcResult::new_error(error, cell, message)),
        None => Ok((rows, columns)),
    }
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

/// The kernel units the computed `elements` of a `rows` x `columns` broadcast are charged before
/// any runs: `elements` times the table weight, or for [`KernelWork::ByValue`] the sum of each
/// element's charge by its argument values. That sum runs over every element, reused or not, so
/// it never charges less than the elements computed; it is one pass over the grid, O(elements).
///
/// A [`ValueWork::CouponWalks`] kernel reads each of its arguments as a number before any walk,
/// so a text is charged as what `reads_as` (the kernel's own `Model::cast_number`) reads it as: a
/// number as that number, any other text as an error, `#VALUE!` before any walk (T-2879). Each
/// reading parses at most [`MAX_NUMBER_TEXT_LEN`] characters, and a scalar's is read once.
pub(super) fn kernel_units(
    work: KernelWork,
    lifted: &[LiftedArg],
    rows: usize,
    columns: usize,
    elements: u64,
    reads_as: &dyn Fn(&str) -> Option<f64>,
) -> u64 {
    let KernelWork::ByValue(work) = work else {
        return elements.saturating_mul(work.weight());
    };
    let reads_numbers = matches!(work, ValueWork::CouponWalks { .. });
    let read_text = |text: &str| {
        // Only a shortcut for `number_text_too_long` in `cast_number`: past
        // `4 * MAX_NUMBER_TEXT_LEN` bytes a text has more characters than a number text may, so
        // it is not parsed (or its characters counted) once per element.
        let number = if text.len() > 4 * MAX_NUMBER_TEXT_LEN {
            None
        } else {
            reads_as(text)
        };
        number.map_or(ArgValue::Error, ArgValue::Number)
    };
    // A scalar's value is read once; a grid's per element.
    let slots: Vec<Result<ArgValue, &Vec<Vec<ArrayNode>>>> = lifted
        .iter()
        .map(|arg| match arg {
            LiftedArg::Scalar(CalcResult::String(text)) if reads_numbers => Ok(read_text(text)),
            LiftedArg::Scalar(value) => Ok(ArgValue::of(value)),
            LiftedArg::Grid(grid) => Err(grid),
        })
        .collect();
    let mut units: u64 = 0;
    for row in 0..rows {
        for column in 0..columns {
            let value = |index: usize| {
                slots.get(index).map(|slot| match slot {
                    Ok(value) => *value,
                    Err(grid) => match bcast_get(grid, row, column).unwrap_or(&MISSING).scalar() {
                        ArrayNode::String(text) if reads_numbers => read_text(text),
                        node => ArgValue::of_node(node),
                    },
                })
            };
            units = units.saturating_add(work.units(value));
        }
    }
    units
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
    /// What one unlifted call returns: an `Array` entry's element is its call's top-left value.
    pub(crate) result: CallResult,
    /// Whether a [`CallResult::ArrayFirstColumn`] entry's emptied elements are `#VALUE!` instead: a
    /// SORTBY call with a second `by_array`, `sort_order` pair (`lift_call_with` sets it).
    pub(crate) later_error: bool,
}

impl LiftOptions {
    /// The options of a table entry's call: every element runs, at the entry's work and result.
    pub(crate) fn for_entry(kinds: &ArgKinds) -> Self {
        Self {
            reuse: Reuse::Never,
            work: kinds.work,
            result: kinds.result,
            later_error: false,
        }
    }

    /// AGGREGATE, IFS, SWITCH and the lookups reuse equal runs, as T-1784's `lift_with_cost` did.
    pub(crate) const REUSE_EQUAL_RUNS: Self = Self {
        reuse: Reuse::EqualRuns,
        work: KernelWork::Constant,
        result: CallResult::Value,
        later_error: false,
    };

    /// The *IF(S) functions, whose kernel only reads the criteria and the ranges resolved before
    /// the lift: any repeated element reuses its first result (T-0408).
    pub(crate) const REUSE_ANY_EQUAL: Self = Self {
        reuse: Reuse::AnyEqual,
        work: KernelWork::Constant,
        result: CallResult::Value,
        later_error: false,
    };

    /// TREND's and GROWTH's `const` (T-1726): as [`LiftOptions::REUSE_ANY_EQUAL`], the kernel
    /// reading only its argument and the data read before the lift, and an element is the top-left
    /// value of its call's array.
    pub(crate) const REUSE_ANY_EQUAL_TOP_LEFT: Self = Self {
        reuse: Reuse::AnyEqual,
        work: KernelWork::Constant,
        result: CallResult::Array,
        later_error: false,
    };

    /// Every element runs, and the caller's [`LiftCost`] already counts its work: the lifted
    /// text functions (TEXTSPLIT, TEXTBEFORE, TEXTAFTER, REGEX*; T-1272).
    pub(crate) const EVERY_ELEMENT: Self = Self {
        reuse: Reuse::Never,
        work: KernelWork::Constant,
        result: CallResult::Value,
        later_error: false,
    };
}

/// What the element loops share: the options, the visits charged so far (the up-front cost,
/// then each element's output charge), the calling cell and what the elements keep.
struct Run<'r> {
    options: &'r LiftOptions,
    visits: &'r mut u64,
    cell: CellReferenceIndex,
    later: Later,
}

/// Which element results are kept (SPEC-0011 "Array-valued functions" rule 2).
enum Later {
    /// Every element keeps its result.
    Kept,
    /// [`CallResult::ArrayFirstColumn`] over entries that are not all equal: an error result stays,
    /// the first other one is kept (`placed` once it is), and every later one is empty.
    EmptyAfterFirst { placed: bool },
    /// [`CallResult::ArrayFirstColumn`] over entries that are all equal: every element runs, then the
    /// non-error ones past the first column are emptied ([`empty_past_first_column`]).
    EmptyPastFirstColumn,
}

impl Run<'_> {
    /// One element's result as the output holds it ([`element_node`]), empty when [`Later`]
    /// says so.
    fn element(&mut self, value: CalcResult) -> Result<ArrayNode, CalcResult> {
        let node = element_node(value, self.options, self.visits, self.cell)?;
        Ok(match &mut self.later {
            Later::EmptyAfterFirst { placed } if !matches!(node, ArrayNode::Error(_)) => {
                if !std::mem::replace(placed, true) {
                    node
                } else if self.options.later_error {
                    ArrayNode::Error(Error::VALUE)
                } else {
                    ArrayNode::Empty
                }
            }
            _ => node,
        })
    }
}

/// Whether every element's grid slots equal the first element's: then a
/// [`CallResult::ArrayFirstColumn`] entry keeps every element (`UNIQUE(D1:D3,{FALSE;FALSE})` is "x"
/// twice, probes/m7-lift-take-drop-sortby row 35). One slot comparison per element and argument,
/// within the work the elements are charged.
fn all_elements_equal(args: &[LiftedArg], rows: usize, columns: usize) -> bool {
    (0..rows).all(|row| (0..columns).all(|column| same_as(args, row, column, (0, 0))))
}

/// A [`CallResult::ArrayFirstColumn`] call over entries that are all equal and span several columns
/// keeps its call's top-left only down the first column: the other columns' elements are empty,
/// errors excepted (`UNIQUE(D1:D3,FALSE,{FALSE,FALSE;FALSE,FALSE})` is x, x in column one and
/// empty in column two, `by_col` or not; a 1x3 `{FALSE,FALSE,FALSE}` is 1, empty, empty;
/// probes/m7-unique-lift-corners, T-2348; docs/excel-semantics/arrays.md "UNIQUE's and SORTBY's
/// lifted elements"). With `later_error` (SORTBY's second pair) they are `#VALUE!` instead.
fn empty_past_first_column(result: CalcResult, later_error: bool) -> CalcResult {
    // SORTBY with a second pair makes them `#VALUE!` instead, as for entries that differ
    // (`SORTBY(A1:A3,C1:C3,{1,1},A1:A3,1)`: 1, `#VALUE!`; probes/sortby-lifted-element-corners
    // row 16, T-3027).
    let emptied = if later_error {
        ArrayNode::Error(Error::VALUE)
    } else {
        ArrayNode::Empty
    };
    match result {
        CalcResult::Array(mut rows) => {
            for row in &mut rows {
                for node in row.iter_mut().skip(1) {
                    if !matches!(node, ArrayNode::Error(_)) {
                        *node = emptied.clone();
                    }
                }
            }
            CalcResult::Array(rows)
        }
        other => other,
    }
}

/// A per-element kernel: the scalar function over the argument values of one element.
pub(crate) type Kernel<'k> = dyn FnMut(&mut Model, &[CalcResult]) -> CalcResult + 'k;

/// A number kernel's shortcut (T-1932): its result for arguments that are all numbers, one per
/// argument written, exactly what its [`Kernel`] returns for them as `CalcResult::Number`s, but
/// without building, dispatching on and dropping those values. A per-row `=DAY(A1)` and each
/// element of `DAY(A1:A1000000)` cost a third more through the kernel than the math macros did.
pub(crate) type NumbersKernel<'k> = dyn Fn(&[f64]) -> Result<f64, Error> + 'k;

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
                ArrayNode::Lambda(id) => id.hash(state),
                // Never: `LiftedArg::element` reads a nested cell through `ArrayNode::scalar`.
                ArrayNode::Nested(_) => {}
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

/// One element's result as the output holds it (SPEC-0011 "Array-valued functions" rules 2 and
/// 3): charged to `visits` by `work` (the cells of an array result, 1 otherwise), `#ERROR!` past
/// the cap; then an array result of an `Array` entry is its top-left value (`INDEX` of a lifted
/// SEQUENCE element has one row and one column, probes/m7-argument-modes K9), and anything else
/// converts as a `Value` entry's does. The element's own array is dropped here, so a lifted call
/// holds at most one at a time.
fn element_node(
    value: CalcResult,
    options: &LiftOptions,
    visits: &mut u64,
    cell: CellReferenceIndex,
) -> Result<ArrayNode, CalcResult> {
    if element_is_value(options) {
        return Ok(calc_result_to_array_node(value));
    }
    let cells = match &value {
        CalcResult::Array(grid) => {
            let (rows, columns) = grid_dims(grid);
            u64::try_from(rows.saturating_mul(columns)).unwrap_or(u64::MAX)
        }
        _ => 1,
    };
    *visits = visits.saturating_add(options.work.output_charge(cells));
    if let Some(error) = too_many_visits(*visits, cell) {
        return Err(error);
    }
    Ok(match value {
        CalcResult::Array(grid) if options.result.is_array() => {
            calc_result_to_array_node(top_left_of_array(&grid, cell))
        }
        value => calc_result_to_array_node(value),
    })
}

/// Whether an element's result is its value as it is: no output charge and no top-left, every
/// entry before T-1834.
fn element_is_value(options: &LiftOptions) -> bool {
    options.result == CallResult::Value && !matches!(options.work, KernelWork::Output { .. })
}

/// The arguments of a [`NumbersKernel`] for `values` but the grid's at `grid_index` (left 0),
/// `None` when one of them is not a number.
fn scalar_numbers(values: &[CalcResult], grid_index: usize) -> Option<ArgBuf<f64>> {
    let mut numbers = ArgBuf::filled(values.len(), || 0.0);
    for (index, (slot, value)) in numbers.as_mut_slice().iter_mut().zip(values).enumerate() {
        match value {
            CalcResult::Number(number) => *slot = *number,
            _ if index == grid_index => {}
            _ => return None,
        }
    }
    Some(numbers)
}

/// An array element as a scalar value, by value: a string element moves, not copied. A LAMBDA
/// element reaches the kernel as that LAMBDA, so each kernel's rule for a LAMBDA value applies
/// (`ISERROR(SCAN(..))` over LAMBDAs is FALSE per element, probes/lambda-arrays B8), and a nested
/// GROUPBY cell as its top-left (`ArrayNode::scalar`, SPEC-0018 rule 4).
fn array_node_into_calc_result(node: ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::String(text) => CalcResult::String(text),
        other => array_element_value(other.scalar(), cell),
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
        match LiftedArg::read_all(self, args, cell) {
            Ok(lifted) => self.lift_grids(lifted, cost, options, cell, kernel, None),
            Err(error) => error,
        }
    }

    /// The broadcast over arguments already read, at least one of them a grid (SPEC-0011
    /// `lift_values` steps 3-7): the output bound (`check_array_cells`, `#ERROR!` past it) and the
    /// work bound (`cost`, `too_many_visits`) are checked before any element is computed; then
    /// the elements run in row-major order, in place when one grid is the only lifted argument.
    /// There, an element whose arguments are all numbers runs `numbers` instead of `kernel`. A
    /// 1x1 grid lifts like any other, also for an array-valued entry: `SEQUENCE({3})` is one
    /// element, 1, where `SEQUENCE(A3:A3)` never gets here, since a one-cell range is its value
    /// (probes/m7-lift-take-drop-sortby Sheet1 rows 12 and 38-43; T-2298).
    pub(super) fn lift_grids(
        &mut self,
        lifted: Vec<LiftedArg>,
        cost: LiftCost,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
        numbers: Option<&NumbersKernel<'_>>,
    ) -> CalcResult {
        self.lift_grids_after(lifted, cost, options, cell, (kernel, numbers), 0)
            .0
    }

    /// [`Self::lift_grids`] continuing a lift that charged `prior` visits up front: the per-call
    /// cap is checked on `prior` and this lift's visits together, and only this lift's are
    /// charged to the formula's work budget. Also returns this lift's up-front visits (0 when it
    /// fails before charging them). `kernels` is the kernel and its number shortcut.
    pub(super) fn lift_grids_after(
        &mut self,
        lifted: Vec<LiftedArg>,
        cost: LiftCost,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernels: (&mut Kernel<'_>, Option<&NumbersKernel<'_>>),
        prior: u64,
    ) -> (CalcResult, u64) {
        let (kernel, numbers) = kernels;
        let (rows, columns) = match broadcast_shape(&lifted, cell) {
            Ok(shape) => shape,
            Err(error) => return (error, 0),
        };
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
        let reads_as = |text: &str| self.cast_number(text);
        let units = kernel_units(options.work, &lifted, rows, columns, elements, &reads_as);
        let visits = match cost {
            LiftCost::Unit => units,
            LiftCost::Cells(cells) => cells.saturating_mul(units),
            LiftCost::Prepared(cells) => cells.saturating_add(units),
        };
        let up_front = prior.saturating_add(visits);
        if let Some(error) = too_many_visits(up_front, cell)
            .or_else(|| self.spend_work(visit_units(visits), cell).err())
        {
            return (error, 0);
        }
        tally_visits(visits);
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
        let first_only = options.result == CallResult::ArrayFirstColumn;
        let later = if first_only && !all_elements_equal(&lifted, rows, columns) {
            Later::EmptyAfterFirst { placed: false }
        } else if options.result == CallResult::ArrayFirstColumn && columns > 1 {
            Later::EmptyPastFirstColumn
        } else {
            Later::Kept
        };
        let past_first_column = matches!(later, Later::EmptyPastFirstColumn);
        let charged = visits;
        let mut visits = up_front;
        let run = Run {
            options,
            visits: &mut visits,
            cell,
            later,
        };
        let result = match in_place {
            Some(index) => self.lift_in_place(lifted, index, run, kernel, numbers),
            None => self.lift_broadcast(&lifted, rows, columns, &plan, run, kernel),
        };
        // The elements' output charges, past the up-front cost, go to the formula's budget too.
        if let Err(error) = self.spend_work(visit_units(visits.saturating_sub(up_front)), cell) {
            return (error, charged);
        }
        if past_first_column {
            return (
                empty_past_first_column(result, options.later_error),
                charged,
            );
        }
        (result, charged)
    }

    /// The lift of a call whose result is a reference (SPEC-0014 "Producers"): `kernel` runs
    /// once per element of the broadcast of `lifted`, as [`Self::lift_grids`] runs it, and each
    /// result is kept as a [`RefElement`]: its reference, or its error (any other value is
    /// `#VALUE!`). The shape and the work (one visit per element) are checked before any element
    /// runs. A 1x1 grid gives one element too: `TAKE(A1:B3,{2})` is one element, A1:B2
    /// (probes/m7-lift-take-drop-sortby Sheet1 F13, I13:K13; T-2298).
    pub(crate) fn lift_references(
        &mut self,
        lifted: Vec<LiftedArg>,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> RefArg {
        let (rows, columns) = match broadcast_shape(&lifted, cell) {
            Ok(shape) => shape,
            Err(error) => return RefArg::Value(error),
        };
        let mut values: Vec<CalcResult> = lifted
            .iter()
            .map(|arg| match arg {
                LiftedArg::Scalar(value) => value.clone(),
                LiftedArg::Grid(grid) => top_left_of_array(grid, cell),
            })
            .collect();
        let elements = rows.saturating_mul(columns);
        if let Some(error) = self.spend_visits(u64::try_from(elements).unwrap_or(u64::MAX), cell) {
            return RefArg::Value(error);
        }
        let mut output = Vec::with_capacity(elements);
        for row in 0..rows {
            for column in 0..columns {
                for (slot, arg) in values.iter_mut().zip(&lifted) {
                    if let Some(node) = arg.element(row, column) {
                        *slot = array_element_value(node, cell);
                    }
                }
                output.push(match kernel(self, &values) {
                    CalcResult::Range { left, right } => {
                        RefElement::Reference(Range { left, right })
                    }
                    CalcResult::Error { error, .. } => RefElement::Error(error),
                    _ => RefElement::Error(Error::VALUE),
                });
            }
        }
        match RefGrid::new(rows, columns, output) {
            Some(grid) => RefArg::Grid(grid),
            None => RefArg::Value(CalcResult::new_error(
                Error::ERROR,
                cell,
                "Empty array of references".to_string(),
            )),
        }
    }

    /// One grid at `grid_index`, the rest scalars: each element is moved out of the grid, given
    /// to the kernel, and its result written back into the same slot, so the output is the input
    /// grid (T-1078: at most one allocation per output element, none for a number). With a
    /// `numbers` shortcut and every scalar a number, a number element runs the shortcut on the
    /// numbers alone (T-1932); the shortcut's elements charge nothing beyond the up-front cost and
    /// hold no text, so only an entry whose elements are charged nothing takes it.
    fn lift_in_place(
        &mut self,
        lifted: Vec<LiftedArg>,
        grid_index: usize,
        mut run: Run<'_>,
        kernel: &mut Kernel<'_>,
        numbers: Option<&NumbersKernel<'_>>,
    ) -> CalcResult {
        let cell = run.cell;
        // The values live inline for up to `INLINE_ARGS` arguments, as in the read phase.
        let mut values = ArgBuf::filled(lifted.len(), || CalcResult::EmptyCell);
        let mut output = Vec::new();
        for (slot, arg) in values.as_mut_slice().iter_mut().zip(lifted) {
            match arg {
                LiftedArg::Scalar(value) => *slot = value,
                LiftedArg::Grid(grid) => output = grid,
            }
        }
        let mut shortcut = numbers
            .filter(|_| element_is_value(run.options))
            .and_then(|numbers| Some((numbers, scalar_numbers(values.as_slice(), grid_index)?)));
        let mut text_bytes = 0;
        for row in &mut output {
            for node in row {
                if let (Some((numbers, args)), ArrayNode::Number(number)) = (&mut shortcut, &*node)
                {
                    if let Some(slot) = args.as_mut_slice().get_mut(grid_index) {
                        *slot = *number;
                    }
                    *node = match numbers(args.as_slice()) {
                        Ok(number) => ArrayNode::Number(number),
                        Err(error) => ArrayNode::Error(error),
                    };
                    continue;
                }
                let element = std::mem::replace(node, ArrayNode::Empty);
                if let Some(slot) = values.as_mut_slice().get_mut(grid_index) {
                    *slot = array_node_into_calc_result(element, cell);
                }
                tally_run();
                *node = match run.element(kernel(self, values.as_slice())) {
                    Ok(node) => node,
                    Err(error) => return error,
                };
                if let Some(error) = over_text_budget(node, &mut text_bytes, cell) {
                    return error;
                }
            }
        }
        // The texts' bytes go to the formula's work budget (SPEC-0024 "text values").
        if let Err(error) = self.spend_work(text_units(text_bytes), cell) {
            return error;
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
        mut run: Run<'_>,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult {
        let cell = run.cell;
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
                                *slot = array_element_value(node, cell);
                            }
                        }
                        tally_run();
                        let value = match run.element(kernel(self, &values)) {
                            Ok(value) => value,
                            Err(error) => return error,
                        };
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
        // The texts' bytes go to the formula's work budget (SPEC-0024 "text values").
        if let Err(error) = self.spend_work(text_units(text_bytes), cell) {
            return error;
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
