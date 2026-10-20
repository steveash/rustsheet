use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::variance::VarianceKind;

/// Excel's correlation coefficient of the numeric pairs `(xs[i], ys[i])`, divided as
/// `COVARIANCE / (STDEV(xs) * STDEV(ys))` with the divisor `kind` picks (`n` or `n - 1`).
///
/// Every sum is a plain left-to-right two-pass sum about the means `Σx/n` and `Σy/n`, with no
/// one-pass/two-pass switch (unlike VAR, statistics.md "VAR/STDEV switch"). CORREL and PEARSON
/// use the population form and RSQ squares the sample form, which is why the same array twice is
/// 1.0000000000000002 for CORREL but 0.9999999999999998 for RSQ: each of these forms reproduces
/// all 29 CORREL/PEARSON and 15 RSQ oracle values of probes/regression-pairs-a,
/// groupby-pivotby-functions(2) and ironcalc `CORREL_SLOPE_INTERCEPT_RSQ_STEYX.xlsx`, and every
/// other tried form misses some (statistics.md "CORREL, PEARSON and RSQ ... their last digit",
/// T-1707).
///
/// `None` when there are fewer than two pairs or a series has no spread (Excel `#DIV/0!`).
fn excel_correlation(xs: &[f64], ys: &[f64], kind: VarianceKind) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let (mut sum_xx, mut sum_yy, mut sum_xy) = (0.0, 0.0, 0.0);
    for (&x, &y) in xs.iter().zip(ys) {
        let (dx, dy) = (x - mean_x, y - mean_y);
        sum_xx += dx * dx;
        sum_yy += dy * dy;
        sum_xy += dx * dy;
    }
    let divisor = match kind {
        VarianceKind::Population => n,
        VarianceKind::Sample => n - 1.0,
    };
    let spread = (sum_xx / divisor).sqrt() * (sum_yy / divisor).sqrt();
    let r = (sum_xy / divisor) / spread;
    (spread != 0.0 && r.is_finite()).then_some(r)
}

impl Model {
    /// CORREL and PEARSON (one function in Excel) and RSQ: the paired numbers of the two
    /// arguments, or the error to return.
    fn correlation(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        kind: VarianceKind,
    ) -> Result<f64, CalcResult> {
        let (_, _, values_left, values_right) = self.fn_get_two_matrices(args, cell)?;
        // Pairs where either side is not a number are skipped.
        let (xs, ys): (Vec<f64>, Vec<f64>) = values_left
            .into_iter()
            .zip(values_right)
            .filter_map(|pair| match pair {
                (Some(x), Some(y)) => Some((x, y)),
                _ => None,
            })
            .unzip();
        excel_correlation(&xs, &ys, kind).ok_or_else(|| {
            CalcResult::new_error(
                Error::DIV,
                cell,
                "Correlation needs two pairs and a spread in each array".to_string(),
            )
        })
    }

    // CORREL(array1, array2)
    pub(crate) fn fn_correl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.correlation(args, cell, VarianceKind::Population) {
            Ok(r) => CalcResult::Number(r),
            Err(error) => error,
        }
    }

    // PEARSON(array1, array2): CORREL under another name, bit for bit (probes/regression-pairs-a
    // column M against L).
    pub(crate) fn fn_pearson(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_correl(args, cell)
    }

    // RSQ(array1, array2): the square of the sample-form correlation, not CORREL squared
    // (`RSQ(J1:J6,B1:B6)` is 1.0000000000000004 while CORREL is 1, probes/regression-pairs-a N23).
    pub(crate) fn fn_rsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.correlation(args, cell, VarianceKind::Sample) {
            Ok(r) => CalcResult::Number(r * r),
            Err(error) => error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{excel_correlation, VarianceKind};

    const POPULATION: VarianceKind = VarianceKind::Population;
    const SAMPLE: VarianceKind = VarianceKind::Sample;

    // Expected values are Excel's (oracle: excel-graph).
    #[test]
    fn the_same_array_twice_is_not_exactly_one() {
        // probes/groupby-pivotby-functions G!B426 (CORREL), -functions2 G!B286 (RSQ).
        let sales = [10.0, 20.0, 30.0, 40.0, 5.0, 7.0, 0.0, 9.0, -3.0, 12.5];
        assert_eq!(
            excel_correlation(&sales, &sales, POPULATION),
            Some(1.0000000000000002)
        );
        let r = excel_correlation(&sales, &sales, SAMPLE).map(|r| r * r);
        assert_eq!(r, Some(0.9999999999999998));
    }

    #[test]
    fn last_digits_follow_the_two_pass_quotient() {
        // ironcalc CORREL_SLOPE_INTERCEPT_RSQ_STEYX.xlsx Sheet1!C2 and F2.
        let x = [1.0, 2.0, 5.0, 8.0, 12.0, 3.0];
        let y = [2.0, 3.0, 5.0, 1.0, 3.0, 4.0];
        assert_eq!(
            excel_correlation(&x, &y, POPULATION),
            Some(-0.13574278487946612)
        );
        let r = excel_correlation(&x, &y, SAMPLE).map(|r| r * r);
        assert_eq!(r, Some(0.018426103646833007));
        // probes/regression-pairs-a L21 and N22: an offset of 1E10 keeps its digits.
        let offset = [
            1e10 + 1.0,
            1e10 + 2.0,
            1e10 + 4.0,
            1e10 + 3.0,
            1e10 + 5.0,
            1e10 + 7.0,
        ];
        let counts = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert_eq!(
            excel_correlation(&offset, &counts, POPULATION),
            Some(0.9402561526801988)
        );
        let small = [2.0, 4.0, 5.0, 4.0, 5.0, 7.0];
        let r = excel_correlation(&small, &offset, SAMPLE).map(|r| r * r);
        assert_eq!(r, Some(0.9174603174602222));
    }

    #[test]
    fn no_spread_or_one_pair_is_none() {
        assert_eq!(excel_correlation(&[1.0], &[2.0], POPULATION), None);
        assert_eq!(
            excel_correlation(&[3.0, 3.0, 3.0], &[1.0, 2.0, 3.0], SAMPLE),
            None
        );
    }
}
