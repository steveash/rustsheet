use crate::arithmetic::{extended_sum, flush_subnormal};
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
    fn db_element_cost(&mut self, args: &[Node], cell: CellReferenceIndex) -> LiftCost<'static> {
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
        if right.row <= left.row {
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
        for row in (query.left.row + 1)..=query.right.row {
            if !self.db_row_matches_criteria(query.left, query.right, row, query.criteria) {
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
            let mut matches = 0usize;
            let mut result = None;
            model.db_fold(query, (), |(), value| {
                matches += 1;
                result = Some(value);
            });
            match (matches, result) {
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

        // We search in the database a column whose header matches field_column_name
        for column in db_left.column..=db_right.column {
            let v = self.evaluate_cell(CellReferenceIndex {
                sheet: db_left.sheet,
                row: db_left.row,
                column,
            });
            match &v {
                CalcResult::String(s) => {
                    if s.to_lowercase() == field_column_name {
                        return Ok(column);
                    }
                }
                CalcResult::Number(n) => {
                    if field_column_name == n.to_string() {
                        return Ok(column);
                    }
                }
                CalcResult::Boolean(b) => {
                    if field_column_name == b.to_string() {
                        return Ok(column);
                    }
                }
                CalcResult::Error { .. }
                | CalcResult::Range { .. }
                | CalcResult::EmptyCell
                | CalcResult::EmptyArg
                | CalcResult::Array(_)
                | CalcResult::Lambda(_) => {}
            }
        }

        Err(CalcResult::Error {
            error: Error::VALUE,
            origin: cell,
            message: "Field header not found".to_string(),
        })
    }

    /// Check whether a database row matches the criteria range.
    /// Criteria logic: OR across criteria rows; AND across columns within a row.
    fn db_row_matches_criteria(
        &mut self,
        db_left: CellReferenceIndex,
        db_right: CellReferenceIndex,
        row: i32,
        criteria: (CellReferenceIndex, CellReferenceIndex),
    ) -> bool {
        let (c_left, c_right) = criteria;

        // Read criteria headers (first row of criteria range)
        // Map header name (lowercased) -> db column (if exists)
        let mut crit_cols: Vec<i32> = Vec::new();
        let mut header_count = 0;
        // We cover the criteria table:
        //  headerA | headerB | ...
        //  critA1  | critA2  | ...
        //  critB1  | critB2  | ...
        //  ...
        for column in c_left.column..=c_right.column {
            let cell = CellReferenceIndex {
                sheet: c_left.sheet,
                row: c_left.row,
                column,
            };
            let criteria_header = self.evaluate_cell(cell);
            if let Ok(s) = self.cast_to_string(criteria_header, cell) {
                // Non-empty string header. If the header is non string we skip it
                header_count += 1;
                let wanted = s.to_lowercase();

                // Find corresponding Database column
                let mut found = false;
                for db_column in db_left.column..=db_right.column {
                    let db_header = self.evaluate_cell(CellReferenceIndex {
                        sheet: db_left.sheet,
                        row: db_left.row,
                        column: db_column,
                    });
                    if let Ok(hs) = self.cast_to_string(db_header, cell) {
                        if hs.to_lowercase() == wanted {
                            crit_cols.push(db_column);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    // that means the criteria column has no matching DB column
                    // If the criteria condition is empty then we remove this condition
                    // otherwise this condition can never be satisfied
                    // We evaluate all criteria rows to see if any is non-empty
                    let mut has_non_empty = false;
                    for r in (c_left.row + 1)..=c_right.row {
                        let ccell = self.evaluate_cell(CellReferenceIndex {
                            sheet: c_left.sheet,
                            row: r,
                            column,
                        });
                        if !matches!(ccell, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                            has_non_empty = true;
                            break;
                        }
                    }
                    if has_non_empty {
                        // This criteria column can never be satisfied
                        header_count -= 1;
                    }
                }
            };
        }

        if c_right.row <= c_left.row {
            // If no criteria rows (only headers), everything matches
            return true;
        }

        if header_count == 0 {
            // If there are not "String" headers, nothing matches
            // NB: There might be String headers that do not match any DB columns,
            // in that case everything matches.
            return false;
        }

        // Evaluate each criteria row (OR)
        for r in (c_left.row + 1)..=c_right.row {
            // AND across columns for this criteria row
            let mut and_ok = true;

            for (offset, db_col) in crit_cols.iter().enumerate() {
                // Criteria cell
                let ccell = self.evaluate_cell(CellReferenceIndex {
                    sheet: c_left.sheet,
                    row: r,
                    column: c_left.column + offset as i32,
                });

                // Empty criteria cell -> ignored
                if matches!(ccell, CalcResult::EmptyCell | CalcResult::EmptyArg) {
                    continue;
                }

                // Database value for this row/column
                let db_val = self.evaluate_cell(CellReferenceIndex {
                    sheet: db_left.sheet,
                    row,
                    column: *db_col,
                });

                if !self.criteria_cell_matches(&db_val, &ccell) {
                    and_ok = false;
                    break;
                }
            }

            if and_ok {
                // This criteria row satisfied (OR)
                return true;
            }
        }

        // none matched
        false
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
