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

    /// The correlation coefficient with the covariance and standard deviations taken over
    /// `divisor`: CORREL and PEARSON divide by `n`, RSQ squares the one over `n - 1`; that is
    /// the order Excel's last digits follow (probes/regression-pairs-a L, M, N columns).
    pub(super) fn correlation(&self, divisor: f64) -> Option<f64> {
        if self.n < 2.0 || self.sxx == 0.0 || self.syy == 0.0 {
            return None;
        }
        let r = (self.sxy / divisor) / ((self.sxx / divisor).sqrt() * (self.syy / divisor).sqrt());
        r.is_finite().then_some(r)
    }

    /// The slope of the regression line of y on x, `None` with fewer than two pairs or all x
    /// equal.
    pub(super) fn slope(&self) -> Option<f64> {
        if self.n < 2.0 || self.sxx == 0.0 {
            return None;
        }
        let slope = self.sxy / self.sxx;
        slope.is_finite().then_some(slope)
    }

    /// The intercept and slope of the regression line of y on x.
    pub(super) fn line(&self) -> Option<(f64, f64)> {
        let slope = self.slope()?;
        let intercept = self.mean_y - slope * self.mean_x;
        intercept.is_finite().then_some((intercept, slope))
    }
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
            Some(r) => CalcResult::Number(r),
            None => CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string()),
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
            Some(slope) => CalcResult::Number(slope),
            None => CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string()),
        }
    }

    // INTERCEPT(known_y's, known_x's) - Returns the y-intercept of the linear regression line
    pub(crate) fn fn_intercept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        match moments.line() {
            Some((intercept, _)) => CalcResult::Number(intercept),
            None => CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string()),
        }
    }

    // STEYX(known_y's, known_x's) - Returns the standard error of the predicted y-values
    pub(crate) fn fn_steyx(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        let div = || CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        // n - 2 degrees of freedom: two points are #DIV/0! (probes/regression-pairs-b M4).
        if moments.n < 3.0 || moments.slope().is_none() {
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
