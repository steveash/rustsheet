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
//! **Not checked against Excel** (decision 0221). Adapted from gridcraft's `ets.rs` (`initial_states`,
//! `filter`, `optimise`, `fit`, `Fit::forecast`); the MIT notice is in the parent module. Excel does
//! not document its optimiser, so α, β and γ, and the last digits of a forecast on noisy data, may
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
    let gammas: &[f64] = if active == 3 { &GRID } else { &GRID[..1] };
    let mut best: (Parameters, f64) = ([0.5, 0.1, 0.1], f64::INFINITY);
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
    let mut simplex: Vec<(Parameters, f64)> = vec![best];
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
    let towards = |from: Parameters, to: Parameters, t: f64| -> Parameters {
        let mut out = [0.0; 3];
        for ((out, a), b) in out.iter_mut().zip(from).zip(to) {
            *out = a + t * (b - a);
        }
        clamp(out, active)
    };
    for _ in 0..NELDER_MEAD_ITERATIONS {
        simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
        let (Some(&lowest), Some(&highest)) = (simplex.first(), simplex.last()) else {
            break;
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
            break;
        }
        let others = simplex.len().saturating_sub(1);
        let mut centroid = [0.0; 3];
        for vertex in simplex.iter().take(others) {
            for (c, value) in centroid.iter_mut().zip(vertex.0) {
                *c += value / others.max(1) as f64;
            }
        }
        let reflected = towards(centroid, highest.0, -1.0);
        let reflected_value = objective(reflected);
        let replacement = if reflected_value < lowest.1 {
            let expanded = towards(centroid, highest.0, -2.0);
            let expanded_value = objective(expanded);
            Some(if expanded_value < reflected_value {
                (expanded, expanded_value)
            } else {
                (reflected, reflected_value)
            })
        } else if reflected_value < second_highest {
            Some((reflected, reflected_value))
        } else {
            let contracted = towards(centroid, highest.0, 0.5);
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
                    let shrunk = towards(lowest.0, vertex.0, 0.5);
                    *vertex = (shrunk, objective(shrunk));
                }
            }
        }
    }
    simplex.sort_by(|a, b| a.1.total_cmp(&b.1));
    clamp(simplex.first().map_or(best.0, |vertex| vertex.0), active)
}

/// A fitted model.
#[derive(Clone, Debug)]
pub(in crate::functions::statistical) struct Fit {
    /// The season length, 0 without seasonality.
    m: usize,
    /// How many points it was fitted to.
    n: usize,
    state: State,
}

/// Fits the model to `y` with season length `m` (below 2: no seasonality).
pub(in crate::functions::statistical) fn fit(y: &[f64], m: usize) -> Fit {
    let m = if m >= 2 { m } else { 0 };
    let initial = initial_states(y, m);
    let parameters = optimise(y, &initial);
    let state = filter(y, &initial, parameters, None);
    Fit {
        m,
        n: y.len(),
        state,
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
}
