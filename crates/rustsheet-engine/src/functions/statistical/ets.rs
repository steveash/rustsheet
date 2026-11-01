//! The FORECAST.ETS family's time series: pairing `values` with `timeline`, the timeline's step,
//! duplicate time stamps, missing points, and the season length FORECAST.ETS.SEASONALITY reports.
//!
//! **Not checked against Excel.** Excel for the web never computes the family (it is `#N/A` for
//! every valid call), and no Excel-desktop value exists yet, so what this module computes is the
//! textbook model as gridcraft implements it, adopted by owner ruling (decision 0221) until desktop
//! evidence replaces it (T-1553, T-2425). docs/excel-semantics/statistics.md "FORECAST.ETS family:
//! the model before Excel evidence" lists what is unverified.
//!
//! Adapted from gridcraft's `crates/functions/src/ets.rs` (<https://github.com/storytold/gridcraft> at
//! 67e39fae, a clean-room implementation of Hyndman, Koehler, Ord & Snyder, "Forecasting with
//! Exponential Smoothing: The State Space Approach", 2008): the series building, the calendar-month
//! timeline, the duplicate aggregation, the interpolation of missing points and the seasonality
//! detection. Changed here: the inputs are RustSheet's cell values, the least-squares line is
//! [`PairedMoments::line`], dates use the calendar Excel displays, and season detection has a work
//! cap ([`MAX_ACF_WORK`]). gridcraft is dual-licensed MIT OR Apache-2.0; RustSheet uses it under the
//! MIT License, whose notice is:
//!
//! > Copyright (c) 2026 ArtCraft Team and the GridCraft contributors
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy of this software
//! > and associated documentation files (the "Software"), to deal in the Software without
//! > restriction, including without limitation the rights to use, copy, modify, merge, publish,
//! > distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
//! > Software is furnished to do so, subject to the following conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all copies or
//! > substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
//! > BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//! > NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
//! > DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! > OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use super::correl::PairedMoments;
use crate::expressions::token::Error;
use crate::formatter::dates::excel_serial_to_ymd;
use crate::functions::date_and_time::displayed_month_length;

/// Most points `values` and `timeline` may each hold, and most points the completed series may
/// span (gridcraft's limit; Excel's is unknown).
pub(super) const MAX_POINTS: usize = 200_000;
/// Longest season the family accepts or detects: the hours of a year (Microsoft's documentation of
/// `seasonality`).
pub(super) const MAX_SEASON: usize = 8_760;
/// Share of the completed series' points that may be missing (Microsoft's documentation of
/// `data_completion`: "up to 30% missing data").
const MAX_MISSING: f64 = 0.3;
/// Most multiply-adds season detection may spend: the series' length times the lags it tries
/// (directive 5). Past it, detection is `#NUM!`: a series of 16,385 points or more (a
/// DEVIATION, statistics.md "FORECAST.ETS family: work caps").
pub(super) const MAX_ACF_WORK: usize = 1 << 27;
/// Most work one FORECAST.ETS-family call may spend over all its lifted elements: season
/// detections (length × lags) plus fits (length × [`MAX_FILTER_RUNS`]), each charged once per
/// distinct series and season. Past it the element is `#NUM!` (directive 5, a DEVIATION,
/// statistics.md "FORECAST.ETS family: work caps"): two fits of the longest series fit.
pub(super) const MAX_CALL_WORK: usize = 1 << 30;
/// A time stamp's step count may miss a whole number by this much and still count as on the grid.
const STEP_TOLERANCE: f64 = 1e-6;

/// One cell of `values` or `timeline`, read before pairing. An error cell is the caller's to
/// return; it never reaches here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Point {
    Number(f64),
    Blank,
    /// Text, a boolean or anything else that is not a number.
    Other,
}

/// How points that share a time stamp are combined (`aggregation`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Aggregate {
    Average,
    Count,
    CountA,
    Max,
    Median,
    Min,
    Sum,
}

impl Aggregate {
    /// `aggregation`, truncated: 0 or 1 AVERAGE (the default), 2 COUNT, 3 COUNTA, 4 MAX, 5 MEDIAN,
    /// 6 MIN, 7 SUM; anything else is `None` (`#NUM!`).
    pub(super) fn from_setting(value: f64) -> Option<Self> {
        let truncated = value.trunc();
        Some(if truncated == 0.0 || truncated == 1.0 {
            Aggregate::Average
        } else if truncated == 2.0 {
            Aggregate::Count
        } else if truncated == 3.0 {
            Aggregate::CountA
        } else if truncated == 4.0 {
            Aggregate::Max
        } else if truncated == 5.0 {
            Aggregate::Median
        } else if truncated == 6.0 {
            Aggregate::Min
        } else if truncated == 7.0 {
            Aggregate::Sum
        } else {
            return None;
        })
    }

    fn combine(self, group: &mut [f64]) -> Option<f64> {
        let count = group.len() as f64;
        match self {
            Aggregate::Count | Aggregate::CountA => Some(count),
            _ if group.is_empty() => None,
            Aggregate::Average => Some(group.iter().sum::<f64>() / count),
            Aggregate::Sum => Some(group.iter().sum()),
            Aggregate::Max => group.iter().copied().reduce(f64::max),
            Aggregate::Min => group.iter().copied().reduce(f64::min),
            Aggregate::Median => {
                group.sort_by(f64::total_cmp);
                let middle = group.len() / 2;
                let high = *group.get(middle)?;
                if group.len() % 2 == 1 {
                    Some(high)
                } else {
                    Some((group.get(middle.checked_sub(1)?)? + high) / 2.0)
                }
            }
        }
    }
}

/// `data_completion`, truncated: 1 (the default) interpolates missing points, 0 makes them zero;
/// anything else is `None` (`#NUM!`).
pub(super) fn completion_setting(value: f64) -> Option<bool> {
    let truncated = value.trunc();
    if truncated == 1.0 {
        Some(true)
    } else if truncated == 0.0 {
        Some(false)
    } else {
        None
    }
}

/// `seasonality`, truncated: 1 (the default) detects the season, 0 means none, 2 to 8,760 is the
/// season length; anything else is `None` (`#NUM!`).
pub(super) fn season_setting(value: f64) -> Option<Season> {
    let truncated = value.trunc();
    if truncated == 1.0 {
        Some(Season::Detect)
    } else if truncated == 0.0 {
        Some(Season::Fixed(0))
    } else if (2.0..=MAX_SEASON as f64).contains(&truncated) {
        Some(Season::Fixed(truncated as usize))
    } else {
        None
    }
}

/// What `seasonality` asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Season {
    /// Detect it ([`detect_season`]).
    Detect,
    /// This many steps; 0 means none.
    Fixed(usize),
}

/// The settings that shape the series.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct SeriesOptions {
    /// `true`: a missing point is the straight line between its known neighbours; `false`: zero.
    pub(super) interpolate: bool,
    pub(super) aggregate: Aggregate,
}

/// How a time stamp maps to its position in the series (0 = the first point).
#[derive(Clone, Copy, Debug)]
enum Axis {
    /// A constant numeric step.
    Linear { first: f64, step: f64 },
    /// Whole-day dates a constant number of calendar months apart, all on one day of the month
    /// (`day = Some(d)`) or all on the last day of their month (`day = None`).
    Months {
        first_month: i64,
        step: i64,
        day: Option<u32>,
    },
}

/// The month index (year × 12 + month - 1) and the displayed date of a whole-day serial.
fn month_of(serial: f64) -> Option<(i64, (i32, u32, u32))> {
    // `serial` is a whole day; one past `i64` saturates and `excel_serial_to_ymd` rejects it.
    let date = excel_serial_to_ymd(serial as i64)?;
    if date.2 == 0 {
        return None;
    }
    Some((i64::from(date.0) * 12 + i64::from(date.1) - 1, date))
}

impl Axis {
    fn position(&self, time: f64) -> Option<f64> {
        match *self {
            Axis::Linear { first, step } => Some((time - first) / step),
            Axis::Months {
                first_month,
                step,
                day,
            } => {
                let (month, (year, month_number, day_of_month)) = month_of(time.floor())?;
                let length = f64::from(displayed_month_length(year, month_number));
                let anchor = day.map_or(length, |anchor| f64::from(anchor).min(length));
                let offset = (f64::from(day_of_month) - anchor + time.fract()) / length;

                Some(((month - first_month) as f64 + offset) / step as f64)
            }
        }
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.abs()
}

/// The axis of sorted, distinct time stamps (at least two): a constant step, missing points
/// allowed, or, for whole-day dates, a constant number of calendar months.
fn detect_axis(times: &[f64]) -> Result<Axis, Error> {
    let (Some(&first), Some(_)) = (times.first(), times.get(1)) else {
        return Err(Error::NUM);
    };
    let step = times
        .windows(2)
        .filter_map(|pair| Some(pair.get(1)? - pair.first()?))
        .fold(f64::INFINITY, f64::min);
    if step.is_finite() && step > 0.0 {
        let on_grid = times
            .windows(2)
            .all(|pair| match (pair.first(), pair.get(1)) {
                (Some(a), Some(b)) => {
                    let steps = (b - a) / step;
                    (steps - steps.round()).abs() <= STEP_TOLERANCE
                }
                _ => true,
            });
        if on_grid {
            return Ok(Axis::Linear { first, step });
        }
    }
    let mut months = Vec::with_capacity(times.len());
    let mut same_day: Option<u32> = None;
    let mut all_same_day = true;
    let mut all_month_ends = true;
    for &time in times {
        if time.fract() != 0.0 {
            return Err(Error::NUM);
        }
        let (month, (year, month_number, day)) = month_of(time).ok_or(Error::NUM)?;
        months.push(month);
        match same_day {
            None => same_day = Some(day),
            Some(seen) if seen != day => all_same_day = false,
            Some(_) => {}
        }
        if day != displayed_month_length(year, month_number) {
            all_month_ends = false;
        }
    }
    let day = if all_same_day {
        same_day
    } else if all_month_ends {
        None
    } else {
        return Err(Error::NUM);
    };
    let mut step = 0i64;
    for pair in months.windows(2) {
        if let (Some(a), Some(b)) = (pair.first(), pair.get(1)) {
            let difference = b - a;
            if difference <= 0 {
                return Err(Error::NUM);
            }
            step = gcd(step, difference);
        }
    }
    let first_month = months.first().copied().ok_or(Error::NUM)?;
    if step <= 0 {
        return Err(Error::NUM);
    }
    Ok(Axis::Months {
        first_month,
        step,
        day,
    })
}

/// An evenly spaced series built from a `values`/`timeline` pair.
#[derive(Clone, Debug)]
pub(super) struct Series {
    /// The completed values, one per step from the first time stamp to the last.
    pub(super) y: Vec<f64>,
    axis: Axis,
}

impl Series {
    /// How many steps `target` lies after the last point (fractional between steps). A target
    /// before the last time stamp is `#NUM!` (Microsoft's documentation of `target_date`).
    pub(super) fn horizon(&self, target: f64) -> Result<f64, Error> {
        let position = self.axis.position(target).ok_or(Error::NUM)?;
        let horizon = position - (self.y.len() as f64 - 1.0);
        if !horizon.is_finite() || horizon < -STEP_TOLERANCE {
            return Err(Error::NUM);
        }
        Ok(horizon.max(0.0))
    }
}

/// Builds the series: pairs `values` with `timeline` in reading order, sorts by time stamp,
/// combines duplicates, checks that the time stamps lie on a constant step and completes the
/// missing points. Different counts are `#N/A`; a value or time stamp that is not a number is
/// `#VALUE!` (a row with neither is skipped); fewer than two time stamps, no constant step, more
/// than 30% missing or too long a series is `#NUM!`.
pub(super) fn build_series(
    values: &[Point],
    timeline: &[Point],
    options: SeriesOptions,
) -> Result<Series, Error> {
    if values.len() != timeline.len() {
        return Err(Error::NA);
    }
    if values.len() > MAX_POINTS {
        return Err(Error::NUM);
    }
    let mut points: Vec<(f64, Option<f64>)> = Vec::with_capacity(values.len());
    for (time, value) in timeline.iter().zip(values) {
        let value = match value {
            Point::Number(value) => Some(*value),
            Point::Blank => None,
            Point::Other => return Err(Error::VALUE),
        };
        let time = match time {
            Point::Number(time) if time.is_finite() => *time,
            Point::Blank if value.is_none() => continue,
            _ => return Err(Error::VALUE),
        };
        points.push((time, value));
    }
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut times: Vec<f64> = points.iter().map(|point| point.0).collect();
    times.dedup();
    if times.len() < 2 {
        return Err(Error::NUM);
    }
    let axis = detect_axis(&times)?;
    let last = times.last().copied().ok_or(Error::NUM)?;
    let span = axis.position(last).ok_or(Error::NUM)?.round();

    if !(span >= 1.0 && span < MAX_POINTS as f64) {
        return Err(Error::NUM);
    }
    // `span` is a whole number in [1, MAX_POINTS) here.
    let length = span as usize + 1;
    let mut slots: Vec<Option<f64>> = vec![None; length];
    let mut group: Vec<f64> = Vec::new();
    let mut group_at: Option<usize> = None;
    let mut flush = |at: Option<usize>, group: &mut Vec<f64>| {
        if let Some(slot) = at.and_then(|at| slots.get_mut(at)) {
            *slot = options.aggregate.combine(group);
        }
        group.clear();
    };
    for &(time, value) in &points {
        let position = axis.position(time).ok_or(Error::NUM)?.round();

        if !(position >= 0.0 && position < length as f64) {
            return Err(Error::NUM);
        }

        let position = position as usize;
        if group_at != Some(position) {
            flush(group_at, &mut group);
            group_at = Some(position);
        }
        if let Some(value) = value {
            group.push(value);
        }
    }
    flush(group_at, &mut group);
    let known = slots.iter().filter(|slot| slot.is_some()).count();

    if known == 0 || (length - known) as f64 > MAX_MISSING * length as f64 {
        return Err(Error::NUM);
    }
    let y = if options.interpolate {
        interpolate(&slots)
    } else {
        slots.iter().map(|slot| slot.unwrap_or(0.0)).collect()
    };
    Ok(Series { y, axis })
}

/// Missing points become the straight line between their known neighbours, or the nearest known
/// value at either end.
fn interpolate(slots: &[Option<f64>]) -> Vec<f64> {
    let mut out = Vec::with_capacity(slots.len());
    let mut known = slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.map(|value| (index, value)));
    let mut next = known.next();
    let mut previous: Option<(usize, f64)> = None;
    for (index, slot) in slots.iter().enumerate() {
        if let Some(value) = slot {
            out.push(*value);
            previous = Some((index, *value));
            next = known.next();
            continue;
        }

        out.push(match (previous, next) {
            (Some((a, value_a)), Some((b, value_b))) => {
                value_a + (value_b - value_a) * (index - a) as f64 / (b - a) as f64
            }
            (Some((_, value)), None) | (None, Some((_, value))) => value,
            (None, None) => 0.0,
        });
    }
    out
}

/// The intercept and slope of the least-squares line through the points (`xs[i]`, `ys[i]`): the
/// mean and slope 0 when the x values do not vary.
fn least_squares(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let xs: Vec<Option<f64>> = xs.iter().copied().map(Some).collect();
    let ys: Vec<Option<f64>> = ys.iter().copied().map(Some).collect();
    let moments = PairedMoments::new(&xs, &ys);
    moments.line().unwrap_or((moments.mean_y, 0.0))
}

/// The residuals of the least-squares line through `y` at x = 0, 1, 2, …
fn detrend(y: &[f64]) -> Vec<f64> {
    let xs: Vec<f64> = (0..y.len()).map(|index| index as f64).collect();
    let (intercept, slope) = least_squares(&xs, y);
    y.iter()
        .zip(&xs)
        .map(|(value, x)| value - (intercept + slope * x))
        .collect()
}

/// The multiply-adds [`detect_season`] spends on a series of `n` points.
pub(super) fn season_work(n: usize) -> usize {
    n.saturating_mul((n / 2).min(MAX_SEASON))
}

/// The season length of `y`, 0 when it shows none: the lag of the highest local peak of the
/// autocorrelation of the linearly detrended series that rises above the 95% white-noise bound
/// (2/√n), at a lag from 2 to n/2 (at most [`MAX_SEASON`]). `#NUM!` past [`MAX_ACF_WORK`].
pub(super) fn detect_season(y: &[f64]) -> Result<usize, Error> {
    let n = y.len();
    if n < 4 {
        return Ok(0);
    }
    if season_work(n) > MAX_ACF_WORK {
        return Err(Error::NUM);
    }
    let max_lag = (n / 2).min(MAX_SEASON);
    let residuals = detrend(y);
    let variance: f64 = residuals.iter().map(|r| r * r).sum();
    let scale: f64 = y.iter().map(|v| v * v).sum();
    if !(variance.is_finite() && variance > 1e-20 * scale.max(f64::MIN_POSITIVE)) {
        return Ok(0);
    }
    let last_lag = (max_lag + 1).min(n - 1);
    let autocorrelation: Vec<f64> = (0..=last_lag)
        .map(|lag| {
            residuals
                .iter()
                .skip(lag)
                .zip(&residuals)
                .map(|(a, b)| a * b)
                .sum::<f64>()
                / variance
        })
        .collect();

    let bound = 2.0 / (n as f64).sqrt();
    let mut best = 0usize;
    let mut best_value = bound;
    for lag in 2..=max_lag {
        let (Some(&before), Some(&here)) = (autocorrelation.get(lag - 1), autocorrelation.get(lag))
        else {
            break;
        };
        let after = autocorrelation
            .get(lag + 1)
            .copied()
            .unwrap_or(f64::NEG_INFINITY);
        if here > before && here >= after && here > best_value {
            best = lag;
            best_value = here;
        }
    }
    Ok(best)
}

mod model;
pub(super) use model::{fit, Fit, MAX_FILTER_RUNS};

#[cfg(test)]
mod tests;
