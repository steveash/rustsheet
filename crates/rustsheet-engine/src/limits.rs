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

    // If rows * columns exceeds MAX_SIZE, return #ERROR!
    // Otherwise this would be an easy way to crash the program by trying to create a huge array.
    // NB: This is a limitation of IronCalc
    if rows * columns > MAX_SIZE {
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
