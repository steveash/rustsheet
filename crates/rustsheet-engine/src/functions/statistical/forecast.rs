use std::collections::HashMap;
use std::rc::Rc;

use super::ets::{
    self, build_series, completion_setting, detect_season, fit, fit_work, season_setting,
    season_work, DuplicateAggregate, Fit, Point, Season, Series, SeriesOptions,
};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::date_and_time::error_kind;
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
    /// for more than [`ets::MAX_POINTS`] cells, checked before any is read. A range is read by
    /// [`Model::evaluate_range`], which charges its cells to the formula's budget.
    fn ets_points(
        &mut self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Point>, CalcResult> {
        let too_many = || CalcResult::new_error(Error::NUM, cell, String::new());
        match value {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(Error::VALUE, cell, String::new()));
                }
                let (rows, columns) = value.shape();
                if rows.saturating_mul(columns) > ets::MAX_POINTS {
                    return Err(too_many());
                }
                let nodes = self.evaluate_range(*left, *right, cell)?;
                nodes
                    .iter()
                    .flatten()
                    .map(|node| array_point(node, cell))
                    .collect()
            }
            CalcResult::Array(rows) => {
                if rows.iter().map(Vec::len).sum::<usize>() > ets::MAX_POINTS {
                    return Err(too_many());
                }
                rows.iter()
                    .flatten()
                    .map(|node| array_point(node, cell))
                    .collect()
            }
            CalcResult::Number(number) => Ok(vec![Point::Number(*number)]),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![Point::Blank]),
            CalcResult::Error { .. } => Err(value.clone()),
            _ => Ok(vec![Point::Other]),
        }
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
        let mut points: [Result<Vec<Point>, CalcResult>; 2] = [Ok(Vec::new()), Ok(Vec::new())];
        for ((slot, read), &index) in direct_errors.iter_mut().zip(&mut points).zip(&whole) {
            if let Some(node) = args.get(index) {
                let value = self.evaluate_node_with_reference(node, cell);
                if value.is_error() {
                    *slot = Some(value);
                } else {
                    *read = self.ets_points(&value, cell);
                }
            }
        }
        let [values, timeline] = points.map(|read| read.map_err(|error| error_kind(&error)));
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
        self.forecast_ets_lifted(Function::ForecastEts, [1, 2], args, cell, forecast_element)
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

/// The `values` and `timeline` of one FORECAST.ETS-family call, read once, and what a lifted
/// call asks of them, computed once per series shape and season: the series, its detected
/// season and the fitted model.
struct EtsData {
    values: Result<Vec<Point>, Error>,
    timeline: Result<Vec<Point>, Error>,
    series: HashMap<SeriesOptions, Result<Rc<Series>, Error>>,
    seasons: HashMap<SeriesOptions, Result<usize, Error>>,
    fits: HashMap<(SeriesOptions, usize), Rc<Fit>>,
    /// Work spent so far, against [`ets::MAX_CALL_WORK`].
    work: usize,
}

impl EtsData {
    fn new(values: Result<Vec<Point>, Error>, timeline: Result<Vec<Point>, Error>) -> Self {
        EtsData {
            values,
            timeline,
            series: HashMap::new(),
            seasons: HashMap::new(),
            fits: HashMap::new(),
            work: 0,
        }
    }

    /// Charges `cost` to the call: `#NUM!` once the call's work would pass
    /// [`ets::MAX_CALL_WORK`].
    fn spend(&mut self, cost: usize) -> Result<(), Error> {
        let work = self.work.saturating_add(cost);
        if work > ets::MAX_CALL_WORK {
            return Err(Error::NUM);
        }
        self.work = work;
        Ok(())
    }

    /// The series `options` builds (an error in `values` comes before one in `timeline`).
    fn series(&mut self, options: SeriesOptions) -> Result<Rc<Series>, Error> {
        if let Some(series) = self.series.get(&options) {
            return series.clone();
        }
        let series = match (&self.values, &self.timeline) {
            (Err(error), _) | (_, Err(error)) => Err(error.clone()),
            (Ok(values), Ok(timeline)) => build_series(values, timeline, options).map(Rc::new),
        };
        self.series.insert(options, series.clone());
        series
    }

    /// The season length `season` asks for on the series `options` builds.
    fn season(&mut self, options: SeriesOptions, season: Season) -> Result<usize, Error> {
        let series = self.series(options)?;
        match season {
            Season::Fixed(length) => Ok(length),
            Season::Detect => {
                if let Some(detected) = self.seasons.get(&options) {
                    return detected.clone();
                }
                self.spend(season_work(series.y.len()))?;
                let detected = detect_season(&series.y);
                self.seasons.insert(options, detected.clone());
                detected
            }
        }
    }

    /// The model fitted to the series `options` builds, with season length `m`.
    fn fit(&mut self, options: SeriesOptions, m: usize) -> Result<Rc<Fit>, Error> {
        if let Some(fitted) = self.fits.get(&(options, m)) {
            return Ok(Rc::clone(fitted));
        }
        let series = self.series(options)?;
        self.spend(fit_work(series.y.len(), m))?;
        let fitted = Rc::new(fit(&series.y, m));
        self.fits.insert((options, m), Rc::clone(&fitted));
        Ok(fitted)
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
        Some(value) => DuplicateAggregate::from_setting(value).ok_or(Error::NUM)?,
        None => DuplicateAggregate::Average,
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
    data.season(options, Season::Detect)
        .map(|season| season as f64)
}

/// `seasonality` at `index` (detect when not given): `#NUM!` when out of range.
fn season_option(settings: &[Option<f64>], index: usize) -> Result<Season, Error> {
    match settings.get(index).copied().flatten() {
        Some(value) => season_setting(value).ok_or(Error::NUM),
        None => Ok(Season::Detect),
    }
}

/// `FORECAST.ETS(target_date, values, timeline, [seasonality], [data_completion],
/// [aggregation])`: the model's forecast at `target_date` (unverified against Excel, decision
/// 0221). The settings are checked in order, then the series is built, then `target_date` must
/// not be before its last time stamp.
fn forecast_element(data: &mut EtsData, settings: &[Option<f64>]) -> Result<f64, Error> {
    let target = settings.first().copied().flatten().ok_or(Error::VALUE)?;
    let season = season_option(settings, 3)?;
    let options = series_options(settings, 4, 5)?;
    let horizon = data.series(options)?.horizon(target)?;
    let m = data.season(options, season)?;
    Ok(data.fit(options, m)?.forecast(horizon))
}

/// The members not implemented yet (T-3601).
fn not_implemented(_: &mut EtsData, _: &[Option<f64>]) -> Result<f64, Error> {
    Err(Error::NIMPL)
}

/// One element of a `values` or `timeline` array or range as the series reads it; an error element
/// is that error.
fn array_point(node: &ArrayNode, cell: CellReferenceIndex) -> Result<Point, CalcResult> {
    Ok(match node {
        ArrayNode::Number(number) => Point::Number(*number),
        ArrayNode::Empty => Point::Blank,
        ArrayNode::Error(error) => {
            return Err(CalcResult::new_error(error.clone(), cell, String::new()))
        }
        _ => Point::Other,
    })
}

#[cfg(test)]
mod tests {
    use super::{ets, fit_work, EtsData};
    use crate::expressions::token::Error;

    #[test]
    fn a_call_spends_at_most_its_work_budget() {
        let mut data = EtsData::new(Ok(Vec::new()), Ok(Vec::new()));
        assert_eq!(data.spend(ets::MAX_CALL_WORK), Ok(()));
        assert_eq!(data.spend(1), Err(Error::NUM));
        // A fit of the longest series costs less than half the budget: two fit, a third does not.
        assert!(2 * fit_work(ets::MAX_POINTS, 0) <= ets::MAX_CALL_WORK);
        assert!(3 * fit_work(ets::MAX_POINTS, 0) > ets::MAX_CALL_WORK);
        // A short series charges its season length too: a lifted `seasonality` array of long
        // seasons over three points is cut off after about 57 fits, not run for thousands.
        let fits = ets::MAX_CALL_WORK / fit_work(3, ets::MAX_SEASON);
        assert!((50..64).contains(&fits), "{fits}");
    }
}
