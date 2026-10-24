use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::num_error;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// FORECAST's data read once: the regression line's intercept and slope over the pairs
    /// of `known_y's` and `known_x's` that are both numbers, or the error the data gives.
    fn forecast_line(
        &mut self,
        known: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        // Fewer than two pairs or all x equal is #DIV/0!.
        self.paired_moments(known, cell)?
            .line()
            .ok_or_else(|| CalcResult::new_error(Error::DIV, cell, String::new()))
    }

    /// `FORECAST(x, known_y's, known_x's)` and FORECAST.LINEAR: the predicted y at `x` by
    /// simple linear regression. `x` lifts over an array and a multi-cell range, the data is
    /// read once (SPEC-0011, T-1982). An element reads `x` (numeric text and TRUE coerce), then
    /// takes the data's error: `FORECAST("a",D1:D3,A1:A3)` with D2 = `#N/A` is `#VALUE!`, while
    /// a lifted element with a good `x` is the data's `#N/A`, and unequal sizes are `#N/A` per
    /// element (probes/stat-test-lift P1-P9, P89).
    pub(crate) fn fn_forecast(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // FORECAST.LINEAR shares FORECAST's table entry (`ONE_THEN_TWO_DATA`).
        let kind = Function::Forecast;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let line = match args.get(1..) {
            Some(known) => self
                .forecast_line(known, cell)
                .map_err(CalcResult::without_message),
            None => return CalcResult::new_args_number_error(cell),
        };
        self.lift_call(kind, args, cell, &mut |model, values| {
            let x = match values.first().map(|x| model.kernel_number(x, cell)) {
                Some(Ok(x)) => x,
                Some(Err(error)) => return error,
                None => return CalcResult::new_args_number_error(cell),
            };
            match &line {
                Ok((intercept, slope)) => {
                    let result = intercept + slope * x;
                    if result.is_finite() {
                        CalcResult::Number(result)
                    } else {
                        num_error(cell)
                    }
                }
                Err(error) => error.clone(),
            }
        })
    }

    /// FORECAST.ETS, FORECAST.ETS.CONFINT and FORECAST.ETS.STAT are recognized but not
    /// implemented: every element is `#N/IMPL!`, after the lifted argument (`target_date`, or
    /// `statistic_type` at `lifted`) is read as a number, so text there is `#VALUE!`
    /// (probes/forecast-ets-longer-series K38; probes/stat-argument-modes H268-H280, T-2028).
    fn forecast_ets_lifted(
        &mut self,
        kind: Function,
        lifted: usize,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let read = values
                .get(lifted)
                .map_or(Ok(0.0), |value| model.kernel_number(value, cell));
            match read {
                Err(error) => error,
                Ok(_) => CalcResult::new_error(
                    Error::NIMPL,
                    cell,
                    "FORECAST.ETS family not implemented".to_string(),
                ),
            }
        })
    }

    pub(crate) fn fn_forecast_ets(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(Function::ForecastEts, 0, args, cell)
    }

    pub(crate) fn fn_forecast_ets_confint(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(Function::ForecastEtsConfint, 0, args, cell)
    }

    pub(crate) fn fn_forecast_ets_seasonality(
        &mut self,
        _args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        CalcResult::new_error(
            Error::NIMPL,
            cell,
            "FORECAST.ETS.SEASONALITY not implemented".to_string(),
        )
    }

    pub(crate) fn fn_forecast_ets_stat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(Function::ForecastEtsStat, 2, args, cell)
    }
}
