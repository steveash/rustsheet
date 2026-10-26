use crate::calc_result::CalcResult;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;

// Limit on the cells of any array the engine builds, so an untrusted formula cannot exhaust
// memory (CLAUDE.md directive 5). One whole column (2^20 cells), so a whole-column range lifts
// (`SUM(ABS(A:A))`, probes/lifted-range-size). Excel builds larger arrays
// (probes/lifted-range-rows C1): DEVIATION, see docs/excel-semantics/arrays.md ("Array results
// above 1,048,576 cells").
pub(crate) const MAX_SIZE: usize = LAST_ROW as usize;

pub fn check_array_size(rows: usize, columns: usize) -> Option<(Error, String)> {
    if columns > LAST_COLUMN as usize {
        return Some((
            Error::VALUE,
            format!("columns exceeds sheet limit of {LAST_COLUMN}"),
        ));
    }

    if rows > LAST_ROW as usize {
        return Some((
            Error::VALUE,
            format!("rows exceeds sheet limit of {LAST_ROW}"),
        ));
    }

    check_array_cells(rows, columns)
}

/// The cell-count half of [`check_array_size`], for a result Excel builds even when it is wider or
/// taller than the grid (TEXTSPLIT's 16,385-column row, T-1781): `#ERROR!` above 2^20 cells.
pub fn check_array_cells(rows: usize, columns: usize) -> Option<(Error, String)> {
    // If rows * columns exceeds MAX_SIZE, return #ERROR!
    // Otherwise this would be an easy way to crash the program by trying to create a huge array.
    // NB: This is a limitation of IronCalc
    // Saturating: callers need not cap rows and columns at the grid first, and even capped the
    // product (up to 2^34) does not fit a 32-bit `usize` (wasm32).
    if rows.saturating_mul(columns) > MAX_SIZE {
        return Some((
            Error::ERROR,
            format!("rows * columns exceeds matrix size limit of {MAX_SIZE}"),
        ));
    }
    None
}

/// Whether an array of `rows` x `columns` anchored at `anchor` runs past the last row or column.
pub(crate) fn spills_past_grid(anchor: CellReferenceIndex, rows: usize, columns: usize) -> bool {
    let last = |start: i32, count: usize| i64::from(start) + count as i64 - 1;
    last(anchor.row, rows) > i64::from(LAST_ROW)
        || last(anchor.column, columns) > i64::from(LAST_COLUMN)
}

/// The `#SPILL!` of a dynamic array anchored at `anchor` that runs past the grid.
pub(crate) fn spill_past_grid_error(anchor: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::SPILL,
        anchor,
        "Spill would exceed worksheet bounds".to_string(),
    )
}

/// The `#SPILL!` of a dynamic array of `rows` x `columns` anchored at `anchor` that is over the
/// cell bound of [`check_array_cells`] and runs past the grid: Excel builds such an array and
/// cannot spill it, where the engine would refuse to build it (`#ERROR!`), so the shape alone gives
/// Excel's answer (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are
/// #ERROR!"; T-0480, T-0483). `None` within the bound, where the array is built and the spill
/// itself decides, and for a shape that fits the grid (the decision-0014 deviation).
pub(crate) fn unspillable_shape(
    anchor: CellReferenceIndex,
    rows: usize,
    columns: usize,
) -> Option<CalcResult> {
    check_array_cells(rows, columns)?;
    spills_past_grid(anchor, rows, columns).then(|| spill_past_grid_error(anchor))
}

/// 2^31, the least size Excel reads as no longer a size: EXPAND's rows or columns (`#VALUE!`) and
/// WRAPROWS/WRAPCOLS's `wrap_count` (`#NUM!`) refuse it (arrays.md "Stacked results past the grid",
/// "WRAPROWS and WRAPCOLS with a huge `wrap_count`").
pub(crate) const SIZE_LIMIT_EXCLUSIVE: f64 = 2_147_483_648.0;

/// 2^31 - 1, the least count Excel's integer-count functions refuse with `#NUM!` (2^31 - 2 is the
/// largest they take): PERMUT's number (probes/stat-overflow-arguments E5, B8), COMBIN's number and
/// COMBINA's `number + number_chosen - 1` (probes/t2199-text-length-and-combin-cuts A20-A37, B20-B34),
/// and BINOM.DIST's, BINOMDIST's and BINOM.DIST.RANGE's trials (probes-numeric/t2645-binom-trials-limit;
/// statistics.md "BINOM.DIST is #NUM! from 2^31 - 1 trials").
pub(crate) const COUNT_LIMIT_EXCLUSIVE: f64 = 2_147_483_647.0;

/// The bound of a MAKEARRAY or SEQUENCE result, which Excel builds past the grid:
/// `COLUMNS(MAKEARRAY(1,20000,..))` is 20000, `ROWS(MAKEARRAY(1048577,1,..))` 1048577 and
/// `COLUMNS(SEQUENCE(1,16385))` 16385 (probes/t2579-array-value-edges B20, B23, B24). Within the
/// cell bound of [`check_array_cells`] any shape is built. Above it, `#VALUE!` as in Excel for
/// columns from `columns_refused` (SEQUENCE passes one past the grid: `SEQUENCE(3,123456789)`,
/// gen-m9/f2bd5f380ec002bf A177; MAKEARRAY passes [`SIZE_LIMIT_EXCLUSIVE`], since Excel builds
/// `MAKEARRAY(1,1048577,..)`, probes/t2452-makearray-cols-1048577, and refuses
/// `MAKEARRAY(3,1E15,..)`, gen-m7/b2078b5d702c691a A2553) and for rows of 2^31 or more
/// (`MAKEARRAY(1E15,4,..)`, gen-m7/8c4570978f18f4c2 A2736; the threshold is EXPAND's,
/// [`SIZE_LIMIT_EXCLUSIVE`]); any other shape is the bound's `#ERROR!`, where Excel builds
/// MAKEARRAY's 1048577 rows or columns (DEVIATION, decision 0014; arrays.md "Arrays past the
/// grid"; T-2600, T-2452). SEQUENCE checks its rows against the grid first.
pub fn check_built_array_size(
    rows: usize,
    columns: usize,
    columns_refused: f64,
) -> Option<(Error, String)> {
    let refused = check_array_cells(rows, columns)?;
    if columns as f64 >= columns_refused {
        return Some((
            Error::VALUE,
            format!("columns must be less than {columns_refused}"),
        ));
    }
    if rows as f64 >= SIZE_LIMIT_EXCLUSIVE {
        return Some((Error::VALUE, "rows must be less than 2^31".to_string()));
    }
    Some(refused)
}

/// The bound of a stacked result (HSTACK, VSTACK): more rows than the grid is `#NUM!`, while
/// columns past the grid are allowed, under the cell-count limit of [`check_array_cells`]:
/// `ROWS(VSTACK(A:A,B1))` is `#NUM!` and `COLUMNS(HSTACK(A1:XFD1,A1))` is 16385
/// (probes/scalar-lift-open-c W1-W10, -c2; arrays.md "Stacked results past the grid"; T-1803).
pub fn check_stack_size(rows: usize, columns: usize) -> Option<(Error, String)> {
    if rows > LAST_ROW as usize {
        return Some((
            Error::NUM,
            format!("rows exceeds sheet limit of {LAST_ROW}"),
        ));
    }
    check_array_cells(rows, columns)
}

/// [`check_array_size`] for the array a range `left:right` would become when it is lifted
/// element by element (`Model::evaluate_range`): Excel lifts any range that fits the grid, so this
/// is the shared allocation bound (DEVIATION, docs/excel-semantics/arrays.md).
pub fn check_range_size(
    left: &CellReferenceIndex,
    right: &CellReferenceIndex,
) -> Option<(Error, String)> {
    let rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
    let columns = usize::try_from(right.column - left.column + 1).unwrap_or(0);
    check_array_size(rows, columns)
}

/// Most cells all the per-element walks of one lifted *IF(S) call may visit, so a crafted formula
/// (`COUNTIFS(A:A,SEQUENCE(1048576))` over a filled column) cannot run for hours (CLAUDE.md
/// directive 5). Excel has no such limit: DEVIATION, docs/excel-semantics/criteria.md
/// ("*IF(S) lift array criteria").
pub(crate) const MAX_LIFTED_CELL_VISITS: u64 = 1 << 27;

/// `#ERROR!` when `visits` cells are more than one call may visit ([`MAX_LIFTED_CELL_VISITS`]):
/// the lifted *IF(S) and AGGREGATE calls, and the two-parameter LAMBDAs GROUPBY and PIVOTBY call
/// with a whole relative set (SPEC-0006 Design step 3). Excel has no such limit: DEVIATION.
pub(crate) fn too_many_visits(visits: u64, cell: CellReferenceIndex) -> Option<CalcResult> {
    (visits > MAX_LIFTED_CELL_VISITS)
        .then(|| CalcResult::new_error(Error::ERROR, cell, "Too many cells to visit".to_string()))
}

/// Most levels a nested GROUPBY or PIVOTBY cell may have (SPEC-0018 "Bound"): `NestedArray::new`
/// refuses a deeper tree (`#ERROR!`), so `Drop` and equality, which recurse, stay this shallow
/// whatever the evaluator's recursion guard allows. Excel has no such limit: DEVIATION
/// (decision 0014; docs/excel-semantics/arrays.md "Nested GROUPBY and PIVOTBY cells: bounds").
pub(crate) const MAX_NESTED_DEPTH: usize = 64;

/// Most cells of nested arrays one evaluation may read the contents of (INDEX of a nested cell),
/// as many as [`MAX_LIFTED_CELL_VISITS`]: copies of a nested cell share it, so a fan-out
/// (`VSTACK(r,r,..)`) costs nothing until its copies are read, and this bounds the reads.
/// Excel has no such limit: DEVIATION (decision 0014; docs/excel-semantics/arrays.md).
pub(crate) const MAX_NESTED_WALK_CELLS: usize = 1 << 27;

/// Most cells of nested arrays one evaluation may keep, summed over its GROUPBY and PIVOTBY
/// calls, as many as a LAMBDA's captures (`MAX_CAPTURED_CELLS`): each call is capped at
/// [`MAX_SIZE`], and this bounds a loop of them (`REDUCE(..,LAMBDA(a,i,VSTACK(a,GROUPBY(..))))`),
/// whose results no array size check sees. Excel has no such limit: DEVIATION (decision 0014).
pub(crate) const MAX_NESTED_KEPT_CELLS: usize = 1 << 22;
