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
pub(crate) use amortization::{amortization_values_valid, asset_life_refused};
pub(crate) use dates::{day_count_basis, days_30_360_us_ymd};
mod discount;
mod financial_bonds;
mod financial_depreciation;
pub(crate) use financial_depreciation::MAX_DEPRECIATION_PERIODS;
mod financial_util;
mod growth;
mod mirr;
mod rate_conversions;

use financial_util::{compute_irr, compute_xirr, compute_xnpv, excel_npv};

use crate::functions::lift::{num_error, number_or_error, number_result, LiftCost};
use crate::functions::util::is_marked_number;
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

/// Which XNPV argument [`Model::xnpv_numbers`] reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum XnpvArgument {
    Values,
    Dates,
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
                let column1 = left.column;
                let (row2, column2) = self.flow_range_end(&left, &right).ok_or_else(|| {
                    CalcResult::new_error(
                        Error::ERROR,
                        *cell,
                        format!("Invalid worksheet index: '{sheet}'"),
                    )
                })?;
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
                            ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
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

    /// XNPV's values or dates. A non-number cell refuses the values with `#NUM!` and the dates
    /// with `#VALUE!`. A values argument that is one empty cell is a single cash flow of 0,
    /// `XNPV(1,A2,B11)` is 0 (probes/xnpv-empty-values-cell C1-C4, T-2599); empty cells in a
    /// larger range stay refused.
    fn xnpv_numbers(
        &mut self,
        result: CalcResult,
        cell: &CellReferenceIndex,
        kind: XnpvArgument,
    ) -> Result<Vec<f64>, CalcResult> {
        let refuse = |error: Error| {
            move || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            }
        };
        if self.empty_cell(&result) {
            // One empty cell, `A2` or `A2:A2`, is a 0 cash flow but no date: `XNPV(1,A2:A2,B11)`
            // is 0 and `XNPV(1,B11,A2)` is `#NUM!` (probes/xnpv-empty-cell-forms C1-C5).
            return match kind {
                XnpvArgument::Values => Ok(vec![0.0]),
                XnpvArgument::Dates => Err(CalcResult::new_error(
                    Error::NUM,
                    *cell,
                    "Expected number".to_string(),
                )),
            };
        }
        let non_number = match kind {
            XnpvArgument::Values => Error::NUM,
            XnpvArgument::Dates => Error::VALUE,
        };
        self.array_of_numbers_from(
            result,
            cell,
            true, // accept_number_node
            refuse(Error::NUM),
            refuse(non_number),
        )
    }

    fn xirr_numbers(
        &mut self,
        arg: CalcResult,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.array_of_numbers_from(
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

    /// XIRR's size checks (T-2428): one value and one date is `#N/A`
    /// (`XIRR(Data!A9:A9,Data!E9:E9)`, gen-m9/30e20ba3302fea8b A110; also with a FALSE date,
    /// bc04c1f478c9896c A24), a single date that is text or a boolean `#VALUE!`
    /// (`XIRR({-100,#N/A,60},<FALSE>)`, 1cd3096e3e87c965 A48), then values and dates of
    /// different counts `#NUM!`, whatever the cells hold (`XIRR({-100,#N/A,60},<5 dates>)`,
    /// b72d952c5f52cb4d A1). An argument that is no range, array or scalar is left to the
    /// number reading.
    fn xirr_size_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let value_count = self.flow_element_count(values)?;
        let date_count = self.flow_element_count(dates)?;
        let error = if value_count == 1 && date_count == 1 {
            // A text, boolean or empty cell keeps the old `#VALUE!` (gen-m9/097af1ca33638670
            // A255, a91bad4e84f49739 A203, bc04c1f478c9896c A318).
            if self.non_number_cell(values) || self.non_number_cell(dates) {
                return None;
            }
            Error::NA
        } else if date_count == 1 && self.non_numeric_text_or_boolean_cell(dates) {
            // gen-m9/1cd3096e3e87c965 A48. An empty or numeric-text date cell is left to the
            // count (`XIRR(Data!B6:C11,<empty>,0.5)` and `XIRR(Data!A6:B12,<"1">)` are `#NUM!`,
            // 89f0aee953a55a45 A330, c0681a8b150ce301 A266; T-2611).
            Error::VALUE
        } else if value_count == 1 && self.non_numeric_text_or_boolean_cell(values) {
            // One text or boolean value cell against several dates is read first, `#VALUE!`
            // (gen-m9/6376e27be7a05871 A67), but one empty value cell is the count's `#NUM!`
            // (`XIRR(Data!D10:D10,Data!B7:E8)`, 1dce4774f5c0e736 A91; T-2611); a larger values
            // range of another count is `#NUM!` whatever it holds (0a957119475760c5 A47,
            // bbdfc06cae0e0d71 A44).
            return None;
        } else if value_count != date_count {
            Error::NUM
        } else {
            return None;
        };
        Some(CalcResult::new_error(error, cell, String::new()))
    }

    /// XNPV's checks before it reads a number (T-2428), after the rate's, in Excel's order:
    ///
    /// 1. One values cell that is non-numeric text or a boolean is `#VALUE!`, before an error
    ///    dates argument (`XNPV(-1,<FALSE>,<#N/A>)`, gen-m9/3aa168259335be54 A334). Numeric text
    ///    is not refused: `XNPV(0.1,<"1.5">,Data!C5:C6)` is the count's `#NUM!` (81bcdc0bec0c8e6b
    ///    A37, through DB).
    /// 2. An error dates argument is that error, before the values' cells
    ///    (`XNPV(0.05,Data!B1:E2,<#N/A>)`, 5c6a23c4f6597f1c A111).
    /// 3. One dates cell that is non-numeric text or a boolean is `#VALUE!`
    ///    (d0854366830dcdee A274).
    /// 4. Values and dates of different counts are `#NUM!` whatever they hold
    ///    (`XNPV(0.1,{-100,#N/A,60},<5 dates>)`, bc04c1f478c9896c A313;
    ///    `XNPV(0.2,0.3,Data!C9:D12)`, 1cd3096e3e87c965 A217).
    ///
    /// An empty cell is left to the reading: one empty values cell is a 0 cash flow (T-2599).
    fn xnpv_size_error(
        &mut self,
        values: &CalcResult,
        dates: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        if self.non_numeric_text_or_boolean_cell(values) {
            return Some(CalcResult::new_error(Error::VALUE, cell, String::new()));
        }
        if matches!(dates, CalcResult::Error { .. }) {
            return (!matches!(values, CalcResult::Error { .. })).then(|| dates.clone());
        }
        if self.non_numeric_text_or_boolean_cell(dates) {
            return Some(CalcResult::new_error(Error::VALUE, cell, String::new()));
        }
        let value_count = self.flow_element_count(values)?;
        let date_count = self.flow_element_count(dates)?;
        (value_count != date_count).then(|| CalcResult::new_error(Error::NUM, cell, String::new()))
    }

    /// The number of elements XIRR and XNPV read from an evaluated argument: a range's cells (a
    /// whole column or row trimmed to the used area, as [`Self::array_of_numbers_from`]), an
    /// array's elements, 1 for a scalar; `None` for an error or anything else.
    fn flow_element_count(&self, arg: &CalcResult) -> Option<usize> {
        match arg {
            CalcResult::Range { left, right } if left.sheet == right.sheet => {
                let (row2, column2) = self.flow_range_end(left, right)?;
                let rows = usize::try_from(row2 - left.row + 1).ok()?;
                let columns = usize::try_from(column2 - left.column + 1).ok()?;
                Some(rows.saturating_mul(columns))
            }
            CalcResult::Array(array) => Some(
                array
                    .len()
                    .saturating_mul(array.first().map_or(0, Vec::len)),
            ),
            CalcResult::Number(_)
            | CalcResult::String(_)
            | CalcResult::Boolean(_)
            | CalcResult::EmptyCell => Some(1),
            _ => None,
        }
    }

    /// A range's last row and column with a whole column or row trimmed to the used area, as
    /// [`Self::array_of_numbers_from`] and the XIRR and XNPV size checks read it; `None` for a
    /// missing sheet.
    fn flow_range_end(
        &self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
    ) -> Option<(i32, i32)> {
        let (used_row, used_column) = self.workbook.worksheet(left.sheet).ok()?.used_area_end();
        let row2 = if left.row == 1 && right.row == LAST_ROW {
            used_row
        } else {
            right.row
        };
        let column2 = if left.column == 1 && right.column == LAST_COLUMN {
            used_column
        } else {
            right.column
        };
        Some((row2, column2))
    }

    /// Whether a one-cell range or scalar is text, a boolean or nothing.
    ///
    /// An array element is not a cell: `XIRR(D2,BYROW(D9,LAMBDA(x,ISBLANK(x))))` is `#N/A`
    /// (gen-m9/bc04c1f478c9896c A24).
    fn non_number_cell(&mut self, arg: &CalcResult) -> bool {
        matches!(
            *self.single_cell_value(arg),
            CalcResult::String(_) | CalcResult::Boolean(_) | CalcResult::EmptyCell
        )
    }

    /// Whether a one-cell range or scalar is empty.
    fn empty_cell(&mut self, arg: &CalcResult) -> bool {
        matches!(*self.single_cell_value(arg), CalcResult::EmptyCell)
    }

    /// Whether a one-cell range or scalar is a boolean or text that does not read as a number.
    fn non_numeric_text_or_boolean_cell(&mut self, arg: &CalcResult) -> bool {
        match &*self.single_cell_value(arg) {
            CalcResult::String(text) => self.cast_number(text).is_none(),
            CalcResult::Boolean(_) => true,
            _ => false,
        }
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
                    (Ok(rate), Ok(flows)) => number_result(excel_npv(rate, flows), cell),
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
        // Excel: an error inside a range or an array is the result only when no direct argument
        // errs, whatever their order: `NPV(0.1,{1,#DIV/0!},"x")` is #VALUE!, `NPV(0.1,Q1:Q2,NA())`
        // over a #DIV/0! cell #N/A, and the first range or array error wins among them
        // (probes/npv-power-edges A36-A51, npv-power-edges-2 A31-A40). docs/excel-semantics/
        // financial.md "NPV's error order".
        let mut deferred = None;
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
                Ok(areas) => {
                    if deferred.is_none() {
                        if let Err(error) =
                            self.for_each_number_in_areas(&areas, cell, |v| values.push(v))
                        {
                            deferred = Some(error);
                        }
                    }
                }
                Err(CalcResult::Number(value)) => values.push(value),
                Err(direct @ (CalcResult::String(_) | CalcResult::Boolean(_))) => {
                    values.push(self.cast_number_ref(&direct, cell)?);
                }
                // A written empty argument is a cash flow of 0 (probes/npv-fvschedule-edges A2).
                Err(CalcResult::EmptyArg) => values.push(0.0),
                // An array is read whole: its numbers are the cash flows, its text and booleans
                // are skipped and its first error is deferred like a range's (H368;
                // probes/npv-reference-forms A7, A8, A13).
                Err(array @ CalcResult::Array(_)) => {
                    if deferred.is_none() {
                        match self.array_of_numbers_from(
                            array,
                            &cell,
                            true,
                            || Ok(None),
                            || Ok(None),
                        ) {
                            Ok(numbers) => values.extend(numbers),
                            Err(error) => deferred = Some(error),
                        }
                    }
                }
                Err(error @ CalcResult::Error { .. }) => return Err(error),
                Err(_) => {}
            }
        }
        deferred.map_or(Ok(values), Err)
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
        // probes/fin-irr-error-array). That error comes before the guess's; the #VALUE! of an
        // error element, or of a number straight from `CELL("row"|"col",..)` or ERROR.TYPE
        // (where any other scalar flow is `#NUM!`), comes after it: `IRR(1/0,NA())` is
        // `#DIV/0!`, and so are `IRR({1,"a"}+0,1/0)` and `IRR(CELL("row",D2),1/0)`
        // (financial.md "IRR's error order", probes/t2728-irr-guess-ref-mark, T-2728).
        let evaluated = self.evaluate_node_in_context(values, cell);
        let flows_error_first = matches!(evaluated, CalcResult::Error { .. })
            && !matches!(*values.ungrouped_at(cell.row), Node::ReferenceKind { .. });
        let flows = if flows_error_first {
            Err(evaluated)
        } else if is_marked_number(values, &evaluated, cell.row) {
            Err(CalcResult::new_error(Error::VALUE, cell, String::new()))
        } else {
            self.array_of_numbers_from(evaluated, &cell, true, || Ok(None), || Ok(None))
                .map_err(|_| CalcResult::new_error(Error::VALUE, cell, String::new()))
        }
        .map_err(CalcResult::without_message);
        self.lift_call_costed(
            Function::Irr,
            args,
            LiftCost::Cells(flow_count(flows.as_deref())),
            cell,
            &mut |model, values| {
                if let (true, Err(error)) = (flows_error_first, &flows) {
                    return error.clone();
                }
                let guess = match model.cast_number_or(values.get(1), 0.1, cell) {
                    Ok(guess) => guess,
                    Err(error) => return error,
                };
                let flows = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
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
        let values = self.evaluate_node_in_context(values, cell);
        let dates = self.evaluate_node_in_context(dates, cell);
        let size_error = self.xnpv_size_error(&values, &dates, cell);
        // The dates are truncated and checked once, not per element: each element is charged
        // only the values' count.
        let flows = self
            .xnpv_numbers(values, &cell, XnpvArgument::Values)
            .and_then(|values| {
                let dates = self.xnpv_numbers(dates, &cell, XnpvArgument::Dates)?;
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
                // A boolean rate, a cell or an array element, is `#VALUE!` (`XNPV(<TRUE>,..)`,
                // gen-m9/1dce4774f5c0e736 A22; `XNPV(SORT(<FALSE>),..)`, 5c6a23c4f6597f1c A258).
                let rate = match values.first() {
                    Some(rate) => model.cast_number_no_bools_ref(rate, cell),
                    None => Err(CalcResult::new_args_number_error(cell)),
                };
                if let (Ok(_), Some(size_error)) = (&rate, &size_error) {
                    return size_error.clone();
                }
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
        let values = self.evaluate_node_in_context(values, cell);
        let dates = self.evaluate_node_in_context(dates, cell);
        // Excel checks the sizes before it reads a number, and only an error dates or guess
        // comes first (financial.md "XIRR and XNPV under the gen-m9 statistical cells" and
        // "Financial functions in the gen-m9 financial sweep (T-2257)", T-2428).
        let size_error = if matches!(dates, CalcResult::Error { .. }) {
            None
        } else {
            self.xirr_size_error(&values, &dates, cell)
        };
        // The dates are truncated and checked once, not per element, as XNPV's. An error values
        // argument comes first, then an error dates argument, before the values' cells are read
        // (T-2611: gen-m9/67a4c03afd63f988 A388 is the dates' `#DIV/0!` beside text values;
        // 79d87fc527927626 A228 is the values' `#VALUE!` beside a `#DIV/0!` dates).
        let values_error = matches!(values, CalcResult::Error { .. })
            .then(|| values.clone().without_message());
        let dates_error = match (&values, &dates) {
            (CalcResult::Error { .. }, _) => None,
            (_, CalcResult::Error { .. }) => Some(dates.clone()),
            _ => None,
        };
        let flows = dates_error
            .map_or_else(|| self.xirr_numbers(values, &cell), Err)
            .and_then(|values| {
                let dates = self.xirr_numbers(dates, &cell)?;
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
                // An error values argument comes first; then the guess: a guess that is an error,
                // or text or a boolean (`#VALUE!`), is that error before the dates' error, the size
                // checks and the values' cells (T-2611: `XIRR(1E+21,<#VALUE!>,<#NUM!>)` is `#NUM!`,
                // gen-m9/1dce4774f5c0e736 A189; `XIRR(<4 values>,<16 dates>,<#N/A>)` `#N/A`,
                // 1b60bf99d71424e7 A29; a TRUE guess beside 6 values and 3 dates `#VALUE!`,
                // b6a02893a8d6b151 A226).
                if let Some(error) = &values_error {
                    return error.clone();
                }
                // Excel: a written empty guess (`XIRR(v,d,)`) is the default 0.1, but an empty
                // cell is guess 0; IRR and RATE read both as 0 (financial.md "XIRR (and IRR): an
                // empty guess argument is the default 0.1 for XIRR, guess 0 for IRR";
                // probes/xirr-irr-empty-guess).
                let guess = arguments
                    .get(2)
                    .filter(|guess| !matches!(guess, CalcResult::EmptyArg));
                let guess = match model.cast_number_or(guess, 0.1, cell) {
                    Ok(guess) => guess,
                    Err(error) => return error,
                };
                // A size error stands only beside a number guess: numeric text keeps the old
                // reading. A written empty guess or an empty cell is a number guess here: both
                // give `#NUM!` over text or boolean flows
                // (probes/xirr-size-mismatch-empty-guess G21-G36).
                let number_guess = matches!(
                    arguments.get(2),
                    None | Some(
                        CalcResult::Number(_) | CalcResult::EmptyArg | CalcResult::EmptyCell
                    )
                );
                if let (true, Some(size_error)) = (number_guess, &size_error) {
                    return size_error.clone();
                }
                let (values, dates) = match &flows {
                    Ok(flows) => flows,
                    Err(error) => return error.clone(),
                };
                let xirr = dates.as_ref().map_err(Clone::clone).and_then(|dates| {
                    compute_xirr(values, dates, guess).map_err(|error| error.0)
                });
                number_result(xirr, cell)
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
            // One empty cell is an empty schedule, the principal unchanged: `FVSCHEDULE(6527,D1)`
            // is 6527 with D1 empty, also through `IF(..,B3,B9)` (gen-m9/30e20ba3302fea8b A250,
            // 6376e27be7a05871 A140; T-2428).
            CalcResult::EmptyCell => Ok(Vec::new()),
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
    // A zero first value skips the later-date check: `{0,-1000,1100}` on 46000, 45000, 45365 is
    // 2.98e-9, and so are equal dates (probes/xirr-span-zero-first A28-A30, xirr-zero-first-bracket
    // A8-A13; gen-m9/e168be7f6d3f84c2 A125; T-2329).
    let first_value_is_zero = values.first() == Some(&0.0);
    if dates
        .iter()
        .any(|date| *date < 0.0 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64)
        || (!first_value_is_zero && !dates.iter().any(|date| *date > first_date))
    {
        return Err(Error::NUM);
    }
    Ok(dates)
}
