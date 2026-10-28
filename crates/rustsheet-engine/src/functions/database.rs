use std::sync::Arc;

use crate::arithmetic::{extended_sum, flush_subnormal};
use crate::constants::LAST_ROW;
use crate::expressions::parser::stringify::retarget_unqualified_references;
use crate::types::Cell;
use crate::{
    arithmetic::extended_quotient,
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    Model,
};

use super::criteria::{build_criteria_with, CriteriaFlavor};
use super::lift::LiftCost;
use super::statistical::variance::{variance_result, VarianceKind};
use super::Function;

/// The resolved `database`, `field` and `criteria` of one D-function element.
struct DbQuery {
    left: CellReferenceIndex,
    right: CellReferenceIndex,
    /// The field's absolute column; `None` for a written empty `field` (`DCOUNT(A1:C6,,E1:E2)`),
    /// which DCOUNT and DCOUNTA read as "the matching records".
    field: Option<i32>,
    criteria: (CellReferenceIndex, CellReferenceIndex),
}

/// Whether a computed criterion (a criteria cell under a header that names no database column)
/// holds: TRUE, a non-zero number and the text "TRUE" (probed in uppercase only) hold; FALSE, 0, an error and any
/// other text (">2") do not (probes/db-computed-criteria-shapes AB1-AB18, statistics.md
/// "Database criteria").
fn computed_criterion_holds(condition: &CalcResult) -> bool {
    match condition {
        CalcResult::Boolean(value) => *value,
        CalcResult::Number(value) => *value != 0.0,
        CalcResult::String(text) => text.eq_ignore_ascii_case("TRUE"),
        _ => false,
    }
}

/// Whether a formula condition evaluated at `context` (its cell moved down to a record) reads
/// only the record area of the database (its rows below the header row, its columns): Excel
/// matches no record whose moved relative reference onto the database's sheet lands on the header
/// row or above, below the last record, or outside the database's columns, and a range matches
/// when it overlaps that area (`SUM(B5:B6)>0` over Same!A5:C12 matches its first record,
/// `SUM(B4:B5)>0` does not). Over Db!A2:D9 (header row 2) with `=B2>3`, the first record reads
/// B2 = 4 and does not match (criteria.md "D* computed criteria and an unknown header in the
/// gen-m9 database sweep (T-2258)"; probes/db-computed-criteria-cross-sheet). A reference with an
/// absolute row (a range with two) stays put and is not checked (probes/db-criteria-formula-placement
/// F2, F14; `$B$5>0` on the header row matches), nor is one on another sheet than the database's.
fn reads_records(node: &Node, context: CellReferenceIndex, query: &DbQuery) -> bool {
    let rows = query.left.row + 1..=query.right.row;
    let columns = query.left.column..=query.right.column;
    let moved = |at: i32, absolute: bool, by: i32| at + if absolute { 0 } else { by };
    let overlaps = |span: &std::ops::RangeInclusive<i32>, a: i32, b: i32| {
        a.min(b) <= *span.end() && a.max(b) >= *span.start()
    };
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        let holds = match node {
            Node::ReferenceKind {
                sheet_index,
                absolute_row: false,
                absolute_column,
                row,
                column,
                ..
            } => {
                *sheet_index != query.left.sheet
                    || (rows.contains(&(row + context.row))
                        && columns.contains(&moved(*column, *absolute_column, context.column)))
            }
            Node::RangeKind {
                sheet_index,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
                ..
            } => {
                *sheet_index != query.left.sheet
                    || (*absolute_row1 && *absolute_row2)
                    || (overlaps(
                        &rows,
                        moved(*row1, *absolute_row1, context.row),
                        moved(*row2, *absolute_row2, context.row),
                    ) && overlaps(
                        &columns,
                        moved(*column1, *absolute_column1, context.column),
                        moved(*column2, *absolute_column2, context.column),
                    ))
            }
            _ => true,
        };
        if !holds {
            return false;
        }
        node.push_children(&mut pending);
    }
    true
}

/// The corners of a reference argument; an error passes through, anything else is `#VALUE!`
/// (a database given as an array makes Excel refuse the workbook, statistics.md "Database
/// function argument modes"; a criteria element of a lifted array is a scalar, `#VALUE!`).
fn reference_corners(
    value: &CalcResult,
    cell: CellReferenceIndex,
) -> Result<(CellReferenceIndex, CellReferenceIndex), CalcResult> {
    match value {
        CalcResult::Range { left, right } => Ok((*left, *right)),
        error @ CalcResult::Error { .. } => Err(error.clone()),
        _ => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Expected reference".to_string(),
        )),
    }
}

impl Model {
    /// Runs a D-function through the SPEC-0011 driver: `field` and `criteria` lift over array
    /// literals, `database` and a criteria range are read whole (probes/db-argument-modes).
    /// `empty_field_counts` is for DCOUNT and DCOUNTA, whose written empty `field` counts records.
    fn db_call(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        empty_field_counts: bool,
        compute: fn(&mut Model, &DbQuery, CellReferenceIndex) -> CalcResult,
    ) -> CalcResult {
        // Only an argument that is not a plain literal or reference can lift, and only then is the
        // call priced (the database and criteria are read again by the driver; cheap).
        let may_lift = args.iter().skip(1).any(|arg| {
            !matches!(
                arg.ungrouped(),
                Node::BooleanKind(_)
                    | Node::NumberKind(_)
                    | Node::StringKind(_)
                    | Node::ReferenceKind { .. }
                    | Node::RangeKind { .. }
                    | Node::ThisRowKind { .. }
                    | Node::EmptyArgKind
            )
        });
        let cost = if may_lift {
            self.db_element_cost(args, cell)
        } else {
            LiftCost::Unit
        };
        self.lift_call_costed(
            kind,
            args,
            cost,
            cell,
            &mut |model, values| match model.db_query(values, empty_field_counts, cell) {
                Ok(query) => compute(model, &query, cell),
                Err(error) => error,
            },
        )
    }

    /// What one lifted element costs: it scans every database row, reading the criteria headers
    /// against the database's columns and each criteria row's cells (`db_row_matches_criteria`),
    /// plus the field read. A whole-column database or criteria range is therefore `#ERROR!` when
    /// lifted (limits.rs `MAX_LIFTED_CELL_VISITS`; a DEVIATION, statistics.md).
    fn db_element_cost(&mut self, args: &[Node], cell: CellReferenceIndex) -> LiftCost {
        let size = |range: &crate::calc_result::Range| {
            let rows = u64::try_from(range.right.row - range.left.row + 1).unwrap_or(0);
            let columns = u64::try_from(range.right.column - range.left.column + 1).unwrap_or(0);
            (rows, columns)
        };
        let Some(Ok(database)) = args.first().map(|node| self.get_reference(node, cell)) else {
            return LiftCost::Unit;
        };
        let (rows, columns) = size(&database);
        let (criteria_rows, criteria_columns) =
            match args.get(2).map(|node| self.get_reference(node, cell)) {
                Some(Ok(criteria)) => size(&criteria),
                _ => (0, 0),
            };
        let per_row = criteria_columns
            .saturating_mul(columns)
            .saturating_add(criteria_rows.saturating_mul(criteria_columns))
            .saturating_add(1);
        LiftCost::Cells(rows.saturating_mul(per_row))
    }

    fn db_query(
        &mut self,
        values: &[CalcResult],
        empty_field_counts: bool,
        cell: CellReferenceIndex,
    ) -> Result<DbQuery, CalcResult> {
        let [database, field, criteria] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let (left, right) = reference_corners(database, cell)?;
        let field = if matches!(field, CalcResult::EmptyArg) && empty_field_counts {
            None
        } else {
            Some(self.resolve_db_field_column(left, right, field, cell)?)
        };
        let criteria = reference_corners(criteria, cell)?;
        // A database or a criteria range of one row (headers only) is `#VALUE!` in every
        // D-function (probes/db-header-criteria-shapes AB2-AB37).
        if right.row <= left.row || criteria.1.row <= criteria.0.row {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "No data rows in database".to_string(),
            ));
        }
        Ok(DbQuery {
            left,
            right,
            field,
            criteria,
        })
    }

    /// Folds over the field values of the records that match the criteria. A query with no
    /// field passes `EmptyArg` for each record.
    fn db_fold<T>(
        &mut self,
        query: &DbQuery,
        init: T,
        mut step: impl FnMut(T, CalcResult) -> T,
    ) -> T {
        let mut acc = init;
        let columns = self.db_criteria_columns(query);
        for row in (query.left.row + 1)..=query.right.row {
            if !self.db_row_matches_criteria(query, row, &columns) {
                continue;
            }
            let value = match query.field {
                Some(column) => self.evaluate_cell(CellReferenceIndex {
                    sheet: query.left.sheet,
                    row,
                    column,
                }),
                None => CalcResult::EmptyArg,
            };
            acc = step(acc, value);
        }
        acc
    }

    /// The finite numbers of the field among the matching records.
    fn db_fold_numbers<T>(
        &mut self,
        query: &DbQuery,
        init: T,
        mut step: impl FnMut(T, f64) -> T,
    ) -> T {
        self.db_fold(query, init, |acc, value| match value {
            CalcResult::Number(n) if n.is_finite() => step(acc, n),
            _ => acc,
        })
    }

    /// DMAX and DMIN: the extreme `pick` chooses among the matching numbers, 0 when there are none.
    fn db_extreme(&mut self, query: &DbQuery, pick: fn(f64, f64) -> f64) -> CalcResult {
        let best = self.db_fold_numbers(query, None, |best: Option<f64>, n| {
            Some(best.map_or(n, |current| pick(n, current)))
        });
        CalcResult::Number(best.unwrap_or(0.0))
    }

    fn db_numbers(&mut self, query: &DbQuery) -> Vec<f64> {
        self.db_fold_numbers(query, Vec::new(), |mut numbers, n| {
            numbers.push(n);
            numbers
        })
    }

    // =DAVERAGE(database, field, criteria)
    pub(crate) fn fn_daverage(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(
            Function::Daverage,
            args,
            cell,
            false,
            |model, query, cell| {
                // Rounds the total to a double after every addition, like SUM (numbers.md
                // "Accumulators beyond SUM and AVERAGE", probes-numeric/daverage-accumulation).
                let (sum, count) =
                    model.db_fold_numbers(query, (0.0f64, 0usize), |(sum, count), n| {
                        (extended_sum(sum, n), count + 1)
                    });
                if count == 0 {
                    return CalcResult::Error {
                        error: Error::DIV,
                        origin: cell,
                        message: "No numeric values matched criteria".to_string(),
                    };
                }
                // Rounds twice like AVERAGE's (numbers.md "Division rounds twice", T-1648), and a
                // subnormal quotient is 0 as for AVERAGE (probes-numeric/daverage-accumulation
                // A20:A24, T-1649).
                CalcResult::Number(flush_subnormal(extended_quotient(sum, count as f64)))
            },
        )
    }

    // =DCOUNT(database, field, criteria)
    // Counts numeric entries in the field for rows that match criteria
    pub(crate) fn fn_dcount(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dcount, args, cell, true, |model, query, _| {
            let no_field = query.field.is_none();
            let count = model.db_fold(query, 0usize, |count, value| {
                count + usize::from(no_field || matches!(value, CalcResult::Number(_)))
            });
            CalcResult::Number(count as f64)
        })
    }

    // =DCOUNTA(database, field, criteria)
    // Counts non-empty entries (any type) in the field for rows that match criteria
    pub(crate) fn fn_dcounta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dcounta, args, cell, true, |model, query, _| {
            let no_field = query.field.is_none();
            let count = model.db_fold(query, 0usize, |count, value| {
                let filled = !matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg);
                count + usize::from(no_field || filled)
            });
            CalcResult::Number(count as f64)
        })
    }

    // =DGET(database, field, criteria)
    // Returns the (single) field value for the unique matching row
    pub(crate) fn fn_dget(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dget, args, cell, false, |model, query, cell| {
            // Only a record with a non-empty field counts: two matching records of which one has a
            // blank field give the other's value (gen-m9 c9d7659896d95368 A1, criteria.md "D*
            // computed criteria"), and none with a field is `#VALUE!` as if nothing matched
            // (probes/db-computed-criteria-shapes AB51).
            let mut filled = 0usize;
            let mut result = None;
            model.db_fold(query, (), |(), value| {
                if !matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                    filled += 1;
                    result = Some(value);
                }
            });
            match (filled, result) {
                (1, Some(value)) => value,
                (0, _) | (_, None) => {
                    CalcResult::new_error(Error::VALUE, cell, "No matching record".to_string())
                }
                _ => CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "More than one matching record".to_string(),
                },
            }
        })
    }

    // =DMAX(database, field, criteria)
    pub(crate) fn fn_dmax(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dmax, args, cell, false, |model, query, _| {
            model.db_extreme(query, f64::max)
        })
    }

    // =DMIN(database, field, criteria)
    pub(crate) fn fn_dmin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dmin, args, cell, false, |model, query, _| {
            model.db_extreme(query, f64::min)
        })
    }

    // =DSUM(database, field, criteria)
    pub(crate) fn fn_dsum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dsum, args, cell, false, |model, query, _| {
            // Each addition rounds twice like SUM's (probes-numeric/double-rounding-accumulators).
            CalcResult::Number(model.db_fold_numbers(query, 0.0, extended_sum))
        })
    }

    // =DPRODUCT(database, field, criteria)
    pub(crate) fn fn_dproduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dproduct, args, cell, false, |model, query, _| {
            let (product, has_numeric) =
                model.db_fold_numbers(query, (1.0f64, false), |(product, _), n| {
                    (product * n, true)
                });
            // Excel returns 0 when no rows / no numeric values match for DPRODUCT
            CalcResult::Number(if has_numeric { product } else { 0.0 })
        })
    }

    // =DSTDEV(database, field, criteria): sample standard deviation of matching numeric values
    pub(crate) fn fn_dstdev(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dstdev, args, cell, false, |model, query, cell| {
            let values = model.db_numbers(query);
            variance_result(&values, VarianceKind::Sample, true, "DSTDEV", cell)
        })
    }

    // =DVAR(database, field, criteria): sample variance of matching numeric values
    pub(crate) fn fn_dvar(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dvar, args, cell, false, |model, query, cell| {
            let values = model.db_numbers(query);
            variance_result(&values, VarianceKind::Sample, false, "DVAR", cell)
        })
    }

    // =DSTDEVP(database, field, criteria): population standard deviation of matching numeric values
    pub(crate) fn fn_dstdevp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(
            Function::Dstdevp,
            args,
            cell,
            false,
            |model, query, cell| {
                let values = model.db_numbers(query);
                variance_result(&values, VarianceKind::Population, true, "DSTDEVP", cell)
            },
        )
    }

    // =DVARP(database, field, criteria): population variance of matching numeric values
    pub(crate) fn fn_dvarp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.db_call(Function::Dvarp, args, cell, false, |model, query, cell| {
            let values = model.db_numbers(query);
            variance_result(&values, VarianceKind::Population, false, "DVARP", cell)
        })
    }

    /// Resolve the "field" (2nd arg) to an absolute column index (i32) within the sheet.
    /// Field can be a number (1-based index) or a header name (case-insensitive).
    /// Returns the absolute column index, not a 1-based offset within the database range.
    fn resolve_db_field_column(
        &mut self,
        db_left: CellReferenceIndex,
        db_right: CellReferenceIndex,
        field: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        let field_column_name = match field {
            CalcResult::String(s) => s.to_lowercase(),
            CalcResult::Number(index) => {
                // `as` saturates a huge or NaN index; i64 keeps the sum from overflowing.
                let index = i64::from(index.floor() as i32);
                if index < 1 || i64::from(db_left.column) + index - 1 > i64::from(db_right.column) {
                    return Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Field index out of range".to_string(),
                    });
                }
                return Ok(db_left.column + (index as i32) - 1);
            }
            CalcResult::Boolean(b) => {
                return if *b {
                    Ok(db_left.column)
                } else {
                    // Index 0 is out of range
                    Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid field specifier".to_string(),
                    })
                };
            }
            error @ CalcResult::Error { .. } => {
                return Err(error.clone());
            }
            CalcResult::Range { .. } => {
                return Err(CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "Arrays not supported yet".to_string(),
                })
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => "".to_string(),
            CalcResult::Array(_) | CalcResult::Lambda(_) => {
                return Err(CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "Arrays not supported yet".to_string(),
                })
            }
        };

        // Headers read as text as in `db_criteria_columns`, except that a field of "" (or a
        // blank cell) names a blank header (probes/db-header-criteria-shapes AB58-AB60,
        // probes/db-computed-criteria-shapes AB40-AB42).
        for column in db_left.column..=db_right.column {
            let header = self.evaluate_cell(CellReferenceIndex {
                sheet: db_left.sheet,
                row: db_left.row,
                column,
            });
            if let Ok(text) = Self::cast_string_ref(&header, cell) {
                if text.to_lowercase() == field_column_name {
                    return Ok(column);
                }
            }
        }

        Err(CalcResult::Error {
            error: Error::VALUE,
            origin: cell,
            message: "Field header not found".to_string(),
        })
    }

    /// The database column each criteria column's header names, `None` when it names none.
    /// Headers compare as text, ignoring case: a number header reads as its coerced text, so the
    /// number 0.5 and the text "0.5" match each other (and 1E-15 matches "0.000000000000001", not
    /// "1E-15"), TRUE matches "TRUE", and " Qty" does not match "Qty". A blank or error criteria
    /// header names no column, even over a blank database header (probes/db-header-criteria-shapes
    /// AB38-AB54, probes/db-computed-criteria-shapes AB29-AB34; statistics.md "Database criteria").
    fn db_criteria_columns(&mut self, query: &DbQuery) -> Vec<Option<i32>> {
        let (c_left, c_right) = query.criteria;
        let header_text = |model: &mut Model, row: i32, sheet: u32, column: i32| {
            let value = model.evaluate_cell(CellReferenceIndex { sheet, row, column });
            match value {
                CalcResult::EmptyCell | CalcResult::EmptyArg => None,
                value => Model::cast_string_ref(&value, query.left)
                    .ok()
                    .map(|text| text.to_lowercase()),
            }
        };
        let db_headers: Vec<(i32, Option<String>)> = (query.left.column..=query.right.column)
            .map(|column| {
                let text = header_text(self, query.left.row, query.left.sheet, column);
                (column, text)
            })
            .collect();
        (c_left.column..=c_right.column)
            .map(|column| {
                let wanted = header_text(self, c_left.row, c_left.sheet, column)?;
                db_headers
                    .iter()
                    .find(|(_, text)| text.as_deref() == Some(wanted.as_str()))
                    .map(|(db_column, _)| *db_column)
            })
            .collect()
    }

    /// Whether database `row` matches the criteria: any criteria row (OR) whose every non-blank
    /// cell holds (AND), so a blank criteria row matches every record. A cell under a header that
    /// names a database column tests that column; a cell under any other header is a computed
    /// criterion that holds when its value is true (statistics.md "Database criteria").
    fn db_row_matches_criteria(
        &mut self,
        query: &DbQuery,
        row: i32,
        columns: &[Option<i32>],
    ) -> bool {
        let (c_left, c_right) = query.criteria;
        (c_left.row + 1..=c_right.row).any(|criteria_row| {
            columns
                .iter()
                .zip(c_left.column..)
                .all(|(db_column, column)| {
                    let condition = self.evaluate_cell(CellReferenceIndex {
                        sheet: c_left.sheet,
                        row: criteria_row,
                        column,
                    });
                    if matches!(condition, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                        return true;
                    }
                    match db_column {
                        Some(db_column) => {
                            let value = self.evaluate_cell(CellReferenceIndex {
                                sheet: query.left.sheet,
                                row,
                                column: *db_column,
                            });
                            self.criteria_cell_matches(&value, &condition)
                        }
                        None => {
                            let condition = self.db_computed_condition(
                                query,
                                row,
                                CellReferenceIndex {
                                    sheet: c_left.sheet,
                                    row: criteria_row,
                                    column,
                                },
                                condition,
                            );
                            computed_criterion_holds(&condition)
                        }
                    }
                })
        })
    }

    /// A computed criterion's value for database `row`. A formula condition is evaluated again
    /// as if it sat as many rows lower as the record is below the first record, so its relative
    /// references follow the record (a blank header over `=B2>2` tests each record's B cell);
    /// a constant is its own value (probes/db-computed-criteria-shapes AB23-AB26). Its references
    /// without a sheet prefix read the database's sheet, so a condition on another sheet is
    /// evaluated for the first record too (probes/db-computed-criteria-cross-sheet). A record
    /// whose moved references leave the database's record area does not match ([`reads_records`]).
    fn db_computed_condition(
        &mut self,
        query: &DbQuery,
        row: i32,
        criteria_cell: CellReferenceIndex,
        value: CalcResult,
    ) -> CalcResult {
        let offset = row - query.left.row - 1;
        let formula = self
            .workbook
            .worksheet(criteria_cell.sheet)
            .ok()
            .and_then(|sheet| sheet.cell(criteria_cell.row, criteria_cell.column))
            .and_then(Cell::get_formula);
        let Some(formula) = formula else {
            return value;
        };
        // The shifted cell stays on the grid: the criteria row plus the record offset can pass
        // the last row only when the criteria range sits below the database's records.
        let context = CellReferenceIndex {
            row: criteria_cell.row + offset,
            ..criteria_cell
        };
        if context.row > LAST_ROW {
            return CalcResult::new_error(Error::REF, criteria_cell, "Off the grid".to_string());
        }
        let Some(mut node) = self
            .parsed_formulas
            .get(criteria_cell.sheet as usize)
            .and_then(|formulas| formulas.get(usize::try_from(formula).ok()?))
            .map(|formula| Arc::clone(&formula.evaluated))
        else {
            return value;
        };
        let other_sheet = criteria_cell.sheet != query.left.sheet;
        if other_sheet {
            retarget_unqualified_references(Arc::make_mut(&mut node), query.left.sheet);
        }
        if !reads_records(&node, context, query) {
            return CalcResult::Boolean(false);
        }
        if offset == 0 && !other_sheet {
            return value;
        }
        // A formula condition that itself runs a D-function with formula conditions would
        // re-evaluate them without passing through a cell, its work growing as the records to the
        // power of the nesting (directive 5). Only one level runs: a nested one is `#ERROR!`, so
        // its condition fails (statistics.md "Database criteria", DEVIATION).
        if self.defined_names_running.db_condition_running {
            return CalcResult::new_error(
                Error::ERROR,
                criteria_cell,
                "Nested criteria formula".to_string(),
            );
        }
        self.defined_names_running.db_condition_running = true;
        let result = self.evaluate_node_in_context(&node, context);
        // Only reached when no condition was running (the guard above returns otherwise).
        self.defined_names_running.db_condition_running = false;
        self.dereference(result, context)
    }

    /// Whether a database value satisfies one criteria cell (shared grammar, D-function flavor).
    fn criteria_cell_matches(&self, db_val: &CalcResult, crit_cell: &CalcResult) -> bool {
        // Criteria text is used as-is, not trimmed: Excel's DSUM with criterion "  > 1 " matches
        // nothing (smoke/d-function-criteria J2 = 0, excel-graph 2026-10-04; see
        // docs/excel-semantics/criteria.md).
        let matches = build_criteria_with(
            crit_cell,
            self.locale,
            &|| self.current_year(),
            self.workbook.settings.date_system,
            CriteriaFlavor::Database,
        );
        matches(db_val)
    }
}
