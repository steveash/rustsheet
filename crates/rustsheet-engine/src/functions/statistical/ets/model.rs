//! The AAA exponential-smoothing model FORECAST.ETS fits: additive error, trend and seasonality
//! (Hyndman, Koehler, Ord & Snyder 2008, ETS(A,A,A), component form):
//!
//! ```text
//! ŷ_t = l_{t-1} + b_{t-1} + s_{t-m}        e_t = y_t − ŷ_t
//! l_t = l_{t-1} + b_{t-1} + α·e_t
//! b_t = b_{t-1} + α·β·e_t
//! s_t = s_{t-m} + γ·e_t
//! ```
//!
//! **Not checked against Excel** (decision 0221). Adapted from gridcraft's `ets.rs`
//! (`initial_states`, `filter`, `optimise`, `fit`, `Fit::forecast`, `Fit::forecast_sd` and
//! FORECAST.ETS.STAT's statistics); the MIT notice is in the parent module. Excel does not
//! document its optimiser, so α, β and γ, and the last digits of a forecast on noisy data, may
//! differ from Excel's; exact data (a straight line, an exactly repeating season) forecast exactly.

use super::{detrend, least_squares};

/// Seasonally adjusted values the initial level and trend are fitted to (Hyndman et al. §2.6.1).
const INIT_POINTS: usize = 10;
/// Most Nelder–Mead iterations after the grid.
const NELDER_MEAD_ITERATIONS: usize = 400;
/// Most filter runs one fit makes: the 125 grid points, the 3 other starting vertices, at most 5
/// per Nelder–Mead iteration (reflect, contract, then shrink 3 vertices), and the final run.
pub(in crate::functions::statistical) const MAX_FILTER_RUNS: usize =
    125 + 3 + 5 * NELDER_MEAD_ITERATIONS + 1;
/// The grid each smoothing parameter starts from.
const GRID: [f64; 5] = [0.05, 0.25, 0.5, 0.75, 0.95];

/// The states the filter starts from.
#[derive(Clone, Debug)]
struct Initial {
    level: f64,
    trend: f64,
    /// One seasonal state per position in the season (empty without seasonality).
    season: Vec<f64>,
}

/// The seasonal states of a classical decomposition of the first seasons (a centred moving
/// average as the trend), or the deviations from a straight line under two seasons; centred on 0.
fn initial_season(y: &[f64], m: usize) -> Vec<f64> {
    let mut season = vec![0.0; m];
    let mut counts = vec![0usize; m];
    if y.len() >= 2 * m {
        let window = y.get(..y.len().min(4 * m)).unwrap_or(y);
        let mut prefix = Vec::with_capacity(window.len() + 1);
        prefix.push(0.0);
        let mut total = 0.0;
        for value in window {
            total += value;
            prefix.push(total);
        }
        let sum = |from: usize, to: usize| {
            prefix.get(to).copied().unwrap_or(0.0) - prefix.get(from).copied().unwrap_or(0.0)
        };
        let half = m / 2;
        for t in half..window.len().saturating_sub(half) {
            let average = if m % 2 == 1 {
                sum(t - half, t + half + 1) / m as f64
            } else {
                let ends = window.get(t - half).copied().unwrap_or(0.0)
                    + window.get(t + half).copied().unwrap_or(0.0);
                (sum(t - half + 1, t + half) + ends / 2.0) / m as f64
            };
            if let (Some(state), Some(count), Some(value)) =
                (season.get_mut(t % m), counts.get_mut(t % m), window.get(t))
            {
                *state += value - average;
                *count += 1;
            }
        }
    } else {
        for (t, residual) in detrend(y).iter().enumerate() {
            if let (Some(state), Some(count)) = (season.get_mut(t % m), counts.get_mut(t % m)) {
                *state += residual;
                *count += 1;
            }
        }
    }
    for (state, count) in season.iter_mut().zip(&counts) {
        if *count > 0 {
            *state /= *count as f64;
        }
    }
    let mean = season.iter().sum::<f64>() / m as f64;
    for state in &mut season {
        *state -= mean;
    }
    season
}

/// The initial states (Hyndman et al. 2008, §2.6.1): the seasonal states, then the level and
/// trend of the straight line through the first [`INIT_POINTS`] seasonally adjusted values.
fn initial_states(y: &[f64], m: usize) -> Initial {
    let season = if m >= 2 {
        initial_season(y, m)
    } else {
        Vec::new()
    };
    let count = y.len().min(INIT_POINTS);
    let xs: Vec<f64> = (1..=count).map(|t| t as f64).collect();
    let adjusted: Vec<f64> = y
        .iter()
        .take(count)
        .enumerate()
        .map(|(t, value)| {
            let state = season.get(t % m.max(1)).copied().unwrap_or(0.0);
            value - state
        })
        .collect();
    let (level, trend) = least_squares(&xs, &adjusted);
    Initial {
        level,
        trend,
        season,
    }
}

/// The smoothing parameters α, β and γ.
type Parameters = [f64; 3];

/// The states after the filter has run over the data, and its sum of squared one-step errors.
#[derive(Clone, Debug)]
struct State {
    level: f64,
    trend: f64,
    season: Vec<f64>,
    sse: f64,
}

/// Runs the filter over `y`; `errors` collects the one-step errors when given.
fn filter(
    y: &[f64],
    initial: &Initial,
    parameters: Parameters,
    mut errors: Option<&mut Vec<f64>>,
) -> State {
    let [alpha, beta, gamma] = parameters;
    let m = initial.season.len();
    let (mut level, mut trend) = (initial.level, initial.trend);
    let mut season = initial.season.clone();
    let mut sse = 0.0;
    for (t, value) in y.iter().enumerate() {
        let slot = if m > 0 { season.get_mut(t % m) } else { None };
        let seasonal = slot.as_ref().map_or(0.0, |state| **state);
        let error = value - (level + trend + seasonal);
        if let Some(slot) = slot {
            *slot = seasonal + gamma * error;
        }
        level += trend + alpha * error;
        trend += alpha * beta * error;
        sse += error * error;
        if let Some(errors) = errors.as_deref_mut() {
            errors.push(error);
        }
    }
    State {
        level,
        trend,
        season,
        sse,
    }
}

/// `parameters` held to the unit box, with the inactive ones (γ without seasonality) at 0.
fn clamp(parameters: Parameters, active: usize) -> Parameters {
    let mut clamped = [0.0; 3];
    for (index, (out, value)) in clamped.iter_mut().zip(parameters).enumerate() {
        *out = if index < active {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    clamped
}

/// The α, β and (with seasonality) γ in [0, 1] that minimise the sum of squared one-step errors:
/// the best point of a coarse grid, then a Nelder–Mead simplex held to the box. It runs the
/// filter at most [`MAX_FILTER_RUNS`] - 1 times.
fn optimise(y: &[f64], initial: &Initial) -> Parameters {
    let active = if initial.season.is_empty() { 2 } else { 3 };
    let objective = |parameters: Parameters| {
        let sse = filter(y, initial, clamp(parameters, active), None).sse;
        if sse.is_finite() {
            sse
        } else {
            f64::INFINITY
        }
    };
    let best = grid_start(&objective, active);
    let mut simplex = initial_simplex(best, &objective, active);
    for _ in 0..NELDER_MEAD_ITERATIONS {
        if !nelder_mead_step(&mut simplex, &objective, active) {
            break;
        }
    }
    simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
    clamp(simplex.first().map_or(best.0, |vertex| vertex.0), active)
}

/// A point of the simplex and its objective value.
type Vertex = (Parameters, f64);

/// The best point of the [`GRID`] (γ only with seasonality): 125 filter runs at most.
fn grid_start(objective: &impl Fn(Parameters) -> f64, active: usize) -> Vertex {
    let gammas: &[f64] = if active == 3 { &GRID } else { &GRID[..1] };
    let mut best: Vertex = ([0.5, 0.1, 0.1], f64::INFINITY);
    for alpha in GRID {
        for beta in GRID {
            for &gamma in gammas {
                let parameters = clamp([alpha, beta, gamma], active);
                let value = objective(parameters);
                if value < best.1 {
                    best = (parameters, value);
                }
            }
        }
    }
    best
}

/// The starting simplex: `best` and, for each active parameter, `best` with that one moved 0.1
/// towards the middle of the box.
fn initial_simplex(
    best: Vertex,
    objective: &impl Fn(Parameters) -> f64,
    active: usize,
) -> Vec<Vertex> {
    let mut simplex = vec![best];
    for index in 0..active {
        let mut parameters = best.0;
        if let Some(value) = parameters.get_mut(index) {
            *value = if *value > 0.5 {
                *value - 0.1
            } else {
                *value + 0.1
            };
        }
        let parameters = clamp(parameters, active);
        simplex.push((parameters, objective(parameters)));
    }
    simplex
}

/// The point `t` of the way from `from` to `to` (negative: beyond `from`), held to the box.
fn towards(from: Parameters, to: Parameters, t: f64, active: usize) -> Parameters {
    let mut out = [0.0; 3];
    for ((out, a), b) in out.iter_mut().zip(from).zip(to) {
        *out = a + t * (b - a);
    }
    clamp(out, active)
}

/// One Nelder–Mead step: reflect the worst vertex through the others' centroid, expand,
/// contract or shrink (5 filter runs at most). `false` once the simplex has converged.
fn nelder_mead_step(
    simplex: &mut [Vertex],
    objective: &impl Fn(Parameters) -> f64,
    active: usize,
) -> bool {
    simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
    let (Some(&lowest), Some(&highest)) = (simplex.first(), simplex.last()) else {
        return false;
    };
    let second_highest = simplex
        .len()
        .checked_sub(2)
        .and_then(|index| simplex.get(index))
        .map_or(highest.1, |vertex| vertex.1);
    let size = simplex
        .iter()
        .map(|vertex| {
            vertex
                .0
                .iter()
                .zip(lowest.0)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max)
        })
        .fold(0.0, f64::max);
    if (highest.1 - lowest.1).abs() <= 1e-12 * lowest.1.abs() + 1e-300 || size < 1e-7 {
        return false;
    }
    let others = simplex.len().saturating_sub(1);
    let mut centroid = [0.0; 3];
    for vertex in simplex.iter().take(others) {
        for (c, value) in centroid.iter_mut().zip(vertex.0) {
            *c += value / others.max(1) as f64;
        }
    }
    let reflected = towards(centroid, highest.0, -1.0, active);
    let reflected_value = objective(reflected);
    let replacement = if reflected_value < lowest.1 {
        let expanded = towards(centroid, highest.0, -2.0, active);
        let expanded_value = objective(expanded);
        Some(if expanded_value < reflected_value {
            (expanded, expanded_value)
        } else {
            (reflected, reflected_value)
        })
    } else if reflected_value < second_highest {
        Some((reflected, reflected_value))
    } else {
        let contracted = towards(centroid, highest.0, 0.5, active);
        let contracted_value = objective(contracted);
        (contracted_value < highest.1).then_some((contracted, contracted_value))
    };
    match replacement {
        Some(vertex) => {
            if let Some(last) = simplex.last_mut() {
                *last = vertex;
            }
        }
        None => {
            for vertex in simplex.iter_mut().skip(1) {
                let shrunk = towards(lowest.0, vertex.0, 0.5, active);
                *vertex = (shrunk, objective(shrunk));
            }
        }
    }
    true
}

/// The work a fit of `n` points with season length `m` is charged: every filter run walks the
/// points and copies the `m` seasonal states, and the initial states are O(n + m) more.
pub(in crate::functions::statistical) fn fit_work(n: usize, m: usize) -> usize {
    MAX_FILTER_RUNS.saturating_mul(n.saturating_add(m))
}

/// A fitted model.
#[derive(Clone, Debug)]
pub(in crate::functions::statistical) struct Fit {
    /// The season length, 0 without seasonality.
    m: usize,
    /// How many points it was fitted to.
    n: usize,
    /// The fitted α, β and γ (γ is 0 without seasonality).
    parameters: Parameters,
    state: State,
    /// The one-step errors of the fitted model, one per point.
    errors: Vec<f64>,
}

/// Fits the model to `y` with season length `m` (below 2: no seasonality).
pub(in crate::functions::statistical) fn fit(y: &[f64], m: usize) -> Fit {
    let m = if m >= 2 { m } else { 0 };
    let initial = initial_states(y, m);
    let parameters = optimise(y, &initial);
    let mut errors = Vec::with_capacity(y.len());
    let state = filter(y, &initial, parameters, Some(&mut errors));
    Fit {
        m,
        n: y.len(),
        parameters,
        state,
        errors,
    }
}

impl Fit {
    /// The model's value `steps` whole steps after the last point.
    fn at(&self, steps: f64) -> f64 {
        let seasonal = if self.m > 0 {
            let index = ((self.n as f64 - 1.0) + steps).rem_euclid(self.m as f64);
            self.state
                .season
                .get(index as usize)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };
        self.state.level + steps * self.state.trend + seasonal
    }

    /// The point forecast `horizon` steps after the last point; between whole steps it is the
    /// straight line between the forecasts at the steps either side.
    pub(in crate::functions::statistical) fn forecast(&self, horizon: f64) -> f64 {
        let below = horizon.floor();
        let at_below = self.at(below);
        if horizon == below {
            at_below
        } else {
            at_below + (horizon - below) * (self.at(below + 1.0) - at_below)
        }
    }

    /// The fitted α, β and γ.
    pub(in crate::functions::statistical) fn parameters(&self) -> [f64; 3] {
        self.parameters
    }

    /// The fitted model's one-step errors, one per point.
    pub(in crate::functions::statistical) fn errors(&self) -> &[f64] {
        &self.errors
    }

    /// The standard deviation of the forecast error `horizon` steps after the last point
    /// (Hyndman et al. 2008, Table 6.1, class 1): σ²·(1 + Σ_{j=1}^{k} (α + αβ·j + γ·[j mod m =
    /// 0])²) with k = ⌈h⌉ - 1 and σ² = SSE / n, in closed form.
    pub(in crate::functions::statistical) fn forecast_sd(&self, horizon: f64) -> f64 {
        let sigma2 = self.state.sse / self.n.max(1) as f64;
        let k = horizon.ceil().max(1.0) - 1.0;
        let [alpha, beta, gamma] = self.parameters;
        let trend_gain = alpha * beta;
        let mut sum = k * alpha * alpha
            + alpha * trend_gain * k * (k + 1.0)
            + trend_gain * trend_gain * k * (k + 1.0) * (2.0 * k + 1.0) / 6.0;
        if self.m > 0 {
            let m = self.m as f64;
            let seasons = (k / m).floor();
            sum +=
                2.0 * gamma * (seasons * alpha + trend_gain * m * seasons * (seasons + 1.0) / 2.0)
                    + gamma * gamma * seasons;
        }
        (sigma2 * (1.0 + sum)).max(0.0).sqrt()
    }
}

/// The mean absolute one-step error (FORECAST.ETS.STAT's statistic 6).
pub(in crate::functions::statistical) fn mean_absolute_error(errors: &[f64]) -> f64 {
    errors.iter().map(|e| e.abs()).sum::<f64>() / errors.len().max(1) as f64
}

/// The root mean squared one-step error (statistic 7).
pub(in crate::functions::statistical) fn root_mean_squared_error(errors: &[f64]) -> f64 {
    (errors.iter().map(|e| e * e).sum::<f64>() / errors.len().max(1) as f64).sqrt()
}

/// The mean absolute scaled error (statistic 4): the MAE over that of the naive forecast, the
/// mean absolute change between neighbouring points. `None` when that is 0 (`#DIV/0!`).
pub(in crate::functions::statistical) fn mean_absolute_scaled_error(
    y: &[f64],
    errors: &[f64],
) -> Option<f64> {
    let changes: Vec<f64> = y
        .windows(2)
        .filter_map(|pair| Some((pair.get(1)? - pair.first()?).abs()))
        .collect();
    let naive = changes.iter().sum::<f64>() / changes.len().max(1) as f64;
    (naive != 0.0).then(|| mean_absolute_error(errors) / naive)
}

/// The symmetric mean absolute percentage error (statistic 5): the mean of |e| / ((|y| + |ŷ|)
/// / 2), with ŷ = y - e, over the points where the denominator is positive.
pub(in crate::functions::statistical) fn symmetric_mean_absolute_percentage_error(
    y: &[f64],
    errors: &[f64],
) -> f64 {
    let terms: Vec<f64> = y
        .iter()
        .zip(errors)
        .filter_map(|(value, error)| {
            let denominator = (value.abs() + (value - error).abs()) / 2.0;
            (denominator > 0.0).then(|| error.abs() / denominator)
        })
        .collect();
    terms.iter().sum::<f64>() / terms.len().max(1) as f64
}

#[cfg(test)]
mod tests {
    use super::{
        filter, fit, initial_states, mean_absolute_error, mean_absolute_scaled_error,
        root_mean_squared_error, symmetric_mean_absolute_percentage_error, Fit, State,
    };

    fn line(n: usize) -> Vec<f64> {
        (1..=n).map(|t| 3.0 + 2.0 * t as f64).collect()
    }

    #[test]
    fn an_exact_line_starts_on_the_line_and_has_no_error() {
        let y = line(12);
        let initial = initial_states(&y, 0);
        assert!((initial.level - 3.0).abs() < 1e-12 && (initial.trend - 2.0).abs() < 1e-12);
        let state = filter(&y, &initial, [0.5, 0.1, 0.0], None);
        assert!(state.sse < 1e-20);
    }

    #[test]
    fn a_season_below_two_is_none() {
        let fitted = fit(&line(12), 1);
        assert_eq!(fitted.m, 0);
        assert!(fitted.state.season.is_empty());
        assert_eq!(fitted.parameters()[2], 0.0);
        assert_eq!(fitted.errors().len(), 12);
        assert!(fitted.forecast_sd(5.0) < 1e-9);
    }

    #[test]
    fn initial_seasonal_states_are_centred() {
        let season = [3.0, -1.0, -4.0, 2.0];
        let y: Vec<f64> = (0..16).map(|t| 10.0 + t as f64 + season[t % 4]).collect();
        let initial = initial_states(&y, 4);
        assert!(initial.season.iter().sum::<f64>().abs() < 1e-12);
        for (state, expected) in initial.season.iter().zip(season) {
            assert!((state - expected).abs() < 1e-12, "{state} vs {expected}");
        }
    }

    #[test]
    fn the_forecast_variance_grows_with_the_horizon() {
        let y: Vec<f64> = line(20)
            .iter()
            .enumerate()
            .map(|(t, v)| v + if t % 3 == 0 { 0.5 } else { -0.25 })
            .collect();
        let fitted = fit(&y, 0);
        let near = fitted.forecast_sd(1.0);
        assert!(near > 0.0 && fitted.forecast_sd(6.0) >= near);
        // A fractional horizon has the variance of the whole step at or past it.
        assert_eq!(fitted.forecast_sd(1.5), fitted.forecast_sd(2.0));
    }

    #[test]
    fn error_statistics_on_a_known_error_vector() {
        let y = [10.0, 12.0, 11.0, 15.0];
        let errors = [1.0, -2.0, 0.5, 3.0];
        assert_eq!(mean_absolute_error(&errors), 1.625);
        assert_eq!(root_mean_squared_error(&errors), (14.25_f64 / 4.0).sqrt());
        // The naive changes are 2, 1 and 4: mean 7/3.
        let mase = mean_absolute_scaled_error(&y, &errors).unwrap_or(f64::NAN);
        assert!((mase - 1.625 / (7.0 / 3.0)).abs() < 1e-15);
        assert_eq!(
            mean_absolute_scaled_error(&[5.0, 5.0, 5.0], &errors[..3]),
            None
        );
        // ŷ = y - e: 9, 14, 10.5, 12; terms |e| / ((|y| + |ŷ|) / 2).
        let expected = (1.0 / 9.5 + 2.0 / 13.0 + 0.5 / 10.75 + 3.0 / 13.5) / 4.0;
        let smape = symmetric_mean_absolute_percentage_error(&y, &errors);
        assert!((smape - expected).abs() < 1e-15);
        // A point with y = ŷ = 0 is left out.
        let with_zero = symmetric_mean_absolute_percentage_error(&[0.0, 10.0], &[0.0, 1.0]);
        assert!((with_zero - 1.0 / 9.5).abs() < 1e-15);
    }

    #[test]
    fn the_seasonal_forecast_variance_matches_its_sum() {
        // The closed form against the sum it stands for, σ²·(1 + Σ_{j=1}^{k} c_j²) with
        // c_j = α + αβ·j + γ·[j mod m = 0], at fixed parameters.
        let (alpha, beta, gamma, m, n, sse) = (0.4, 0.3, 0.2, 4usize, 10usize, 5.0);
        let fitted = Fit {
            m,
            n,
            parameters: [alpha, beta, gamma],
            state: State {
                level: 0.0,
                trend: 0.0,
                season: vec![0.0; m],
                sse,
            },
            errors: Vec::new(),
        };
        for horizon in [1.0, 2.0, 3.5, 4.0, 5.0, 9.0, 13.0] {
            let k = (f64::ceil(horizon) as usize).max(1) - 1;
            let sum: f64 = (1..=k)
                .map(|j| {
                    let seasonal = if j % m == 0 { gamma } else { 0.0 };
                    let c = alpha + alpha * beta * j as f64 + seasonal;
                    c * c
                })
                .sum();
            let expected = (sse / n as f64 * (1.0 + sum)).sqrt();
            let closed = fitted.forecast_sd(horizon);
            assert!(
                (closed - expected).abs() < 1e-12,
                "h = {horizon}: {closed} vs {expected}"
            );
        }
    }
}
