//! LOOKUP (both forms) and the vector reading MATCH shares with it.

use crate::cast::calc_result_to_array_node;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::{array_node_to_calc_result, is_at_if, is_at_xlookup_take};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::lookup_index;
use super::lookup_table::{LookupTable, SearchVector};

/// LOOKUP's `#N/A` for a value that is not found or a position its `result_vector` cannot give.
fn not_found(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::Error {
        error: Error::NA,
        origin: cell,
        message: "Not found".to_string(),
    }
}

impl Model {
    /// Materializes a value that is expected to be a vector (a single row or a
    /// single column) into a flat list of values. Used by MATCH: a value that is
    /// not a vector is `#N/A` (docs/excel-semantics/lookup.md), as is anything that
    /// is not a range or an array (see [`Model::lookup_table`]); an error is
    /// returned as is.
    pub(super) fn as_vector(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<SearchVector, CalcResult> {
        let table = Self::lookup_table(value, cell)?;
        let (rows, columns) = (table.rows(), table.columns());
        if rows > 1 && columns > 1 {
            return Err(CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Argument must be a vector".to_string(),
            });
        }
        Ok(match table {
            LookupTable::Range { left, right } => self.range_search_vector(&left, &right, rows > 1),
            LookupTable::Array(array) => SearchVector::new(
                array
                    .iter()
                    .flatten()
                    .map(|node| array_node_to_calc_result(node, cell))
                    .collect(),
            ),
        })
    }

    // LOOKUP has two forms:
    //   * Vector form: LOOKUP(lookup_value, lookup_vector, result_vector)
    //     Searches `lookup_vector` (a 2-D one along its first row or column, as
    //     in the array form) and returns the value at the same position in
    //     `result_vector` (see `lookup_result`).
    //   * Array form: LOOKUP(lookup_value, array)
    //     If `array` has more columns than rows it searches the first row and
    //     returns from the last row; otherwise it searches the first column and
    //     returns from the last column. (This subsumes the vector case where the
    //     "first" and "last" row/column coincide.)
    // The search is Excel's approximate-match search: see
    // docs/excel-semantics/lookup.md "Approximate-match lookups".
    pub(crate) fn fn_lookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.lookup_value_arg(&args[0], cell);
        if target.is_error() {
            return target;
        }
        // Read as a reference so that a single cell is a one-element vector
        // (`LOOKUP(2,C1,D1)`; probes/lookup-vector-edges A1-A4).
        let value = self.evaluate_node_with_reference(&args[1], cell);
        let table = match Self::lookup_table(value, cell) {
            Ok(table) => table,
            Err(error) => return error,
        };
        // A wide table is searched along its first row, any other along its first column. In the
        // vector form that holds for a 2-D lookup_vector too (`LOOKUP(5,A1:D2,F1:F4)` searches
        // A1:D1; probes/lookup-2d-vector, -wide).
        let by_row = table.columns() > table.rows();
        let search_vector = if by_row {
            table.first_row(self, cell)
        } else {
            table.first_column(self, cell)
        };
        // LOOKUP always assumes the values being searched are in ascending order.
        let Some(position) = lookup_index(&target, &search_vector, true) else {
            return not_found(cell);
        };
        if args.len() == 3 {
            // Vector form: the value at the same position of result_vector, read only once
            // the search has succeeded.
            return self.lookup_result(&args[2], position, cell);
        }
        // Array form: return from the last row or column.
        if by_row {
            table.get(self, table.rows() - 1, position as i32, cell)
        } else {
            table.get(self, position as i32, table.columns() - 1, cell)
        }
    }

    /// A LOOKUP or MATCH argument read as a table: a range or an array, its own error, and
    /// `#N/A` for anything else.
    fn lookup_table(
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<LookupTable, CalcResult> {
        match value {
            CalcResult::Range { left, right } => Ok(LookupTable::Range { left, right }),
            CalcResult::Array(array) => Ok(LookupTable::Array(array)),
            error @ CalcResult::Error { .. } => Err(error),
            _ => Err(CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Invalid".to_string(),
            }),
        }
    }

    /// The value at 0-based `position` of LOOKUP's `result_vector` (docs/excel-semantics/lookup.md
    /// "LOOKUP vector form with 2-D or short vectors"; probes/lookup-2d-vector, -wide): a 2-D
    /// result is `#N/A`; a range shorter than `position` reads on past its end, down for a column
    /// and to the right for a row or a single cell (`LOOKUP(99,A1:B2,F1)` reads G1); an array
    /// shorter than `position` is `#N/A`.
    fn lookup_result(
        &mut self,
        node: &Node,
        position: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // A single cell is read as a reference, so that it can be read past (`LOOKUP(99,A1:B2,F1)`).
        // The `@` of an IF that picks one cell keeps the reference
        // (`LOOKUP(99,A1:A3,@IF(TRUE,F1))` = `LOOKUP(99,A1:A3,F1)`;
        // probes/at-if-spill-anchor-operand H50, T-1368). Gated to `@IF`: a plain `IF` has no
        // evidence here, and a scalar result_vector stays `#N/A` (H70). `@XLOOKUP`/`@TAKE` keep
        // the reference too (probes/anchorarray-lookup-implicit-xlookup-take L1, J10, L10, N10).
        let picked = if is_at_if(node) || is_at_xlookup_take(node) {
            self.spill_anchor_operand(node, cell)
        } else {
            self.eval_ref_or_index(node, cell)
        };
        let value = match picked {
            // A scalar is a one-element array: `LOOKUP(2,A1:A2,5)` is 5, and 5 is not read past
            // (probes/lookup-vector-edges A5-A8).
            scalar @ (CalcResult::Number(_)
            | CalcResult::String(_)
            | CalcResult::Boolean(_)
            | CalcResult::EmptyCell) => {
                CalcResult::Array(vec![vec![calc_result_to_array_node(scalar)]])
            }
            other => other,
        };
        let table = match Self::lookup_table(value, cell) {
            Ok(table) => table,
            Err(error) => return error,
        };
        let (rows, columns) = (table.rows(), table.columns());
        if rows > 1 && columns > 1 {
            return not_found(cell);
        }
        let Ok(offset) = i32::try_from(position) else {
            return not_found(cell);
        };
        let down = rows > 1;
        match table {
            LookupTable::Range { left, right } => {
                // Every cell of the range is evaluated, as before, so a result range holding the
                // formula's own cell stays circular.
                let vector = self.range_search_vector(&left, &right, down);
                if position < vector.len() {
                    return vector.get(position).clone();
                }
                let target = if down {
                    left.row.checked_add(offset).map(|row| (row, left.column))
                } else {
                    left.column
                        .checked_add(offset)
                        .map(|column| (left.row, column))
                };
                match target {
                    Some((row, column)) if row <= LAST_ROW && column <= LAST_COLUMN => self
                        .evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }),
                    // Past the sheet's edge reads a blank, 0 (probes/lookup-vector-edges A15-A20).
                    _ => CalcResult::EmptyCell,
                }
            }
            LookupTable::Array(_) => {
                let (row, column) = if down { (offset, 0) } else { (0, offset) };
                if row < rows && column < columns {
                    table.get(self, row, column, cell)
                } else {
                    not_found(cell)
                }
            }
        }
    }
}
