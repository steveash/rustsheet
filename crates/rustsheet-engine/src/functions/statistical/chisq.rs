use crate::calc_result::Range;
use crate::reference_operators::{Ranges, RefArg};
use statrs::distribution::{Continuous, Gamma};

use super::erf::{gamma_half_lower, gamma_half_upper};
use super::incomplete_gamma::{gamma_cdf, gamma_inverse_cdf, gamma_sf};
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
const MAX_DEGREES_OF_FREEDOM: f64 = 10_000_000_000.0;

/// The chi-squared distribution of `df` degrees of freedom, truncated, as statrs builds it (a
/// gamma of shape df/2 and rate 1/2); a df below 1 or above 1E10 is `#NUM!`.
fn chi_squared(df: f64, cell: CellReferenceIndex) -> Result<Gamma, CalcResult> {
    let df = df.trunc();
    if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
        return Err(num_error(cell));
    }
    Gamma::new(df / 2.0, 0.5).map_err(|_| num_error(cell))
}

/// Whether a chi-squared distribution from [`chi_squared`] has one degree of freedom (shape 1/2),
/// where Excel computes from its ERF/ERFC kernels (docs/excel-semantics/statistics.md, T-1778).
fn is_one_degree(dist: &Gamma) -> bool {
    dist.shape() == 0.5
}

/// The right tail P(X > x) of a chi-squared distribution from [`chi_squared`]: at one degree of
/// freedom Q(1/2, x/2) = ERFC.PRECISE(SQRT(x/2)), as Excel's CHISQ.DIST.RT(x,1), CHIDIST and
/// CHISQ.TEST compute it (T-1778, `kmyst-erfc-r9side4`), else statrs' `sf`, capped.
fn chi_squared_sf(dist: &Gamma, x: f64) -> f64 {
    if is_one_degree(dist) {
        gamma_half_upper(x / 2.0)
    } else {
        gamma_sf(dist, x)
    }
}

/// A distribution value as a cell value: `#NUM!` unless finite and not negative.
fn non_negative(value: f64, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(num_error(cell))
    }
}

impl Model {
    /// `CHISQ.DIST(x, deg_freedom, cumulative)`: every argument lifts over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes H152-H156, probes/stat-quantile-lift H27). The cdf
    /// is the capped port of statrs' incomplete gamma (`incomplete_gamma.rs`), which the lifting
    /// table charges (kinds.rs `INCOMPLETE_GAMMA_WEIGHT`).
    pub(crate) fn fn_chisq_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ChisqDist, args, cell, &mut |model, values| {
            number_or_error(model.chisq_dist_of(values, cell))
        })
    }

    fn chisq_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let df = self.cast_number_ref(df, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if x < 0.0 {
            return Err(num_error(cell));
        }
        let dist = chi_squared(df, cell)?;
        let result = if cumulative && is_one_degree(&dist) {
            gamma_half_lower(x / 2.0)
        } else if cumulative {
            gamma_cdf(&dist, x)
        } else {
            dist.pdf(x)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `CHISQ.DIST.RT(x, deg_freedom)`, and CHIDIST: both arguments lift over an array and a
    /// range (SPEC-0011; probes/stat-argument-modes H132-H139, probes/stat-quantile-lift
    /// H29-H32). The right tail is statrs' `sf`, capped (`incomplete_gamma.rs`).
    pub(crate) fn fn_chisq_dist_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::ChisqDistRT, args, cell, &mut |model, values| {
            number_or_error(model.chisq_dist_rt_of(values, cell))
        })
    }

    fn chisq_dist_rt_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let df = self.cast_number_ref(df, cell)?;
        if x < 0.0 {
            return Err(num_error(cell));
        }
        let dist = chi_squared(df, cell)?;
        non_negative(chi_squared_sf(&dist, x), cell)
    }

    /// `CHISQ.INV(probability, deg_freedom)`: both arguments lift over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes H144-H147, probes/stat-quantile-lift H33). The
    /// quantile is the capped port of statrs' search (`incomplete_gamma.rs`).
    pub(crate) fn fn_chisq_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ChisqInv, args, cell, &mut |model, values| {
            number_or_error(model.chisq_inv_of(values, false, cell))
        })
    }

    /// `CHISQ.INV.RT(probability, deg_freedom)`, and CHIINV: the quantile of `1 - probability`
    /// (SPEC-0011; probes/stat-argument-modes H140-H143, H148-H151, probes/stat-quantile-lift
    /// H35-H38).
    pub(crate) fn fn_chisq_inv_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::ChisqInvRT, args, cell, &mut |model, values| {
            number_or_error(model.chisq_inv_of(values, true, cell))
        })
    }

    fn chisq_inv_of(
        &self,
        values: &[CalcResult],
        right_tail: bool,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, df] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        let df = self.cast_number_ref(df, cell)?;
        // probability outside [0, 1] is #NUM!.
        if !(0.0..=1.0).contains(&p) {
            return Err(num_error(cell));
        }
        let dist = chi_squared(df, cell)?;
        // The right tail: p = P(X > x) = 1 - CDF(x), so x = inverse_cdf(1 - p).
        let p = if right_tail { 1.0 - p } else { p };
        non_negative(gamma_inverse_cdf(&dist, p), cell)
    }

    /// The values of a range, row-major, for a caller that keeps the numbers: a number is `Some`,
    /// text and booleans are `None`, blank cells give no entry, and the first error cell is the
    /// `Err`. The range is walked over each sheet's used area, so a whole column of an almost
    /// empty sheet, or one stretched by a far cell, costs time as in SUM but no memory (T-1681).
    /// Entries are not one per cell: callers that pair two lists by position use
    /// [`Self::paired_values`] instead.
    pub(crate) fn values_from_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        self.values_from_areas(&[Range { left, right }], left)
    }

    /// One entry per non-blank cell of `areas`, in formula order: a number is `Some`, text and
    /// booleans are `None`, and the first error cell is the `Err`. Blank cells give no entry, so
    /// callers must not pair entries by position (none does: they keep the numbers). Areas are
    /// counted as written, so an overlap counts twice (docs/excel-semantics/references.md
    /// "Multi-area references in functions"; probes/multiarea-rank, multiarea-cond).
    pub(crate) fn values_from_areas(
        &mut self,
        areas: &[Range],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let mut values = Vec::new();
        self.for_each_cell_or_error_in_areas(areas, cell, |_, value| match value {
            CalcResult::Number(v) => {
                values.push(Some(v));
                Ok(())
            }
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::EmptyCell => Ok(()),
            _ => {
                values.push(None);
                Ok(())
            }
        })?;
        Ok(values)
    }

    /// The numbers LARGE, SMALL and RANK rank: a reference (one range, or the areas of a union, each
    /// counted as written; probes/multiarea-rank), an array, or a scalar typed in place.
    pub(crate) fn numbers_from_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        self.numbers_from_ranked_arg(node, cell, false)
    }

    /// [`Self::numbers_from_reference_arg`] for the RANK family, whose `ref` must be a reference:
    /// an array that is not a reference (`IF(..)`, arithmetic) is an array of its own shape holding
    /// `#VALUE!`, or the element's error where the element is one (docs/excel-semantics/statistics.md,
    /// "RANK of a non-reference array"; probes/rank-nonref-12..14). LARGE and SMALL accept such arrays.
    pub(crate) fn numbers_from_rank_ref_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        self.numbers_from_ranked_arg(node, cell, true)
    }

    fn numbers_from_ranked_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        arrays_are_value_errors: bool,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let by_reference = matches!(node.ungrouped(), Node::ReferenceKind { .. });
        // The `@` of a non-reference array is its top-left value, which the RANK family takes as
        // it takes the whole array: `#VALUE!`, or the element's error, in one cell
        // (`RANK(5,@IF(A1:B2>0,A1:B2))`, `RANK(5,@{3,4})`; probes/rank-at-if-ref H1, H31, K28).
        let at_value = arrays_are_value_errors
            && matches!(node.ungrouped(), Node::ImplicitIntersection { .. });
        let unsupported = |error: Error| {
            Err(CalcResult::Error {
                error,
                origin: cell,
                message: "Unsupported argument type".to_string(),
            })
        };
        // A cached range of another workbook is a reference: its cached numbers count and its
        // text, booleans and blanks do not (`RANK(5,[1]Uses!$A$1:$B$3)` is 2;
        // fixtures/external-link-cached-ranges Refs!B71).
        if let Some(cells) = self.external_reference_cells(node, cell) {
            return cells?
                .into_iter()
                .map(|value| match value {
                    ArrayNode::Number(number) => Ok(Some(number)),
                    ArrayNode::Error(error) => Err(CalcResult::new_error(
                        error,
                        cell,
                        "Error in external reference".to_string(),
                    )),
                    _ => Ok(None),
                })
                .collect();
        }
        let single = match self.evaluate_reference_arg(node, cell) {
            RefArg::Ranges(areas @ Ranges::Areas(_)) => {
                return self.values_from_areas(&areas, cell)
            }
            single => single.into_single_area(cell),
        };
        match single {
            // Excel keeps an error element's own error and makes every other element #VALUE!
            // (probes/rank-nonref-error-elements).
            CalcResult::Array(array) if arrays_are_value_errors => Err(CalcResult::Array(
                array
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|node| match node {
                                ArrayNode::Error(error) => ArrayNode::Error(error.clone()),
                                _ => ArrayNode::Error(Error::VALUE),
                            })
                            .collect()
                    })
                    .collect(),
            )),
            CalcResult::Array(array) => {
                self.values_from_array(array)
                    .map_err(|e| CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: format!("Unsupported array argument: {e}"),
                    })
            }
            CalcResult::Range { left, right } => self.values_from_range(left, right),
            CalcResult::Boolean(_)
            | CalcResult::Number(_)
            | CalcResult::String(_)
            | CalcResult::EmptyCell
            | CalcResult::EmptyArg
                if at_value =>
            {
                unsupported(Error::VALUE)
            }
            // A cell holding a scalar is a reference the function cannot rank (#NUM!); the same
            // scalar typed in place is a one-element list.
            CalcResult::Boolean(_) | CalcResult::Number(_) | CalcResult::String(_)
                if by_reference =>
            {
                unsupported(Error::NUM)
            }
            CalcResult::Boolean(value) => Ok(vec![Some(if value { 1.0 } else { 0.0 })]),
            CalcResult::Number(value) => Ok(vec![Some(value)]),
            CalcResult::String(value) => match self.cast_number(&value) {
                Some(parsed) => Ok(vec![Some(parsed)]),
                None => unsupported(Error::VALUE),
            },
            // An error value is the function's error: a legacy `LARGE(B1:B6*1,1)` off the range's
            // rows is `#VALUE!` (probes/legacy-value-argument-intersection P10, T-1526).
            error @ CalcResult::Error { .. } => Err(error),
            _ => unsupported(Error::NIMPL),
        }
    }

    pub(crate) fn values_from_array(
        &mut self,
        array: Vec<Vec<ArrayNode>>,
    ) -> Result<Vec<Option<f64>>, Error> {
        let mut values = Vec::new();
        for row in array {
            for item in row {
                match item {
                    ArrayNode::Number(f) => {
                        values.push(Some(f));
                    }
                    ArrayNode::Error(error) => {
                        return Err(error);
                    }
                    _ => {
                        values.push(None);
                    }
                }
            }
        }
        Ok(values)
    }

    // CHISQ.TEST(actual_range, expected_range)
    pub(crate) fn fn_chisq_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (width, height, values_left, values_right) = match self.fn_get_two_matrices(args, cell)
        {
            Ok(v) => v,
            Err(r) => return r,
        };

        let mut values = Vec::with_capacity(values_left.len());

        // Now we have:
        // - values: flattened (observed, expected)
        // - width, height: shape
        for (left_value, right_value) in values_left.iter().zip(&values_right) {
            match (*left_value, *right_value) {
                (Some(v1), Some(v2)) => {
                    values.push((v1, v2));
                }
                _ => {
                    values.push((1.0, 1.0));
                }
            }
        }
        if width * height < 2 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "CHISQ.TEST requires at least two data points".to_string(),
            );
        }

        let mut chi2 = 0.0;
        for (obs, exp) in &values {
            if *obs < 0.0 || *exp < 0.0 {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Negative value in CHISQ.TEST data".to_string(),
                );
            }
            if *exp == 0.0 {
                return CalcResult::new_error(
                    Error::DIV,
                    cell,
                    "Zero expected value in CHISQ.TEST".to_string(),
                );
            }
            let diff = obs - exp;
            chi2 += (diff * diff) / exp;
        }

        if chi2 < 0.0 && chi2 > -1e-12 {
            chi2 = 0.0;
        }

        let total = width * height;
        if total <= 1 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "CHISQ.TEST degrees of freedom is zero".to_string(),
            );
        }

        let df = if width > 1 && height > 1 {
            (width - 1) * (height - 1)
        } else {
            total - 1
        };

        let dist = match Gamma::new(df as f64 / 2.0, 0.5) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid degrees of freedom in CHISQ.TEST".to_string(),
                );
            }
        };

        // The capped cdf. A sheet's cell count keeps the shape under 2^33, where the series can
        // need about 630,000 terms, past the 2^19 cap, so a very large test is #NUM! (the cap's
        // DEVIATION). A statistic whose half underflows is a cdf of 0 (incomplete_gamma.rs), so p 1.
        let mut p = if is_one_degree(&dist) {
            chi_squared_sf(&dist, chi2)
        } else {
            1.0 - gamma_cdf(&dist, chi2)
        };
        if p.is_nan() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.TEST".to_string(),
            );
        }

        // clamp tiny fp noise
        if p < 0.0 && p > -1e-15 {
            p = 0.0;
        }
        if p > 1.0 && p < 1.0 + 1e-15 {
            p = 1.0;
        }

        CalcResult::Number(p)
    }
}
