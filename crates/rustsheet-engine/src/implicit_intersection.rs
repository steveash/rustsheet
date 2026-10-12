use crate::{calc_result::Range, expressions::types::CellReferenceIndex};

/// It returns the closest cell from `cell_reference` to range in the same column/row
/// Examples
///  * `i_i(B5`, A2:A9) -> B5
///  * `i_i(B5`, A7:A9) -> None
///  * `i_i(B5`, A2:D2) -> B2
///  * `i_i(B2`, Data!A2:D2) -> Data!B2 (a cell in the row of a one-row range intersects by column)
///
/// Excel-parity, cross-sheet:
///  * a 1x1 range always dereferences, regardless of the consuming cell's
///    position or sheet (Excel: `=@Sheet2!D5` and OFFSET/INDIRECT results in
///    scalar context return the single cell's value);
///  * row/column-aligned intersection works across sheets — the intersection
///    happens in the range's own coordinate space (Excel: `=Data!D1:D5`
///    entered on another sheet's C3 returns Data!D3). The resulting reference
///    lives on the RANGE's sheet.
pub(crate) fn implicit_intersection(
    cell_reference: &CellReferenceIndex,
    range: &Range,
) -> Option<CellReferenceIndex> {
    let left = &range.left;
    let right = &range.right;
    // A range spanning two sheets never intersects.
    if left.sheet != right.sheet {
        return None;
    }
    let sheet = left.sheet;
    // Single-cell range: always dereference (position- and sheet-independent).
    if left.row == right.row && left.column == right.column {
        return Some(CellReferenceIndex {
            sheet,
            row: left.row,
            column: left.column,
        });
    }
    let row = cell_reference.row;
    let column = cell_reference.column;
    // A column intersects the cell's row and a row its column. A cell in the row of a one-row
    // range intersects it by column too: `@XLOOKUP(2,Data!A1:A3,Data!B1:C3)` in another sheet's
    // B2 is Data!B2 (probes/xlookup-row-column-picks B2, C2).
    if left.column == right.column && row >= left.row && row <= right.row {
        return Some(CellReferenceIndex {
            sheet,
            row,
            column: left.column,
        });
    }
    if left.row == right.row && column >= left.column && column <= right.column {
        return Some(CellReferenceIndex {
            sheet,
            row: left.row,
            column,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(sheet: u32, row: i32, column: i32) -> CellReferenceIndex {
        CellReferenceIndex { sheet, row, column }
    }

    #[test]
    fn a_cell_in_a_one_row_range_row_intersects_by_column() {
        // probes/xlookup-row-column-picks B2, C2: Data!B2:C2 seen from another sheet's B2 and C2.
        let range = Range {
            left: at(1, 2, 2),
            right: at(1, 2, 3),
        };
        assert_eq!(
            implicit_intersection(&at(0, 2, 2), &range),
            Some(at(1, 2, 2))
        );
        assert_eq!(
            implicit_intersection(&at(0, 2, 3), &range),
            Some(at(1, 2, 3))
        );
        assert_eq!(implicit_intersection(&at(0, 2, 5), &range), None);
        let block = Range {
            left: at(1, 2, 2),
            right: at(1, 3, 3),
        };
        assert_eq!(implicit_intersection(&at(0, 2, 2), &block), None);
    }
}
