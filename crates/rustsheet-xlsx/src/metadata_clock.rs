//! The IronCalc `METADATA`/`NOW` convention: Excel-authored test workbooks carry the time they
//! were computed at, which the harness pins the workbook clock to (SPEC-0008).

use rustsheet_engine::{cell::CellValue, clock::Clock, Model};

use crate::util::get_metadata_sheet_index;

// Excel serial date of the Unix epoch (1970-01-01)
const UNIX_EPOCH_SERIAL: f64 = 25569.0;
const MILLISECONDS_PER_DAY: f64 = 86_400_000.0;

/// Looks in the METADATA sheet for a row with "NOW" in column A and an Excel
/// serial date in column B (typically `=NOW()` with the value Excel cached on
/// save) and returns it as milliseconds since the Unix epoch.
pub fn get_metadata_timestamp(model: &Model) -> Option<i64> {
    let sheet_index = get_metadata_sheet_index(model)?;
    for row in 1..=32 {
        if let Ok(CellValue::String(label)) = model.get_cell_value_by_index(sheet_index, row, 1) {
            if label == "NOW" {
                if let Ok(CellValue::Number(serial)) =
                    model.get_cell_value_by_index(sheet_index, row, 2)
                {
                    let milliseconds = (serial - UNIX_EPOCH_SERIAL) * MILLISECONDS_PER_DAY;
                    return Some(milliseconds.round() as i64);
                }
                return None;
            }
        }
    }
    None
}

/// Pins `model`'s clock to the timestamp stored in its METADATA sheet, so that NOW, TODAY and
/// the other readers of the time evaluate to the values Excel saved (SPEC-0008). Returns whether
/// it pinned: a workbook without a timestamp, or with one outside Excel's calendar, keeps its
/// clock.
pub fn pin_clock_from_metadata(model: &mut Model) -> bool {
    let Some(clock) = clock_from_metadata(model) else {
        return false;
    };
    model.set_clock(clock);
    true
}

/// The fixed clock at `model`'s METADATA timestamp; `None` without one, or when it is outside
/// Excel's calendar.
pub fn clock_from_metadata(model: &Model) -> Option<Clock> {
    get_metadata_timestamp(model).and_then(|ms| Clock::fixed_unix_millis(ms).ok())
}
