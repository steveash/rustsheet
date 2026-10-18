use crate::{
    calc_result::{CalcResult, Range},
    constants::{LAST_COLUMN, LAST_ROW, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

mod accrint;
mod annuity;
mod dates;
mod discount;
mod financial_bonds;
mod financial_depreciation;
mod financial_util;
mod mirr;
mod rate_conversions;

use annuity::{compute_ipmt, compute_ppmt};
use financial_util::{compute_irr, compute_npv, compute_xirr, compute_xnpv};

use crate::functions::lift::number_or_error;

/// A computation's `(error, message)` as the cell's error.
fn formula_error(cell: CellReferenceIndex) -> impl Fn((Error, String)) -> CalcResult {
    move |(error, message)| CalcResult::new_error(error, cell, message)
}

// See:
// https://github.com/apache/openoffice/blob/c014b5f2b55cff8d4b0c952d5c16d62ecde09ca1/main/scaddins/source/analysis/financial.cxx

impl Model {
    fn get_array_of_numbers_generic(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        accept_number_node: bool,
        handle_empty_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
        handle_non_number_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        match self.evaluate_node_in_context(arg, *cell) {
            CalcResult::Number(value) if accept_number_node => values.push(value),
            CalcResult::Number(_) => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected range of numbers".to_string(),
                ));
            }
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        *cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let sheet = left.sheet;
                let row1 = left.row;
                let mut row2 = right.row;
                let column1 = left.column;
                let mut column2 = right.column;
                let (used_row, used_column) = self
                    .workbook
                    .worksheet(sheet)
                    .map_err(|_| {
                        CalcResult::new_error(
                            Error::ERROR,
                            *cell,
                            format!("Invalid worksheet index: '{sheet}'"),
                        )
                    })?
                    .used_area_end();
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = used_row;
                }
                if column1 == 1 && column2 == LAST_COLUMN {
                    column2 = used_column;
                }
                for row in row1..=row2 {
                    for column in column1..=column2 {
                        let cell_ref = CellReferenceIndex { sheet, row, column };
                        match self.evaluate_cell(cell_ref) {
                            CalcResult::Number(value) => values.push(value),
                            error @ CalcResult::Error { .. } => return Err(error),
                            CalcResult::EmptyCell => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
                // The clip skips empty cells Excel still walks, so an empty-cell error applies after
                // the used area: `XNPV(0.1,A:A,B:B)` is #NUM! (probes/xirr-xnpv-whole-column-nogaps A1).
                if row2 < right.row || column2 < right.column {
                    handle_empty_cell()?;
                }
            }
            CalcResult::Array(arr) => {
                for row in arr {
                    for node in row {
                        match node {
                            ArrayNode::Number(f) => values.push(f),
                            ArrayNode::Error(e) => {
                                return Err(CalcResult::new_error(
                                    e,
                                    *cell,
                                    "Error in array".to_string(),
                                ));
                            }
                            ArrayNode::Empty => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {
                handle_non_number_cell()?;
            }
        }
        Ok(values)
    }

    fn get_array_of_numbers(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true,        // accept_number_node
            || Ok(None), // Ignore empty cells
            || Ok(None), // Ignore non-number cells
        )
    }

    fn get_array_of_numbers_xpnv(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        error: Error,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true, // accept_number_node
            || {
                Err(CalcResult::new_error(
                    Error::NUM,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
            || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    fn get_array_of_numbers_xirr(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            false,            // Do not accept a single number node
            || Ok(Some(0.0)), // Treat empty cells as zero
            || {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    // NPV(rate, value1, [value2],...)
    // npv = Sum[value[i]/(1+rate)^i, {i, 1, n}]
    pub(crate) fn fn_npv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut values = Vec::new();
        for arg in &args[1..] {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Range { left, right } => {
                    let areas = [Range { left, right }];
                    if let Err(e) = self.for_each_number_in_areas(&areas, cell, |v| values.push(v))
                    {
                        return e;
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // We ignore booleans and strings
                }
            };
        }
        match compute_npv(rate, &values) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::new_error(error.0, cell, error.1),
        }
    }

    // Returns the internal rate of return for a series of cash flows represented by the numbers
    // in values.
    // These cash flows do not have to be even, as they would be for an annuity.
    // However, the cash flows must occur at regular intervals, such as monthly or annually.
    // The internal rate of return is the interest rate received for an investment consisting
    // of payments (negative values) and income (positive values) that occur at regular periods

    // IRR(values, [guess])
    pub(crate) fn fn_irr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 2 || arg_count == 0 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        match compute_irr(&values, guess) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // XNPV(rate, values, dates)
    pub(crate) fn fn_xnpv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // All three are required (docs/excel-semantics/syntax.md, "Minimum argument counts").
        let [rate, values, dates] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let rate = match self.get_number(rate, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let values = match self.get_array_of_numbers_xpnv(values, &cell, Error::NUM) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xpnv(dates, &cell, Error::VALUE) {
            Ok(s) => s,
            Err(error) => return error,
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        let first_date = dates[0];
        for date in &dates {
            if *date < MINIMUM_DATE_SERIAL_NUMBER as f64
                || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            {
                // Excel docs claim that if any number in dates is not a valid date,
                // XNPV returns the #VALUE! error value, but it seems to return #VALUE!
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid number for date".to_string(),
                );
            }
            // If any number in dates precedes the starting date, XNPV returns the #NUM! error value.
            if date < &first_date {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Date precedes the starting date".to_string(),
                );
            }
        }
        // It seems Excel returns #NUM! if rate < 0, this is only necessary if r <= -1
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate needs to be > 0".to_string());
        }
        match compute_xnpv(rate, &values, &dates) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::new_error(error, cell, message),
        }
    }

    // XIRR(values, dates, [guess])
    pub(crate) fn fn_xirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_xirr(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xirr(&args[1], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        // Excel reads an empty value or date cell as 0 and accepts date 0 and dates before the
        // first one, but is #NUM! for a negative date, a date past 9999-12-31, or when no date is
        // later than the first: {-1000,1100} on {0,45365} is 1.1^(365/45365) - 1,
        // {-1000,1100,-1000} on {45000,45365,44000} is -0.2467, and {-1000,1100} on
        // {45365,45000} is #NUM! although it has a root (financial.md, "XIRR reads empty cells as
        // 0 and accepts earlier dates"; probes/xirr-mismatched-empties, xirr-early-dates; T-1124).
        let first_date = dates[0];
        if dates
            .iter()
            .any(|date| *date < 0.0 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
        {
            return CalcResult::new_error(Error::NUM, cell, "Invalid number for date".to_string());
        }
        if !dates.iter().any(|date| *date > first_date) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "No date is later than the starting date".to_string(),
            );
        }
        match compute_xirr(&values, &dates, guess) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::Error {
                error,
                origin: cell,
                message,
            },
        }
    }

    // CUMIPMT(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ipmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // CUMPRINC(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumprinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ppmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // DB(cost, salvage, life, period, [month])
    pub(crate) fn fn_db(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let month = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f.trunc(),
                Err(s) => return s,
            }
        } else {
            12.0
        };
        if month == 12.0 && period > life
            || (period > life + 1.0)
            || month <= 0.0
            || month > 12.0
            || period <= 0.0
            || cost < 0.0
        {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        };
        if cost == 0.0 {
            return CalcResult::Number(0.0);
        }
        let result = financial_depreciation::db_depreciation(cost, salvage, life, period, month);
        CalcResult::Number(result)
    }

    fn get_yearfrac(
        &mut self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = [
            Node::NumberKind(start as f64),
            Node::NumberKind(end as f64),
            Node::NumberKind(basis),
        ];
        match self.fn_yearfrac(&args, cell) {
            CalcResult::Number(yf) => Ok(yf),
            error => Err(error),
        }
    }

    // Day-count factors shared by PRICEMAT and YIELDMAT, mirroring
    // `getMatFactors` in ExcelFinancialFunctions: all three terms share a single
    // denominator `b = DaysInYear(issue, settlement)`, and the settlement→maturity
    // span is `dim - a` (issue-anchored), not an independent year fraction.
    //
    // Returns `(a, dim, dsm)` already divided by `b`, i.e. in year units:
    //   a   = DaysBetween(issue, settlement) / b   (= YEARFRAC(issue, settlement))
    //   dim = DaysBetween(issue, maturity)   / b
    //   dsm = dim - a
    fn get_mat_factors(
        &mut self,
        issue: i64,
        settlement: i64,
        maturity: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64), CalcResult> {
        let a = self.get_yearfrac(issue, settlement, basis, cell)?;
        let dim = if basis == 1.0 {
            // Actual/Actual has a date-dependent DaysInYear, so YEARFRAC(issue,
            // maturity) would use a different denominator. Re-base onto `b` via
            // the actual day counts: dim = dim_days * a / a_days = dim_days / b.
            let a_days = (settlement - issue) as f64;
            let dim_days = (maturity - issue) as f64;
            // Callers guarantee `issue < settlement`, so `a_days > 0`; this guard
            // is defensive only.
            if a_days == 0.0 {
                return Err(CalcResult::new_error(
                    Error::DIV,
                    cell,
                    "Division by zero".to_string(),
                ));
            }
            dim_days * a / a_days
        } else {
            // Constant DaysInYear, so the common denominator is automatic.
            self.get_yearfrac(issue, maturity, basis, cell)?
        };
        Ok((a, dim, dim - a))
    }

    // FVSCHEDULE(principal, schedule)
    pub(crate) fn fn_fvschedule(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let principal = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let schedule = match self.get_array_of_numbers(&args[1], &cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let mut result = principal;
        for rate in schedule {
            result *= 1.0 + rate;
        }
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(result)
    }
}
