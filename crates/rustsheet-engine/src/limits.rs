use crate::calc_result::CalcResult;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;

// Limit on the cells of any array the engine builds, so an untrusted formula cannot exhaust
// memory (CLAUDE.md directive 5). One whole column (2^20 cells), so a whole-column range lifts
// (`SUM(ABS(A:A))`, probes/lifted-range-size). Excel builds larger arrays
// (probes/lifted-range-rows C1): DEVIATION, see docs/excel-semantics/arrays.md ("Array results
// above 1,048,576 cells").
const MAX_SIZE: usize = LAST_ROW as usize;

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
