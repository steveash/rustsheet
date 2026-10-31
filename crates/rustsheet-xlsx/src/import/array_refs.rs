//! The bound on the cells an xlsx sheet's array formulas may cover, checked before the importer
//! maps each covered cell to its anchor (T-3437, directive 5).

use rustsheet_engine::LAST_ROW;

use crate::error::XlsxError;

/// Takes the cells of the array formula `range`, (`row1`, `column1`, `row2`, `column2`), out
/// of `budget` before `array_cell` gets one entry for each: a `ref` is a few bytes
/// that can name the whole sheet, and this map is filled before the model checks anything
/// (T-3437, directive 5). An array of more cells than the engine builds (a whole column) is
/// refused, as the load check would refuse it later (T-2896).
pub(super) fn claim_array_cells(
    budget: &mut u64,
    range: &str,
    (row1, column1, row2, column2): (i32, i32, i32, i32),
) -> Result<(), XlsxError> {
    let side = |first: i32, last: i32| u64::try_from(i64::from(last) - i64::from(first) + 1);
    let cells = match (side(row1, row2), side(column1, column2)) {
        (Ok(rows), Ok(columns)) => rows.saturating_mul(columns),
        _ => return Err(XlsxError::Xml(format!("Invalid range: {range}"))),
    };
    if cells > u64::from(LAST_ROW.unsigned_abs()) {
        return Err(XlsxError::Xml(format!(
            "The array formula at {range} spills over more than {LAST_ROW} cells"
        )));
    }
    *budget = budget.checked_sub(cells).ok_or_else(|| {
        XlsxError::Xml(format!(
            "The array formulas up to {range} spill over more cells than the sheet stores"
        ))
    })?;
    Ok(())
}

/// The cells the array formulas of a worksheet part of `part_bytes` may cover in total, beyond
/// one whole column (the most one array formula may cover): one per 16 bytes of the part. A file
/// stores every cell its arrays cover, as a `<c>` element in a `<row>`, which past the first
/// rows takes more than 16 bytes a cell (`<c r="A10000"/>` is 16 alone), so such a file fits,
/// and a few-byte `ref` naming far more cells than the part holds is refused before each is put
/// in the map (T-3437).
pub(super) fn array_cell_budget(part_bytes: u64) -> u64 {
    (part_bytes / 16).saturating_add(u64::from(LAST_ROW.unsigned_abs()))
}
