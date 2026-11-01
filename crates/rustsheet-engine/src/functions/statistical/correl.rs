use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The moments of the pairs of two paired arguments that are both numbers, about their means:
/// what CORREL, PEARSON, RSQ, SLOPE, INTERCEPT, STEYX and FORECAST compute from.
///
/// Excel sums the squares and products of the deviations from the means (two passes), not
/// `n*Σxy - Σx*Σy`: data with a large offset keeps its variance (`CORREL(I1:I6,B1:B6)` with
/// I = 10000000001.. is 0.9402561526801988, not `#DIV/0!`), and every result of
/// probes/regression-pairs-a and -b matches to the bit (docs/excel-semantics/statistics.md
/// "Regression pairs: sums about the means").
pub(super) struct PairedMoments {
    /// How many pairs are both numbers.
    pub(super) n: f64,
    pub(super) mean_x: f64,
    pub(super) mean_y: f64,
    /// Σ(x - x̄)².
    pub(super) sxx: f64,
    /// Σ(y - ȳ)².
    pub(super) syy: f64,
    /// Σ(x - x̄)(y - ȳ).
    pub(super) sxy: f64,
}

impl PairedMoments {
    /// The moments of the pairs of `xs` and `ys` (aligned by position) that are both numbers.
    pub(super) fn new(xs: &[Option<f64>], ys: &[Option<f64>]) -> Self {
        let pairs = || xs.iter().zip(ys).filter_map(|(x, y)| Some(((*x)?, (*y)?)));
        let (mut n, mut sum_x, mut sum_y) = (0.0, 0.0, 0.0);
        for (x, y) in pairs() {
            n += 1.0;
            sum_x += x;
            sum_y += y;
        }
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for (x, y) in pairs() {
            let (dx, dy) = (x - mean_x, y - mean_y);
            sxx += dx * dx;
            syy += dy * dy;
            sxy += dx * dy;
        }
        PairedMoments {
            n,
            mean_x,
            mean_y,
            sxx,
            syy,
            sxy,
        }
    }

    /// `#NUM!` when a sum of squares of the deviations overflows to infinity, with at least two
    /// pairs: Excel checks it before the zero-variance `#DIV/0!`, so `CORREL` of x = 1E200,
    /// 2E200, 3E200 against a constant y is `#NUM!`, not `#DIV/0!`, while a single pair stays
    /// `#DIV/0!` (probes/correl-inexact-constant-overflow Z31-Z79; statistics.md "Regression
    /// pairs: sums about the means"). `needs_y` is false where only x's sum matters: SLOPE with
    /// y of 1E200 scale is finite.
    fn overflow(&self, needs_y: bool) -> Result<(), Error> {
        let overflowed = !self.sxx.is_finite() || (needs_y && !self.syy.is_finite());
        if self.n >= 2.0 && overflowed {
            Err(Error::NUM)
        } else {
            Ok(())
        }
    }

    /// The correlation coefficient with the covariance and standard deviations taken over
    /// `divisor`: CORREL and PEARSON divide by `n`, RSQ squares the one over `n - 1`; that is
    /// the order Excel's last digits follow (probes/regression-pairs-a L, M, N columns).
    /// `#NUM!` on overflow (see [`Self::overflow`]), else `#DIV/0!` with fewer than two pairs
    /// or a constant series.
    pub(super) fn correlation(&self, divisor: f64) -> Result<f64, Error> {
        self.overflow(true)?;
        if self.n < 2.0 || self.sxx == 0.0 || self.syy == 0.0 {
            return Err(Error::DIV);
        }
        let r = (self.sxy / divisor) / ((self.sxx / divisor).sqrt() * (self.syy / divisor).sqrt());
        if r.is_finite() {
            Ok(r)
        } else {
            Err(Error::DIV)
        }
    }

    /// The slope of the regression line of y on x: `#NUM!` when x's sum of squares overflows,
    /// else `#DIV/0!` with fewer than two pairs or all x equal.
    pub(super) fn slope(&self) -> Result<f64, Error> {
        self.overflow(false)?;
        if self.n < 2.0 || self.sxx == 0.0 {
            return Err(Error::DIV);
        }
        let slope = self.sxy / self.sxx;
        if slope.is_finite() {
            Ok(slope)
        } else {
            Err(Error::DIV)
        }
    }

    /// The intercept and slope of the regression line of y on x.
    pub(super) fn line(&self) -> Result<(f64, f64), Error> {
        let slope = self.slope()?;
        let intercept = self.mean_y - slope * self.mean_x;
        if intercept.is_finite() {
            Ok((intercept, slope))
        } else {
            Err(Error::DIV)
        }
    }
}

/// The error result of a [`PairedMoments`] failure: `#NUM!` for an overflow, else `#DIV/0!`.
pub(super) fn moment_error(error: Error, cell: CellReferenceIndex) -> CalcResult {
    let message = if error == Error::NUM {
        "Invalid data"
    } else {
        "Divide by 0"
    };
    CalcResult::new_error(error, cell, message.to_string())
}

impl Model {
    /// The moments of the two paired arguments of a regression function, `known_y's` then
    /// `known_x's` (CORREL's correlation is the same either way round).
    pub(super) fn paired_moments(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<PairedMoments, CalcResult> {
        let (_, _, known_y, known_x) = self.fn_get_two_matrices(args, cell)?;
        Ok(PairedMoments::new(&known_x, &known_y))
    }

    /// CORREL and PEARSON: the correlation coefficient (Excel computes both alike).
    pub(super) fn correlation_coefficient(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        match moments.correlation(moments.n) {
            Ok(r) => CalcResult::Number(r),
            Err(error) => moment_error(error, cell),
        }
    }

    // CORREL(array1, array2) - Returns the correlation coefficient of two data sets
    pub(crate) fn fn_correl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.correlation_coefficient(args, cell)
    }

    // SLOPE(known_y's, known_x's) - Returns the slope of the linear regression line
    pub(crate) fn fn_slope(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        match moments.slope() {
            Ok(slope) => CalcResult::Number(slope),
            Err(error) => moment_error(error, cell),
        }
    }

    // INTERCEPT(known_y's, known_x's) - Returns the y-intercept of the linear regression line
    pub(crate) fn fn_intercept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        match moments.line() {
            Ok((intercept, _)) => CalcResult::Number(intercept),
            Err(error) => moment_error(error, cell),
        }
    }

    // STEYX(known_y's, known_x's) - Returns the standard error of the predicted y-values
    pub(crate) fn fn_steyx(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        let div = || CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        // x's overflow is `#NUM!` even with two points, before the n < 3 `#DIV/0!`
        // (probes/correl-inexact-constant-overflow Z65, Z69); n - 2 degrees of freedom make two
        // points otherwise `#DIV/0!` (probes/regression-pairs-b M4).
        let slope = moments.slope();
        if let Err(Error::NUM) = slope {
            return moment_error(Error::NUM, cell);
        }
        if moments.n < 3.0 || slope.is_err() {
            return div();
        }
        // Excel: Σ(y - ȳ)² less the explained part, sxy * sxy / sxx in that order
        // (probes/regression-pairs-b M column to the bit). An exact fit whose difference rounds
        // below zero is #NUM!, the square root of a negative, not 0 (probes/steyx-exact-fit A8).
        let variance = (moments.syy - moments.sxy * moments.sxy / moments.sxx) / (moments.n - 2.0);
        if variance < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data".to_string());
        }
        let sey = variance.sqrt();
        if sey.is_finite() {
            CalcResult::Number(sey)
        } else {
            div()
        }
    }
}
