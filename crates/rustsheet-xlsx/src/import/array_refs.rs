//! The bound on the cells an xlsx sheet's array formulas may cover, checked before the importer
//! maps each covered cell to its anchor: a `ref` is a few bytes that can name the whole sheet,
//! and the map is filled before the model checks anything (T-3437, directive 5).

use rustsheet_engine::LAST_ROW;

use crate::error::XlsxError;

/// The most cells one array formula may cover: one whole column, the engine's array limit
/// (`limits::MAX_SIZE`, which the load check applies later, T-2896); the two must stay equal.
const MAX_ARRAY_CELLS: u64 = LAST_ROW.unsigned_abs() as u64;

/// The cells the array formulas of one workbook may cover beyond one per 16 bytes of its sheets
/// read: four whole columns, once per load, so that up to four whole-column spills side by side,
/// whose cells come after their anchors, still load; a fifth is refused (a DEVIATION,
/// docs/excel-semantics/xlsx-storage.md).
const SLACK: u64 = 4 * MAX_ARRAY_CELLS;

/// The cells the array formulas of one workbook's sheets cover, against the bytes of those sheets
/// read so far. One tally serves the whole load, so many small sheets share one [`SLACK`].
#[derive(Default)]
pub(crate) struct ArrayClaims {
    /// The bytes of the sheets read before the one being read.
    read_before: u64,
    /// The bytes of the sheet being read so far, its reader's position.
    read: u64,
    claimed: u64,
}

impl ArrayClaims {
    /// Records that the sheet being read has been read up to byte `position`.
    pub(super) fn read_up_to(&mut self, position: u64) {
        self.read = position;
    }

    /// Ends the sheet being read: its bytes count for the sheets after it.
    pub(super) fn end_sheet(&mut self) {
        self.read_before = self.read_before.saturating_add(self.read);
        self.read = 0;
    }

    /// Charges the cells of the array formula `range`, whose corners are (`row1`, `column1`) and
    /// (`row2`, `column2`), before each is mapped. Refused: an array of more than a whole column,
    /// and arrays covering more than one cell per 16 bytes read plus [`SLACK`]. The bytes counted
    /// are the ones read when the anchor is reached, not a size the package declares. Once its
    /// cells are read, a covered cell has paid for itself: Excel writes `r` on every cell, so it
    /// takes at least 22 bytes (`<c r="A1"><v>1</v></c>`), and at least 14 without `r`.
    pub(super) fn claim(
        &mut self,
        range: &str,
        (row1, column1, row2, column2): (i32, i32, i32, i32),
    ) -> Result<(), XlsxError> {
        let side = |first: i32, last: i32| u64::try_from(i64::from(last) - i64::from(first) + 1);
        let cells = match (side(row1, row2), side(column1, column2)) {
            (Ok(rows), Ok(columns)) => rows.saturating_mul(columns),
            _ => return Err(XlsxError::Xml(format!("Invalid range: {range}"))),
        };
        if cells > MAX_ARRAY_CELLS {
            return Err(XlsxError::Xml(format!(
                "The array formula at {range} spills over more than {LAST_ROW} cells"
            )));
        }
        self.claimed = self.claimed.saturating_add(cells);
        let read = self.read_before.saturating_add(self.read);
        if self.claimed > (read / 16).saturating_add(SLACK) {
            return Err(XlsxError::Xml(format!(
                "The array formulas up to {range} spill over more cells than the sheets read so far store"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ArrayClaims, MAX_ARRAY_CELLS, SLACK};

    /// A whole column of A, B, C and so on, anchored in row 1.
    fn column(claims: &mut ArrayClaims, n: i32) -> Result<(), crate::error::XlsxError> {
        claims.claim("a column", (1, n, 1_048_576, n))
    }

    /// The slack is once per load: a sheet ending does not renew it, so many small sheets with a
    /// whole-column array each are refused as one sheet with all of them would be.
    #[test]
    fn many_sheets_share_one_slack() {
        let mut claims = ArrayClaims::default();
        let fits = usize::try_from(SLACK / MAX_ARRAY_CELLS).unwrap_or(0);
        for sheet in 0..fits {
            claims.read_up_to(200);
            assert!(column(&mut claims, 1).is_ok(), "sheet {sheet}");
            claims.end_sheet();
        }
        claims.read_up_to(200);
        assert!(column(&mut claims, 1).is_err());
    }

    /// The bytes that count are the ones read, so the cells a file writes pay for the arrays
    /// that cover them, and a size the package merely declares buys nothing.
    #[test]
    fn bytes_read_pay_for_more_arrays() {
        let mut claims = ArrayClaims::default();
        for n in 1..=4 {
            assert!(column(&mut claims, n).is_ok());
        }
        assert!(column(&mut claims, 5).is_err());

        let mut claims = ArrayClaims::default();
        claims.read_up_to(16 * MAX_ARRAY_CELLS);
        for n in 1..=5 {
            assert!(column(&mut claims, n).is_ok());
        }
    }

    #[test]
    fn one_array_past_a_whole_column_is_refused() {
        let mut claims = ArrayClaims::default();
        let error = claims.claim("A1:B1048576", (1, 1, 1_048_576, 2)).err();
        assert!(error.is_some_and(|e| e.to_string().contains("more than")));
    }
}
