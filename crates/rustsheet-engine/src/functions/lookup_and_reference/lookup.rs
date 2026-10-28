//! LOOKUP (both forms) and the vector reading MATCH shares with it.

use crate::cast::scalar_as_array;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::{array_node_to_calc_result, is_at_if, is_at_xlookup_take};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::lookup_index;
use super::lookup_table::{is_searched_kind, LookupTable, SearchVector};
use crate::functions::lift::{LiftCost, LiftOptions, LiftedArg};

/// LOOKUP's `#N/A` for a value that is not found or a position its `result_vector` cannot give.
fn not_found(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::Error {
        error: Error::NA,
        origin: cell,
        message: "Not found".to_string(),
    }
}

/// LOOKUP's searched argument, read once: the table, whether it is searched along its first
/// row (else its first column), and that row or column.
struct LookupSearchTable {
    table: LookupTable,
    by_row: bool,
    vector: SearchVector,
    /// The cells one lifted search costs: the vector's, or a tailed vector's logical size.
    scan: u64,
}

/// LOOKUP's `result_vector`, read once.
enum ResultVector {
    /// A 2-D result: `#N/A` wherever the value is found.
    TwoD,
    /// A range, its cells evaluated (`down` for a column), read past its end from `left`.
    Range {
        left: CellReferenceIndex,
        down: bool,
        vector: SearchVector,
    },
    /// An array (or a scalar as a one-element array), not read past.
    Array { table: LookupTable, down: bool },
}

impl Model {
    /// Materializes a value that is expected to be a vector (a single row or a
    /// single column) into a flat list of values. Used by MATCH: a value that is
    /// not a vector is `#N/A` (docs/excel-semantics/lookup.md), as is anything that
    /// is not a range or an array (see [`Model::lookup_table`]); an error is
    /// returned as is.
    pub(super) fn as_vector(
        &mut self,
        table: LookupTable,
        cell: CellReferenceIndex,
    ) -> Result<SearchVector, CalcResult> {
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
            LookupTable::External(_) if rows > 1 => table.first_column(self, cell),
            LookupTable::External(_) => table.first_row(self, cell),
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
    //
    // lookup_value lifts over a multi-cell range and an array, one search per element
    // (`ROWS(LOOKUP(A1:A3,A1:A3,B1:B3))` is 3; probes/lookup-argument-modes F16:G17,
    // gen-m9/adb3fd63c7368166 A4); the vectors are read once, whole, for every element.
    pub(crate) fn fn_lookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.lookup_value_arg(&args[0], cell);
        if target.is_error() {
            return target;
        }
        let search = self.lookup_search(&args[1], cell, args.len() == 3);
        if !LiftedArg::is_grid(&target) {
            let position = match Self::lookup_position(&target, &search, cell) {
                Ok(position) => position,
                Err(error) => return error,
            };
            // Vector form: result_vector is read only once the search has succeeded.
            let result = args.get(2).map(|node| self.lookup_result(node, cell));
            // A blank result cell is the empty value (lookup.md "VLOOKUP and HLOOKUP hand a
            // blank result cell to their caller as 0"; probes/lookup-vector-blank-result-to-caller).
            return self
                .lookup_pick(&search, result.as_ref(), position, cell)
                .blank_as_empty_value();
        }
        let result = args.get(2).map(|node| self.lookup_result(node, cell));
        // Each element tests the evaluated cells of the searched row or column once.
        let scan = search.as_ref().map_or(1, |search| search.scan);
        self.lift_values(
            vec![target],
            LiftCost::Cells(scan),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |model, values| {
                let target = &values[0];
                if target.is_error() {
                    return target.clone();
                }
                match Self::lookup_position(target, &search, cell) {
                    Ok(position) => model.lookup_pick(&search, result.as_ref(), position, cell),
                    Err(error) => error,
                }
            },
        )
    }

    /// LOOKUP's `lookup_vector` or `array`, read as the table it searches; a `lookup_vector`
    /// with a `result_vector` (`vector_form`) possibly read as a head and a tail
    /// ([`Model::tailed_lookup_search`]).
    fn lookup_search(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        vector_form: bool,
    ) -> Result<LookupSearchTable, CalcResult> {
        if vector_form {
            if let Some(search) = self.tailed_lookup_search(node, cell) {
                return Ok(search);
            }
        }
        // Read as a reference so that a single cell is a one-element vector
        // (`LOOKUP(2,C1,D1)`; probes/lookup-vector-edges A1-A4).
        let value = self.evaluate_node_with_reference(node, cell);
        // A scalar lookup_vector or array is a one-element array, however it is written or picked:
        // `LOOKUP(2,1)` is 1, `LOOKUP("b","a")` is "a", `LOOKUP(2,1,5)` is 5, `LOOKUP(2,IF(TRUE,1))` and
        // `LOOKUP(2,{1})` are 1 (probes/mode-direct-error-lookup-scalar-vector B1-B5, T-1651;
        // `LOOKUP(2,CHOOSE(1,@B1:B6*1))`: probes/legacy-choose-arm-aggregate-and-array-constant-at J1).
        let table = self.lookup_table(scalar_as_array(value), cell)?;
        // A wide table is searched along its first row, any other along its first column. In the
        // vector form that holds for a 2-D lookup_vector too (`LOOKUP(5,A1:D2,F1:F4)` searches
        // A1:D1; probes/lookup-2d-vector, -wide).
        let by_row = table.columns() > table.rows();
        let vector = if by_row {
            table.first_row(self, cell)
        } else {
            table.first_column(self, cell)
        };
        let scan = vector.scan_cells();
        Ok(LookupSearchTable {
            table,
            by_row,
            vector,
            scan,
        })
    }

    /// An element-wise `lookup_vector` of whole columns, or of bounded spans past the used area,
    /// searched as its used head and its tail row (SPEC-0020 Amendment 1 section 3, T-0494). A
    /// tail element of no searched kind (an error, as `1/(J:J<>"")` gives, or an empty element)
    /// is skipped by the approximate search for every target, as `SearchVector`'s virtual empty
    /// tail is: the vector is the head followed by that empty tail, its logical length the
    /// materialized array's, so every position found is the same and in the head. `None` (the
    /// ordinary reading) for any other shape or a tail of a searched kind, whose run of equal
    /// keys the search's midpoints would enter. The table is the head: only the array form,
    /// which never comes here, picks from it.
    fn tailed_lookup_search(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<LookupSearchTable> {
        let (head, tail) = self
            .read_tailed(node, cell, |model| {
                model.evaluate_node_with_reference(node, cell)
            })?
            .elements_by(|value| match value {
                CalcResult::Array(array) => Some(array),
                _ => None,
            })?;
        let head: Vec<Vec<ArrayNode>> = head.map(|element| vec![element]).collect();
        let values: Vec<CalcResult> = head
            .iter()
            .flatten()
            .map(|element| array_node_to_calc_result(element, cell))
            .collect();
        let len = match tail {
            Some((element, tail_len)) => {
                if is_searched_kind(&array_node_to_calc_result(&element, cell)) {
                    return None;
                }
                values.len().saturating_add(tail_len)
            }
            // No tail when the pass is being abandoned: the value is discarded.
            None => values.len(),
        };
        Some(LookupSearchTable {
            table: LookupTable::Array(head),
            by_row: false,
            vector: SearchVector::with_len(values, len),
            scan: u64::try_from(len).unwrap_or(u64::MAX).max(1),
        })
    }

    /// The 0-based position LOOKUP finds `target` at, or the error of the call. LOOKUP always
    /// assumes the values being searched are in ascending order.
    fn lookup_position(
        target: &CalcResult,
        search: &Result<LookupSearchTable, CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<usize, CalcResult> {
        let search = search.as_ref().map_err(Clone::clone)?;
        lookup_index(target, &search.vector, true).ok_or_else(|| not_found(cell))
    }

    /// The value LOOKUP returns for a match at `position`: from `result` in the vector form,
    /// from the last row or column of the searched table in the array form.
    fn lookup_pick(
        &mut self,
        search: &Result<LookupSearchTable, CalcResult>,
        result: Option<&Result<ResultVector, CalcResult>>,
        position: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match (result, search) {
            (Some(Err(error)), _) | (None, Err(error)) => error.clone(),
            (Some(Ok(result)), _) => self.lookup_result_at(result, position, cell),
            (None, Ok(search)) => {
                let table = &search.table;
                if search.by_row {
                    table.get(self, table.rows() - 1, position as i32, cell)
                } else {
                    table.get(self, position as i32, table.columns() - 1, cell)
                }
            }
        }
    }

    /// MATCH's, VLOOKUP's and HLOOKUP's table argument read as a table: a range of a linked
    /// workbook's cached sheet in place ([`Model::cached_external_lookup_area`]), anything else
    /// through [`Model::evaluate_lookup_table_arg`] and [`Model::lookup_table`].
    pub(super) fn lookup_table_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LookupTable, CalcResult> {
        if let Some(area) = self.cached_external_lookup_area(node, cell) {
            return Ok(LookupTable::External(area));
        }
        let value = self.evaluate_lookup_table_arg(node, cell);
        self.lookup_table(value, cell)
    }

    /// A LOOKUP, MATCH, VLOOKUP or HLOOKUP argument read as a table: a range or an array, or else
    /// its own error. A single value is no table: `#VALUE!` for text that is not a number, `#N/A`
    /// for anything else (`MATCH("b","b",0)` and `VLOOKUP("b","b",1,FALSE)` are `#VALUE!`,
    /// `MATCH(2,2,0)`, `MATCH("2","2",0)` and `MATCH(TRUE,TRUE,0)` `#N/A`;
    /// probes/match-single-value-lookup-array, T-1862). LOOKUP reads a single value as a
    /// one-element array before it gets here.
    pub(super) fn lookup_table(
        &self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<LookupTable, CalcResult> {
        match value {
            CalcResult::Range { left, right } => Ok(LookupTable::Range { left, right }),
            CalcResult::Array(array) => Ok(LookupTable::Array(array)),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::String(text) if self.cast_number(&text).is_none() => {
                Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                })
            }
            _ => Err(CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            }),
        }
    }

    /// LOOKUP's `result_vector`, read once (docs/excel-semantics/lookup.md "LOOKUP vector form
    /// with 2-D or short vectors"; probes/lookup-2d-vector, -wide): its error, or what
    /// [`Model::lookup_result_at`] picks from.
    fn lookup_result(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<ResultVector, CalcResult> {
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
        // A scalar result_vector is a one-element array and is not read past
        // (`LOOKUP(2,A1:A2,5)` is 5; probes/lookup-vector-edges A5-A8).
        let table = self.lookup_table(scalar_as_array(picked), cell)?;
        let (rows, columns) = (table.rows(), table.columns());
        if rows > 1 && columns > 1 {
            return Ok(ResultVector::TwoD);
        }
        let down = rows > 1;
        Ok(match table {
            LookupTable::Range { left, right } => {
                // Every cell of the range is evaluated, as before, so a result range holding the
                // formula's own cell stays circular.
                let vector = self.range_search_vector(&left, &right, down);
                ResultVector::Range { left, down, vector }
            }
            table @ (LookupTable::Array(_) | LookupTable::External(_)) => {
                ResultVector::Array { table, down }
            }
        })
    }

    /// The value at 0-based `position` of LOOKUP's `result_vector`: a 2-D result is `#N/A`; a
    /// range shorter than `position` reads on past its end, down for a column and to the right
    /// for a row or a single cell (`LOOKUP(99,A1:B2,F1)` reads G1); an array shorter than
    /// `position` is `#N/A`.
    fn lookup_result_at(
        &mut self,
        result: &ResultVector,
        position: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let Ok(offset) = i32::try_from(position) else {
            return not_found(cell);
        };
        match result {
            ResultVector::TwoD => not_found(cell),
            ResultVector::Range { left, down, vector } => {
                if position < vector.len() {
                    return vector.get(position).clone();
                }
                let target = if *down {
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
            ResultVector::Array { table, down } => {
                let (row, column) = if *down { (offset, 0) } else { (0, offset) };
                if row < table.rows() && column < table.columns() {
                    table.get(self, row, column, cell)
                } else {
                    not_found(cell)
                }
            }
        }
    }
}
