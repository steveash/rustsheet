use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;

// Arbitrary limit to prevent creating huge arrays that would crash the program. Excel builds larger
// arrays (probes/resource-limits C12, C14): DEVIATION, see docs/excel-semantics/arrays.md ("Array results above 1,000,000 cells").
// A better way would be to look at the memory available
const MAX_SIZE: usize = 1_000_000;

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
            format!("rows * columns exceeds matrix size limit of {}", MAX_SIZE),
        ));
    }
    None
}

/// [`check_array_size`] for the array a range `left:right` would become when a function lifts
/// over it element by element (IS*, NOT): Excel spills any range that fits the grid, so this is
/// the shared allocation bound (DEVIATION, docs/excel-semantics/arrays.md).
pub fn check_range_size(
    left: &CellReferenceIndex,
    right: &CellReferenceIndex,
) -> Option<(Error, String)> {
    let rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
    let columns = usize::try_from(right.column - left.column + 1).unwrap_or(0);
    check_array_size(rows, columns)
}
