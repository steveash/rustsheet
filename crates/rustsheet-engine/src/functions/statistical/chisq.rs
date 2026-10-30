use crate::calc_result::Range;
use crate::nested_array::nested_array_error;
use crate::reference_operators::{Ranges, RefArg};
use statrs::distribution::{Continuous, Gamma};

use super::element_errors::array_element_error;
use super::erf::{gamma_half_lower, gamma_half_upper};
use super::gamma::tiny_ratio_density;
use super::incomplete_gamma::{gamma_cdf, gamma_density, gamma_quantile, gamma_sf, Tail};
use crate::arithmetic::flush_subnormal;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
const MAX_DEGREES_OF_FREEDOM: f64 = 10_000_000_000.0;

/// The scale of every chi-squared distribution: a gamma of shape df/2 and scale 2 (rate 1/2).
const CHI_SQUARED_SCALE: f64 = 2.0;

/// The chi-squared distribution of `df` degrees of freedom, truncated, as statrs builds it (a
/// gamma of shape df/2 and rate 1/2); a df below 1 or above 1E10 is `#NUM!`.
fn chi_squared(df: f64, cell: CellReferenceIndex) -> Result<Gamma, CalcResult> {
    let df = df.trunc();
    if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
        return Err(num_error(cell));
    }
    Gamma::new(df / 2.0, 1.0 / CHI_SQUARED_SCALE).map_err(|_| num_error(cell))
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

/// Below `RIGHT_TAIL_P_MIN[df - 3]` Excel's CHISQ.INV.RT and CHIINV at 3 to 42 degrees of freedom
/// are `#NUM!`, though the quantile exists. 1E-k there is still a value, and 1E-(k+1) is `#NUM!`
/// (probes-numeric/t2561-chisq-inv-rt-num-boundary, -num-edge, -num-large-df and -num-large-df-edge,
/// excel-graph: every p = 1E-k from 1E-20 to 1E-300 at each df bracketing its edge, 1,320 cells,
/// one edge per df with no value below it). `CHISQ.INV.RT(1E-20,3)` = 96.2391239380938 and
/// `(1E-21,3)` is `#NUM!`; `(1E-291,42)` is a value and `(1E-292,42)` `#NUM!`; from 43 degrees of
/// freedom every p to 1E-300 is a value (`CHISQ.INV.RT(1E-300,50)` = 1592.6841019848946, T-1543).
/// The edge rises by 3 to 10 decades a degree of freedom with no closed form found (it is
/// neither a fixed quantile, a fixed multiple of df, nor a fixed distance from the
/// Wilson-Hilferty start; docs/excel-semantics/statistics.md, T-2561). It is Excel's search
/// giving up, not its cdf: CHISQ.DIST.RT is a plain number up to 7% past each edge
/// (probes-numeric/t2561-chisq-dist-rt-past-edge). Each entry is a fit inside its probed decade:
/// any threshold in (1E-(k+1), 1E-k] scores the same, and a p strictly inside a decade is unprobed.
const RIGHT_TAIL_P_MIN: [f64; 40] = [
    1e-20, 1e-27, 1e-35, 1e-43, 1e-52, 1e-62, 1e-72, 1e-81, 1e-87, 1e-90, 1e-94, 1e-99, 1e-104,
    1e-110, 1e-116, 1e-121, 1e-127, 1e-133, 1e-139, 1e-146, 1e-152, 1e-158, 1e-165, 1e-171, 1e-178,
    1e-184, 1e-191, 1e-198, 1e-205, 1e-212, 1e-219, 1e-226, 1e-234, 1e-241, 1e-249, 1e-257, 1e-265,
    1e-273, 1e-282, 1e-291,
];

/// Whether Excel's right-tail quantile of `p` at `df` (truncated) degrees of freedom is `#NUM!`
/// below its search's edge ([`RIGHT_TAIL_P_MIN`]).
fn right_tail_beyond_edge(df: f64, p: f64) -> bool {
    // df is a whole number here; at most 40 entries are scanned.
    (3_u8..)
        .zip(RIGHT_TAIL_P_MIN)
        .find(|&(entry_df, _)| f64::from(entry_df) == df)
        .is_some_and(|(_, p_min)| p < p_min)
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
    /// table charges (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT`).
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
            // CHISQ.DIST's density is GAMMA.DIST's at scale 2, its tiny-ratio log form included:
            // `CHISQ.DIST(1E-300,1,FALSE)` is within 1 ulp of Excel's 3.9894228040142567E+149,
            // which is 154 ulp below the exact 1/SQRT(2*PI()*1E-300) (gen-m9/3e98b26cf6713e77
            // A268, T-2638).
            let scale = 1.0 / dist.rate();
            tiny_ratio_density(dist.shape(), x, scale)
                .or_else(|| gamma_density(dist.shape(), x, scale))
                .unwrap_or_else(|| dist.pdf(x))
        };
        // A subnormal cdf or density is 0, as in Excel (`(x)=0` TRUE: probes/
        // t2706-subnormal-density-paths B35, probes/t2737-subnormal-cdf-shapes B15:B17).
        let result = flush_subnormal(result);
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
        if !right_tail {
            return non_negative(
                gamma_quantile(dist.shape(), CHI_SQUARED_SCALE, p, Tail::Lower),
                cell,
            );
        }
        // At one degree of freedom Excel's right-tail quantile is the left one of 1 - p:
        // `CHISQ.INV.RT(1E-10,1)` is `CHISQ.INV(0.9999999999,1)`, 41.82145620298276, and below
        // 1.1E-16 (1 - p rounds to 1) it is `#NUM!` (T-1543 probe, T-1634).
        if is_one_degree(&dist) {
            return non_negative(
                gamma_quantile(dist.shape(), CHI_SQUARED_SCALE, 1.0 - p, Tail::Lower),
                cell,
            );
        }
        if right_tail_beyond_edge(dist.shape() * 2.0, p) {
            return Err(num_error(cell));
        }
        non_negative(
            gamma_quantile(dist.shape(), CHI_SQUARED_SCALE, p, Tail::Upper),
            cell,
        )
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

    /// [`Self::numbers_from_reference_arg`] of an element-wise expression of whole columns or of
    /// bounded spans past the used area, read as its used head and its tail row (SPEC-0020
    /// Amendment 1 section 3, T-0492): the head's first error, else the tail's error, else the
    /// head's numbers and `tail_len` copies of a numeric tail; what the materialized array gives,
    /// whose elements come in that order. `None` when the argument is not tailed: then it is read
    /// the ordinary way (its cells already evaluated if a head was read). LARGE and SMALL only;
    /// the RANK family makes arrays `#VALUE!` and keeps the ordinary reading.
    pub(crate) fn tailed_numbers_from_reference_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Option<Result<Vec<Option<f64>>, CalcResult>> {
        let (head, tail) = self
            .read_tailed(node, cell, |model| model.evaluate_reference_arg(node, cell))?
            .elements()?;
        // One column, the head's elements and the tail element once, read by the ordinary
        // reading's kernels: their first error is the head's, else the tail's.
        let mut column: Vec<ArrayNode> = head.collect();
        let mut repeats = 0;
        if let Some((element, tail_len)) = tail {
            repeats = tail_len;
            column.push(element);
        }
        let column = vec![column];
        if let Some(error) = nested_array_error(&column, cell) {
            return Some(Err(error));
        }
        let mut numbers = match self.values_from_array(column) {
            Ok(numbers) => numbers,
            Err(error) => return Some(Err(array_element_error(error, cell))),
        };
        // A numeric tail counts once per tail row; any other tail adds nothing. No tail when
        // the pass is being abandoned: the value is discarded.
        if repeats > 0 {
            if let Some(Some(number)) = numbers.pop() {
                numbers.extend(std::iter::repeat_n(Some(number), repeats));
            }
        }
        Some(Ok(numbers))
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
        let by_reference = matches!(*node.ungrouped_at(cell.row), Node::ReferenceKind { .. });
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
            // An error element is the result, whichever element it is (statistics.md "An error
            // element of an array argument is that error"; gen-m9, T-2255).
            CalcResult::Array(array) => {
                if let Some(error) = nested_array_error(&array, cell) {
                    return Err(error);
                }
                self.values_from_array(array)
                    .map_err(|error| array_element_error(error, cell))
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
                    // A LAMBDA element read as a value: `INTERCEPT(..,BYCOL(..,LAMBDA(x,LAMBDA(w,w))))`
                    // is `#VALUE!` (gen-m9/ee2497a5e683c5d9 A124).
                    ArrayNode::Lambda(_) | ArrayNode::Nested(_) => return Err(Error::VALUE),
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

        // A pair with a non-number on either side adds nothing; the degrees of freedom still come
        // from the first range's shape (statistics.md, CHITEST text rule; for two different shapes
        // that choice is UNVERIFIED, TODO(T-2650)). Pairs of only text and
        // blanks are 1.0 when both sides hold numbers that are never paired (gen-m9/8dd2312c96effb1b
        // A343); a side with no number at all is #DIV/0! (checked below).
        let values: Vec<(f64, f64)> = values_left
            .iter()
            .zip(&values_right)
            .map(|(left, right)| match (*left, *right) {
                (Some(v1), Some(v2)) => (v1, v2),
                _ => (1.0, 1.0),
            })
            .collect();
        // Excel: a scalar or a single value is #N/A (probes-numeric/t1049-chisqtest A36
        // `CHISQ.TEST(5,5)`, probes/regression-pairs-text-scalar K1 `CHISQ.TEST("a","a")`).
        if width * height < 2 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "CHISQ.TEST requires at least two data points".to_string(),
            );
        }

        // Excel: a side holding no number at all is #DIV/0!, not p = 1
        // (probes-numeric/t1049-chisqtest A23 `CHISQ.TEST(B1:B4,Z1:Z4)` and A24, Z1:Z4 blank;
        // m9-legacy-chitest A28 `CHITEST(P1:P3,G1:G3)`; gen-m9/0a9226016e8f5f33 A333
        // `CHISQ.TEST(Data!A10:B12,Data!A8:C9)`, a side of text, a boolean and blanks).
        if values_left.iter().all(Option::is_none) || values_right.iter().all(Option::is_none) {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "CHISQ.TEST range holds no number".to_string(),
            );
        }

        let mut chi2 = 0.0;
        for (obs, exp) in &values {
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
        // Excel: there is no per-cell sign check, only a negative summed statistic is #NUM!:
        // a negative actual (`CHITEST(Data!D7:D9,Data!C10:E10)`, c6cfda6bd10cae93 A375) and a
        // negative expected cell with a positive total (f2bd5f380ec002bf A314, 0) are numbers,
        // while probes-numeric/m9-legacy-chitest A21 (total below zero) is #NUM!.
        if chi2 < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Negative CHISQ.TEST statistic".to_string(),
            );
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

        // The capped right tail, taken directly for every df and not as 1 - P, which cancels digits
        // as p gets small: CHITEST(F1:F3,G1:G3) at chi2 = 18, df 2 is EXP(-9) in Excel, and 1 - P is
        // off in the 13th digit (probes-numeric/m9-legacy-chitest A23; statistics.md "Incomplete
        // gamma: accurate kernel", T-0971, T-3058). A sheet's cell count keeps the shape under 2^33,
        // where the series can need about 630,000 terms, past the 2^19 cap, so a very large test is
        // #NUM! (the cap's DEVIATION). A statistic whose half underflows gives p 1.
        let mut p = chi_squared_sf(&dist, chi2);
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
