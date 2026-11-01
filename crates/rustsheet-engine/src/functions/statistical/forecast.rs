use std::collections::HashMap;

use super::ets::{
    self, build_series, completion_setting, detect_season, Aggregate, Point, SeriesOptions,
};
use crate::expressions::parser::ArrayNode;
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

    /// The cells of `values` or `timeline` in reading order (an error cell is the error), or `#NUM!`
    /// for more than [`ets::MAX_POINTS`] cells, read before any is evaluated.
    fn ets_points(&mut self, value: &CalcResult) -> Result<Vec<Point>, Error> {
        Ok(match value {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(Error::VALUE);
                }
                let rows = usize::try_from(right.row - left.row + 1).unwrap_or(0);
                let columns = usize::try_from(right.column - left.column + 1).unwrap_or(0);
                if rows.saturating_mul(columns) > ets::MAX_POINTS {
                    return Err(Error::NUM);
                }
                let mut points = Vec::with_capacity(rows * columns);
                for row in left.row..=right.row {
                    for column in left.column..=right.column {
                        let position = CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        };
                        points.push(match self.evaluate_cell(position) {
                            CalcResult::Number(number) => Point::Number(number),
                            CalcResult::EmptyCell | CalcResult::EmptyArg => Point::Blank,
                            CalcResult::Error { error, .. } => return Err(error),
                            _ => Point::Other,
                        });
                    }
                }
                points
            }
            CalcResult::Array(rows) => {
                let mut points = Vec::new();
                for node in rows.iter().flatten() {
                    points.push(match node {
                        ArrayNode::Number(number) => Point::Number(*number),
                        ArrayNode::Empty => Point::Blank,
                        ArrayNode::Error(error) => return Err(error.clone()),
                        _ => Point::Other,
                    });
                    if points.len() > ets::MAX_POINTS {
                        return Err(Error::NUM);
                    }
                }
                points
            }
            CalcResult::Number(number) => vec![Point::Number(*number)],
            CalcResult::EmptyCell | CalcResult::EmptyArg => vec![Point::Blank],
            CalcResult::Error { error, .. } => return Err(error.clone()),
            _ => vec![Point::Other],
        })
    }

    /// The FORECAST.ETS family's call: the argument step Excel takes before it forecasts, then
    /// `element` for each element. The step reads the arguments in order, and the first that fails
    /// decides: a scalar argument's error, or text that is not a number (`#VALUE!`), or an error
    /// given directly as `values` or `timeline`. Every scalar argument lifts over an array and a
    /// multi-cell range (statistics.md "FORECAST.ETS family on excel-graph: argument errors, text
    /// and lifting decide before the #N/A (T-2255)"; probes/forecast-ets-longer-series K38;
    /// probes/stat-argument-modes H268-H280; T-2028, T-2424). `whole` are the positions of
    /// `values` and `timeline`, read here once; `element` gets the scalar arguments as numbers by
    /// position (`None` at `whole` and past the arguments given).
    fn forecast_ets_lifted(
        &mut self,
        kind: Function,
        whole: [usize; 2],
        args: &[Node],
        cell: CellReferenceIndex,
        element: fn(&mut EtsData, &[Option<f64>]) -> Result<f64, Error>,
    ) -> CalcResult {
        let mut direct_errors: [Option<CalcResult>; 2] = [None, None];
        let mut points: [Result<Vec<Point>, Error>; 2] = [Ok(Vec::new()), Ok(Vec::new())];
        for ((slot, read), &index) in direct_errors.iter_mut().zip(&mut points).zip(&whole) {
            if let Some(node) = args.get(index) {
                let value = self.evaluate_node_with_reference(node, cell);
                if value.is_error() {
                    *slot = Some(value);
                } else {
                    *read = self.ets_points(&value);
                }
            }
        }
        let [values, timeline] = points;
        let mut data = EtsData::new(values, timeline);
        self.lift_call(kind, args, cell, &mut |model, values| {
            let mut settings = vec![None; values.len()];
            for (index, value) in values.iter().enumerate() {
                if let Some(position) = whole.iter().position(|&w| w == index) {
                    if let Some(error) = &direct_errors[position] {
                        return error.clone();
                    }
                } else {
                    match model.kernel_number(value, cell) {
                        Ok(number) => settings[index] = Some(number),
                        Err(error) => return error,
                    }
                }
            }
            match element(&mut data, &settings) {
                Ok(number) if number.is_finite() => CalcResult::Number(number),
                Ok(_) => num_error(cell),
                Err(Error::NIMPL) => CalcResult::new_error(
                    Error::NIMPL,
                    cell,
                    "FORECAST.ETS family not implemented".to_string(),
                ),
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            }
        })
    }

    pub(crate) fn fn_forecast_ets(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(Function::ForecastEts, [1, 2], args, cell, not_implemented)
    }

    pub(crate) fn fn_forecast_ets_confint(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(
            Function::ForecastEtsConfint,
            [1, 2],
            args,
            cell,
            not_implemented,
        )
    }

    pub(crate) fn fn_forecast_ets_seasonality(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(
            Function::ForecastEtsSeasonality,
            [0, 1],
            args,
            cell,
            seasonality_element,
        )
    }

    pub(crate) fn fn_forecast_ets_stat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.forecast_ets_lifted(
            Function::ForecastEtsStat,
            [0, 1],
            args,
            cell,
            not_implemented,
        )
    }
}

/// The `values` and `timeline` of one FORECAST.ETS-family call, read once, and the season length
/// of each series shape a lifted call asks for, computed once.
struct EtsData {
    values: Result<Vec<Point>, Error>,
    timeline: Result<Vec<Point>, Error>,
    seasons: HashMap<SeriesOptions, Result<usize, Error>>,
}

impl EtsData {
    fn new(values: Result<Vec<Point>, Error>, timeline: Result<Vec<Point>, Error>) -> Self {
        EtsData {
            values,
            timeline,
            seasons: HashMap::new(),
        }
    }

    /// The detected season length of the series `options` builds (an error in `values` comes
    /// before one in `timeline`).
    fn season(&mut self, options: SeriesOptions) -> Result<usize, Error> {
        if let Some(season) = self.seasons.get(&options) {
            return season.clone();
        }
        let season = match (&self.values, &self.timeline) {
            (Err(error), _) | (_, Err(error)) => Err(error.clone()),
            (Ok(values), Ok(timeline)) => {
                build_series(values, timeline, options).and_then(|series| detect_season(&series.y))
            }
        };
        self.seasons.insert(options, season.clone());
        season
    }
}

/// The series settings at `completion` and `aggregation` (the defaults when not given): `#NUM!`
/// when either is out of range.
fn series_options(
    settings: &[Option<f64>],
    completion: usize,
    aggregation: usize,
) -> Result<SeriesOptions, Error> {
    let interpolate = match settings.get(completion).copied().flatten() {
        Some(value) => completion_setting(value).ok_or(Error::NUM)?,
        None => true,
    };
    let aggregate = match settings.get(aggregation).copied().flatten() {
        Some(value) => Aggregate::from_setting(value).ok_or(Error::NUM)?,
        None => Aggregate::Average,
    };
    Ok(SeriesOptions {
        interpolate,
        aggregate,
    })
}

/// `FORECAST.ETS.SEASONALITY(values, timeline, [data_completion], [aggregation])`: the detected
/// season length, 0 for none (unverified against Excel, decision 0221).
fn seasonality_element(data: &mut EtsData, settings: &[Option<f64>]) -> Result<f64, Error> {
    let options = series_options(settings, 2, 3)?;
    data.season(options).map(|season| season as f64)
}

/// The members not implemented yet (T-3600, T-3601).
fn not_implemented(_: &mut EtsData, _: &[Option<f64>]) -> Result<f64, Error> {
    Err(Error::NIMPL)
}
