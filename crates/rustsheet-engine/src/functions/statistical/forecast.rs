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
        let (_, _, values_y, values_x) = self.fn_get_two_matrices(known, cell)?;
        let error = |error| CalcResult::new_error(error, cell, String::new());

        let mut n = 0.0_f64;
        let mut sum_x = 0.0_f64;
        let mut sum_y = 0.0_f64;
        let mut sum_x2 = 0.0_f64;
        let mut sum_xy = 0.0_f64;

        for (y, xi) in values_y.iter().zip(&values_x) {
            if let (Some(y), Some(xi)) = (y, xi) {
                n += 1.0;
                sum_x += xi;
                sum_y += y;
                sum_x2 += xi * xi;
                sum_xy += xi * y;
            }
        }

        // At least two numeric data points.
        if n < 2.0 {
            return Err(error(Error::DIV));
        }

        // All x values equal.
        let denom = n * sum_x2 - sum_x * sum_x;
        if denom == 0.0 || !denom.is_finite() {
            return Err(error(Error::DIV));
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denom;
        let intercept = (sum_y - slope * sum_x) / n;
        Ok((intercept, slope))
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

    // The FORECAST.ETS family (exponential triple smoothing) is recognized but
    // not yet implemented; it returns the #N/IMPL! error.
    pub(crate) fn fn_forecast_ets(
        &mut self,
        _args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        CalcResult::new_error(
            Error::NIMPL,
            cell,
            "FORECAST.ETS not implemented".to_string(),
        )
    }

    pub(crate) fn fn_forecast_ets_confint(
        &mut self,
        _args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        CalcResult::new_error(
            Error::NIMPL,
            cell,
            "FORECAST.ETS.CONFINT not implemented".to_string(),
        )
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
        _args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        CalcResult::new_error(
            Error::NIMPL,
            cell,
            "FORECAST.ETS.STAT not implemented".to_string(),
        )
    }
}
