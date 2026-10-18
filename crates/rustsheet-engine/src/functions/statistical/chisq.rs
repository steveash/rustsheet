use crate::calc_result::Range;
use crate::reference_operators::{Ranges, RefArg};
use statrs::distribution::{ChiSquared, Continuous, ContinuousCDF};

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
const MAX_DEGREES_OF_FREEDOM: f64 = 10_000_000_000.0;

impl Model {
    // CHISQ.DIST(x, deg_freedom, cumulative)
    pub(crate) fn fn_chisq_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[2], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in CHISQ.DIST".to_string(),
            );
        }
        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.DIST".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    // CHISQ.DIST.RT(x, deg_freedom)
    pub(crate) fn fn_chisq_dist_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in CHISQ.DIST.RT".to_string(),
            );
        }

        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.DIST.RT".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        // Right-tail probability: P(X > x).
        // Use sf(x) directly for better numerical properties than 1 - cdf(x).
        let result = dist.sf(x);

        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.DIST.RT".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    // CHISQ.INV(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV".to_string(),
            );
        }

        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.INV".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        let x = dist.inverse_cdf(p);

        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.INV".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    // CHISQ.INV.RT(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV.RT".to_string(),
            );
        }
        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.INV.RT".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        // Right-tail inverse: p = P(X > x) = SF(x) = 1 - CDF(x)
        // So x = inverse_cdf(1 - p).
        let x = dist.inverse_cdf(1.0 - p);

        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.INV.RT".to_string(),
            );
        }

        CalcResult::Number(x)
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

        let dist = match ChiSquared::new(df as f64) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid degrees of freedom in CHISQ.TEST".to_string(),
                );
            }
        };

        let mut p = 1.0 - dist.cdf(chi2);

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
