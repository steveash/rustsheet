use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

mod accrint;
mod amortization;
mod annuity;
mod cumulative;
mod dates;
pub(crate) use dates::{day_count_basis, days_30_360_us_ymd};
mod discount;
mod financial_bonds;
mod financial_depreciation;
pub(crate) use financial_depreciation::MAX_DEPRECIATION_PERIODS;
mod financial_util;
pub(crate) use financial_util::RATE_MAX_STEPS;
mod mirr;
mod rate_conversions;

use financial_util::{compute_irr, compute_npv, compute_xirr, compute_xnpv};

use crate::functions::lift::{num_error, number_or_error, number_result, LiftCost};
use crate::functions::Function;

/// The cash flows (or FVSCHEDULE's rates) a lifted NPV, IRR, MIRR, XNPV, XIRR or FVSCHEDULE
/// element walks, at least 1: what the call is charged per element (`LiftCost::Cells`).
fn flow_count(flows: Result<&[f64], &CalcResult>) -> u64 {
    let count = flows.map_or(1, |flows| flows.len().max(1));
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// A computation's error as the cell's error, with no message: the kernels run lifted, where an
/// element's message is dropped (`calc_result_to_array_node`), so building one only allocates
/// (SPEC-0011, T-1823).
fn formula_error(cell: CellReferenceIndex) -> impl Fn(Error) -> CalcResult {
    move |error| CalcResult::new_error(error, cell, String::new())
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
        let result = self.evaluate_node_in_context(arg, *cell);
        self.array_of_numbers_from(
            result,
            cell,
            accept_number_node,
            handle_empty_cell,
            handle_non_number_cell,
        )
    }

    /// [`Self::get_array_of_numbers_generic`] on an argument already evaluated.
    fn array_of_numbers_from(
        &mut self,
        result: CalcResult,
        cell: &CellReferenceIndex,
        accept_number_node: bool,
        handle_empty_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
        handle_non_number_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        match result {
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
                            ArrayNode::Lambda(_) => {
                                return Err(Model::lambda_as_number_error(*cell))
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

    /// XNPV's values (`error` is #NUM!) or dates (#VALUE!). A values argument that is one empty
    /// cell is a single cash flow of 0, `XNPV(1,A2,B11)` is 0 (probes/xnpv-empty-values-cell
    /// C1-C4, T-2066); empty cells in a range stay refused.
    fn get_array_of_numbers_xpnv(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        error: Error,
    ) -> Result<Vec<f64>, CalcResult> {
        let result = self.evaluate_node_in_context(arg, *cell);
        if error == Error::NUM && matches!(result, CalcResult::EmptyCell) {
            return Ok(vec![0.0]);
        }
        let refuse = |error: Error| {
            move || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            }
        };
        self.array_of_numbers_from(
            result,
            cell,
            true, // accept_number_node
            refuse(Error::NUM),
            refuse(error),
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

    /// `NPV(rate, value1, [value2], ...)`: `Sum[value[i]/(1+rate)^i, {i, 1, n}]`. The rate lifts
    /// over arrays and ranges; the values are read once, whole, and handed to every element, whose
    /// rate is checked first: a rate error wins over an error in the values, the scalar order kept
    /// from before T-1821. The values are evaluated before the rate, also when the rate is an
    /// error, since the call's charge needs their count; Excel's order is unprobed (financial.md
    /// "NPV, IRR and MIRR error order", T-2103).
    pub(crate) fn fn_npv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let [_, flows @ ..] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        if flows.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let flows = self
            .npv_cash_flows(flows, cell)
            .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Npv,
            args,
            LiftCost::Cells(flow_count(flows.as_deref())),
            cell,
            &mut |model, values| {
                let rate = match values.first() {
                    Some(rate) => model.cast_number_ref(rate, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                match (rate, &flows) {
                    (Err(error), _) => error,
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(rate), Ok(flows)) => {
                        number_result(compute_npv(rate, flows).map_err(|error| error.0), cell)
                    }
                }
            },
        )
    }

    /// The cash flows of NPV, read from its value arguments in order.
    fn npv_cash_flows(
        &mut self,
        flows: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        for arg in flows {
            // Excel: an argument read as a reference skips the text and booleans of its cells,
            // however the reference is written: `B1`, `(B1)`, `@B1:B1`, `IF(TRUE,B1)`,
            // `INDEX(B1:B2,1)`, `CHOOSE(1,B1)`, `OFFSET(B1,0,0)`, `INDIRECT("B1")`, `LET`, a
            // name for a cell, `Sheet1:Sheet1!B1`; a union is its areas as written (ironcalc-calc
            // FINANCIAL/NPV.xlsx K5, L5; probes/npv-fvschedule-typed A7, A10, A11;
            // probes/npv-reference-forms). Any other value is typed: it is coerced
            // (`NPV(0.1,-10000,"2750",TRUE,4250)` is -3914.6233180793665) and text that is no
            // number is #VALUE! (probes/fin-argument-modes H368-H371; probes/npv-fvschedule-typed
            // A5, A6, A8, A9; probes/npv-fvschedule-edges A1, A5).
            // docs/excel-semantics/financial.md "NPV and FVSCHEDULE arguments".
            match self
                .evaluate_reference_arg(arg, cell)
                .into_ranges_or_value()
            {
                Ok(areas) => self.for_each_number_in_areas(&areas, cell, |v| values.push(v))?,
                Err(CalcResult::Number(value)) => values.push(value),
                Err(direct @ (CalcResult::String(_) | CalcResult::Boolean(_))) => {
                    values.push(self.cast_number_ref(&direct, cell)?);
                }
                // A written empty argument is a cash flow of 0 (probes/npv-fvschedule-edges A2).
                Err(CalcResult::EmptyArg) => values.push(0.0),
                // An array is read whole: its numbers are the cash flows, its text and booleans
                // are skipped and its first error is the result (H368; probes/npv-reference-forms
                // A7, A8, A13).
                Err(array @ CalcResult::Array(_)) => values.extend(self.array_of_numbers_from(
                    array,
                    &cell,
                    true,
                    || Ok(None),
                    || Ok(None),
                )?),
                Err(error @ CalcResult::Error { .. }) => return Err(error),
                Err(_) => {}
            }
        }
        Ok(values)
    }

    /// `IRR(values, [guess])`: the internal rate of return of cash flows at regular intervals.
    /// The guess lifts over arrays and ranges; the values are read once, whole, and an error in
    /// them is every element's (T-1821). Each element may run the solver's every pass over the
    /// flows, so the call is charged their count per element, weighted by the table's
    /// `IRR_WEIGHT`.
    pub(crate) fn fn_irr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (Some(values), 1..=2) = (args.first(), args.len()) else {
            return CalcResult::new_args_number_error(cell);
        };
        // Excel: an error inside an array or a referenced range is #VALUE!, whatever its kind,
        // so the original error is dropped. An error argument that is not a single cell
        // passes through (docs/excel-semantics/financial.md, "IRR over an error element";
        // probes/fin-irr-error-array).
        let flows = match self.evaluate_node_in_context(values, cell) {
            error @ CalcResult::Error { .. }
                if !matches!(values.ungrouped(), Node::ReferenceKind { .. }) =>
            {
                Err(error)
            }
            result => self
                .array_of_numbers_from(result, &cell, true, || Ok(None), || Ok(None))
                .map_err(|_| CalcResult::new_error(Error::VALUE, cell, String::new())),
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Irr,
            args,
            LiftCost::Cells(flow_count(flows.as_deref())),
            cell,
            &mut |model, values| {
                let flows = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
                };
                let guess = match model.cast_number_or(values.get(1), 0.1, cell) {
                    Ok(guess) => guess,
                    Err(error) => return error,
                };
                number_result(compute_irr(flows, guess).map_err(|error| error.0), cell)
            },
        )
    }

    /// `XNPV(rate, values, dates)`: the rate lifts over arrays (a multi-cell range is refused);
    /// the values and dates are read once, whole, and handed to every element, whose rate is
    /// checked first, the scalar order (T-1843).
    pub(crate) fn fn_xnpv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // All three are required (docs/excel-semantics/syntax.md, "Minimum argument counts").
        let [_, values, dates] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        // The dates are truncated and checked once, not per element: each element is charged
        // only the values' count.
        let flows = self
            .get_array_of_numbers_xpnv(values, &cell, Error::NUM)
            .and_then(|values| {
                let dates = self.get_array_of_numbers_xpnv(dates, &cell, Error::VALUE)?;
                let dates = xnpv_dates(&values, &dates);
                Ok((values, dates))
            })
            .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Xnpv,
            args,
            LiftCost::Cells(flow_count(
                flows.as_ref().map(|(values, _)| values.as_slice()),
            )),
            cell,
            &mut |model, values| {
                let rate = match values.first() {
                    Some(rate) => model.cast_number_ref(rate, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                match (rate, &flows) {
                    (Err(error), _) => error,
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(rate), Ok((values, dates))) => {
                        let xnpv = dates.as_ref().map_err(Clone::clone).and_then(|dates| {
                            // It seems Excel returns #NUM! if rate < 0, this is only necessary
                            // if r <= -1
                            if rate <= 0.0 {
                                return Err(Error::NUM);
                            }
                            compute_xnpv(rate, values, dates).map_err(|error| error.0)
                        });
                        number_result(xnpv, cell)
                    }
                }
            },
        )
    }

    /// `XIRR(values, dates, [guess])`: the guess lifts over arrays (a multi-cell range is
    /// refused); the values and dates are read once, whole, and an error in them is every
    /// element's, before the guess's, the scalar order (T-1843). Each element may run the bracket
    /// walks and the bisection over the flows, so the call is charged their count per element,
    /// weighted by the table's `XIRR_WEIGHT`.
    pub(crate) fn fn_xirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let ([values, dates] | [values, dates, _]) = args else {
            return CalcResult::new_args_number_error(cell);
        };
        // The dates are truncated and checked once, not per element, as XNPV's.
        let flows = self
            .get_array_of_numbers_xirr(values, &cell)
            .and_then(|values| {
                let dates = self.get_array_of_numbers_xirr(dates, &cell)?;
                let dates = xirr_dates(&values, &dates);
                Ok((values, dates))
            })
            .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Xirr,
            args,
            LiftCost::Cells(flow_count(
                flows.as_ref().map(|(values, _)| values.as_slice()),
            )),
            cell,
            &mut |model, arguments| {
                let (values, dates) = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
                };
                match model.cast_number_or(arguments.get(2), 0.1, cell) {
                    Ok(guess) => {
                        let xirr = dates.as_ref().map_err(Clone::clone).and_then(|dates| {
                            compute_xirr(values, dates, guess).map_err(|error| error.0)
                        });
                        number_result(xirr, cell)
                    }
                    Err(error) => error,
                }
            },
        )
    }

    fn get_yearfrac(
        &mut self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let (days, year) = self.year_parts(start, end, basis, cell)?;
        // Signed: basis 0 counts 1900-02-28 to the fictitious 1900-02-29 as -1 day, so
        // `ACCRINTM(59,60,0.1,1000,0)` is negative (probes/discount-serial-zero).
        Ok(days / year)
    }

    /// YEARFRAC's `(day count, year length)` from `start` to `end` (`Model::year_fraction_parts`),
    /// `#NUM!` for a basis outside 0-4 or a date outside the calendar.
    fn year_parts(
        &self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        day_count_basis(basis)
            .and_then(|basis| self.year_fraction_parts(start as f64, end as f64, basis))
            .ok_or_else(|| num_error(cell))
    }

    /// Day-count parts shared by PRICEMAT and YIELDMAT, as Excel's `getMatFactors`: one year
    /// length `b`, the year of issue to settlement (so basis 1 reads the year length of that
    /// span), and the issue-to-maturity and issue-to-settlement day counts, with
    /// settlement-to-maturity their difference (financial.md "PRICEMAT and YIELDMAT count the
    /// settlement-to-maturity days").
    ///
    /// Returns `(b, dim, a, dsm)` in days.
    fn get_mat_factors(
        &self,
        issue: i64,
        settlement: i64,
        maturity: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64, f64, f64), CalcResult> {
        let (a, b) = self.year_parts(issue, settlement, basis, cell)?;
        let (dim, _) = self.year_parts(issue, maturity, basis, cell)?;
        Ok((b, dim, a, dim - a))
    }

    /// `FVSCHEDULE(principal, schedule)`: the principal lifts over arrays (a multi-cell range is
    /// refused); the schedule is read once, whole, and handed to every element, whose principal
    /// is checked first, the scalar order (T-1843).
    pub(crate) fn fn_fvschedule(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let [_, schedule] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        // Excel: text in the schedule, in a range or an array literal, is #VALUE!
        // (probes/fin-argument-modes H382-H384).
        // Typed text that reads as a number is a one-rate schedule: `FVSCHEDULE(1000,"0.1")`,
        // `("0.1")` and `T("0.1")` are 1100, `"x"` and `TRUE` are #VALUE!
        // (probes/npv-fvschedule-typed A12, A13; probes/npv-fvschedule-edges A3, A4, A6).
        let not_a_number = || CalcResult::new_error(Error::VALUE, cell, String::new());
        let schedule = match self.evaluate_node_in_context(schedule, cell) {
            CalcResult::String(text) => self
                .cast_number(&text)
                .map(|rate| vec![rate])
                .ok_or_else(not_a_number),
            result => {
                self.array_of_numbers_from(result, &cell, true, || Ok(None), || Err(not_a_number()))
            }
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Fvschedule,
            args,
            LiftCost::Cells(flow_count(schedule.as_deref())),
            cell,
            &mut |model, values| {
                // A boolean principal is #VALUE!, though a text number is converted
                // (probes/fin-fvschedule A18, A9).
                let principal = match values.first() {
                    Some(principal) => model.cast_number_no_bools_ref(principal, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                match (principal, &schedule) {
                    (Err(error), _) => error,
                    (Ok(_), Err(error)) => error.clone(),
                    (Ok(principal), Ok(schedule)) => {
                        let result = schedule
                            .iter()
                            .fold(principal, |result, rate| result * (1.0 + rate));
                        if result.is_infinite() {
                            return CalcResult::new_error(Error::DIV, cell, String::new());
                        }
                        CalcResult::Number(result)
                    }
                }
            },
        )
    }
}

/// XNPV's and XIRR's dates truncated to whole days, with the first: `#NUM!` when there are none
/// or when the values and dates differ in count. The errors carry no message: the kernels run
/// lifted, where an element's message is dropped (SPEC-0011).
fn whole_day_dates(values: &[f64], dates: &[f64]) -> Result<(Vec<f64>, f64), Error> {
    let dates: Vec<f64> = dates.iter().map(|date| date.floor()).collect();
    match dates.first() {
        Some(&first_date) if values.len() == dates.len() => Ok((dates, first_date)),
        _ => Err(Error::NUM),
    }
}

/// XNPV's dates truncated to whole days, with the scalar call's checks in its order: the counts,
/// then the dates (the rate is checked after them, per element).
fn xnpv_dates(values: &[f64], dates: &[f64]) -> Result<Vec<f64>, Error> {
    let (dates, first_date) = whole_day_dates(values, dates)?;
    for date in &dates {
        // Excel docs claim that if any number in dates is not a valid date,
        // XNPV returns the #VALUE! error value, but it seems to return #NUM!.
        // If any number in dates precedes the starting date, XNPV returns the #NUM! error value.
        if *date < MINIMUM_DATE_SERIAL_NUMBER as f64
            || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || *date < first_date
        {
            return Err(Error::NUM);
        }
    }
    Ok(dates)
}

/// XIRR's dates truncated to whole days, with the scalar call's checks in its order: the counts,
/// then the dates (the solver's own checks follow, per element).
fn xirr_dates(values: &[f64], dates: &[f64]) -> Result<Vec<f64>, Error> {
    let (dates, first_date) = whole_day_dates(values, dates)?;
    // Excel reads an empty value or date cell as 0 and accepts date 0 and dates before the
    // first one, but is #NUM! for a negative date, a date past 9999-12-31, or when no date is
    // later than the first: {-1000,1100} on {0,45365} is 1.1^(365/45365) - 1,
    // {-1000,1100,-1000} on {45000,45365,44000} is -0.2467, and {-1000,1100} on
    // {45365,45000} is #NUM! although it has a root (financial.md, "XIRR reads empty cells as
    // 0 and accepts earlier dates"; probes/xirr-mismatched-empties, xirr-early-dates; T-1124).
    if dates
        .iter()
        .any(|date| *date < 0.0 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
        || !dates.iter().any(|date| *date > first_date)
    {
        return Err(Error::NUM);
    }
    Ok(dates)
}
