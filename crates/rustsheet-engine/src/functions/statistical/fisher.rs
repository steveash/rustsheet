use statrs::distribution::{Continuous, FisherSnedecor};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::statistical::incomplete_beta::{
    beta_density, beta_reg_complement, beta_reg_upper,
};
use crate::functions::statistical::inverse_beta::{beta_lower_crossing, f_inverse_cdf};
use crate::functions::statistical::special::{ln_gamma, refine_increasing};
use crate::functions::statistical::t_dist::sample_var;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// Largest (truncated) degrees of freedom Excel accepts for the F family (F.DIST, F.DIST.RT,
/// F.INV, F.INV.RT, FDIST, FINV): either df above 1E10 is `#NUM!`, the pdf included. Without the
/// guard `statrs`' quantile search does not finish for a huge df.
/// Excel: see docs/excel-semantics/statistics.md#f-distribution-df-limit-t-1530
const F_DF_MAX: f64 = 1e10;

fn f_df_ok(df: f64) -> bool {
    (1.0..=F_DF_MAX).contains(&df)
}

/// P(F <= x) for the F distribution: `I_y(df1/2, df2/2)` with y = df1 x / (df1 x + df2), its
/// complement df2 / (df1 x + df2) formed directly.
fn f_cdf(df1: f64, df2: f64, x: f64) -> f64 {
    let scaled = df1 * x;
    let total = scaled + df2;
    beta_reg_complement(scaled / total, df2 / total, df1 / 2.0, df2 / 2.0)
}

/// P(F > x): the upper tail of [`f_cdf`]'s incomplete beta, taken from its evaluation (the
/// unreflected side directly, not as 1 - a reflected one) rather than as 1 - [`f_cdf`]
/// (T-0970: `F.DIST.RT(0.7,4,2.4)` is 0.6597222222222223 in `F_DIST.xlsx`).
fn f_sf(df1: f64, df2: f64, x: f64) -> f64 {
    let scaled = df1 * x;
    let total = scaled + df2;
    beta_reg_upper(scaled / total, df2 / total, df1 / 2.0, df2 / 2.0)
}

/// Above this df `statrs`' `inverse_cdf` for the F distribution does not finish (found at 1E10, the
/// same failure as Student-t), so from it the quantile is [`f_quantile_through_beta`]: Excel's
/// BETA.INV mapped to F. Either side of it the quantile is within 2E-13 relative of Excel's at
/// df 99999, 1E5 and 1E6 with p = 1E-12 and 1 - 1E-12 (24 cells,
/// probes-numeric/t1538-f-quantile-switchover).
const F_QUANTILE_STATRS_DF_MAX: f64 = 1e5;

/// Below this probability (of either tail) statrs' search panics (from p = 1.6E-162) or never
/// finishes (about 7% of calls below 3.6E-27, `F.INV(1E-50,6000,4)`; T-1848), so the quantile has no
/// statrs start there.
const F_QUANTILE_STATRS_P_MIN: f64 = 1e-20;

/// Below this lower-tail probability statrs' estimate, where it has one, is kept unrefined: the
/// deep tail of [`f_cdf`] carries exp(a ln y)'s rounding, and refining to its crossing makes
/// `F.INV(1E-20,5,5)` visibly off Excel's 5.081599921802649E-09 (probes/stat-quantile-lift E12),
/// which statrs' search matches to 15 digits (T-0974: 0 cells newly visibly off at 1E-15, 4 at 0).
const F_QUANTILE_REFINE_P_MIN: f64 = 1e-15;

/// Quantile of the F distribution for `p` (the right tail's when `right_tail`) in [0, 1]: statrs'
/// search (the capped port of `inverse_beta.rs`, T-1848) as a start where it is reliable, refined to
/// the crossing of [`f_cdf`] or [`f_sf`] (`special::refine_increasing`) in the upper tail and from a
/// lower-tail probability of [`F_QUANTILE_REFINE_P_MIN`] up. At most 512 Newton steps and 129 cdfs, or 65 cdfs without a
/// start. From a df of [`F_QUANTILE_STATRS_DF_MAX`], Excel's BETA.INV mapped to F (T-1538).
fn f_quantile(df1: f64, df2: f64, p: f64, right_tail: bool) -> f64 {
    let lower = if right_tail { 1.0 - p } else { p };
    if lower <= 0.0 {
        return 0.0;
    }
    if lower >= 1.0 && !right_tail {
        return f64::INFINITY;
    }
    if df1 >= F_QUANTILE_STATRS_DF_MAX || df2 >= F_QUANTILE_STATRS_DF_MAX {
        return if right_tail {
            1.0 / f_quantile_through_beta(df2, df1, p)
        } else {
            f_quantile_through_beta(df1, df2, p)
        };
    }
    // statrs' search is the start where it is reliable; elsewhere the refinement bisects the
    // whole range of doubles.
    let estimate = if (F_QUANTILE_STATRS_P_MIN..=1.0 - F_QUANTILE_STATRS_P_MIN).contains(&lower) {
        f_inverse_cdf(df1, df2, lower)
    } else {
        f64::NAN
    };
    // Each tail's target is the given p or its exact complement 1 - p (p >= 0.5); the smaller one
    // is solved on the T-0970 incomplete beta, as for BETA.INV (T-0974), so a tiny right-tail p
    // keeps the digits 1 - p loses. statrs' estimate is kept unrefined only for a deep lower tail,
    // the tail it was given: for a deep upper tail it solved the rounded 1 - p, which
    // `F.INV.RT(1E-16,9,3)` misses by 7% (Excel 60647458731.09812, gen-m9/0a9226016e8f5f33 A45).
    if estimate.is_finite() && lower < F_QUANTILE_REFINE_P_MIN {
        return estimate;
    }
    if lower > 0.5 {
        let upper = if right_tail { p } else { 1.0 - p };
        refine_increasing(|x| -f_sf(df1, df2, x), -upper, estimate, 0.0, f64::MAX)
    } else {
        refine_increasing(|x| f_cdf(df1, df2, x), lower, estimate, 0.0, f64::MAX)
    }
}

/// The lower-tail F quantile as Excel forms it at a huge df, from its BETA.INV:
/// (df2/df1) y/(1 - y) with y = BETA.INV(p, df1/2, df2/2) (`inverse_beta::beta_quantile_crossing`),
/// its flipped root s = 1 - y taken as it is for p above 1/2. F.INV.RT is 1/F.INV(p, df2, df1).
/// So y near 1 keeps only its distance from 1 on the 2^-53 grid, and `F.INV(0.5,1E10,5)` is
/// Excel's 1.1490395521130337 (probes-numeric/t1538-f-inv-huge-df A3, from
/// `BETA.INV(0.5,5E9,2.5)` = 0.999999999564854, A134) where the true quantile is
/// 1.1490395821369862.
/// See docs/excel-semantics/statistics.md "F quantile at a huge df goes through Excel's BETA.INV
/// (T-1538)".
fn f_quantile_through_beta(df1: f64, df2: f64, p: f64) -> f64 {
    if p > 0.5 {
        // BETA.INV's flip: s = 1 - y is the root solved, and mapped as it is, not as 1 - (1 - s):
        // `F.INV(1-1E-12,100000,5)` is then within 1.6E-15 relative of Excel's 97570.08944108484
        // (t1538-f-quantile-switchover A9); 1 - (1 - s) keeps only s's distance from 1 and misses by
        // 9E-8 relative.
        let s = beta_lower_crossing(df2 / 2.0, df1 / 2.0, 1.0 - p);
        return (df2 / df1) * ((1.0 - s) / s);
    }
    let y = beta_lower_crossing(df1 / 2.0, df2 / 2.0, p);
    (df2 / df1) * (y / (1.0 - y))
}

/// The F density. statrs' direct form raises `df1*x` and `df1*x + df2` to the power `df1` and
/// `df1 + df2`, which overflows to inf/inf (NaN) for a huge x although the density is tiny:
/// Excel's `F.DIST(1E308,1,3,FALSE)` is 0 (probes/stat-overflow-arguments E11, -b A6). Only in
/// that case, where the overflow comes from x alone (`df1^df1`, `df2^df2` and
/// `(df1+df2)^(df1+df2)` are finite, so the direct form does not overflow at x near 1), is the
/// density evaluated in logarithms, and only an underflow to 0 is taken from it: a density that
/// is a positive double stays `#NUM!` as before, because no oracle cell covers it (moderate x
/// with df1 near 30-140, e.g. `F.DIST(20,100,3,FALSE)`) and the logarithmic form misses there by
/// up to about 300 ulp (TODO(T-1864)). Degrees of freedom large enough to overflow those powers
/// (`F.DIST(3,50,100,FALSE)`, `F.DIST(2,5,1E10,FALSE)`) overflow the direct form too; they stay
/// `#NUM!` as before, because the logarithmic form is not accurate enough there (Excel's
/// `F.DIST(2,5,1E10,FALSE)` is 0.14167277676179968, probes-numeric/t1530-f-df-cut; TODO(T-1864)).
fn f_pdf(dist: &FisherSnedecor, df1: f64, df2: f64, x: f64) -> f64 {
    let direct = dist.pdf(x);
    // Where the direct form is finite: the beta density at y = df1 x / (df1 x + df2) times
    // dy/dx = y (1 - y) / x, while Gamma((df1 + df2)/2) is finite. `F.DIST(0.7,4,2,FALSE)` is
    // then 1 ulp from Excel's 0.40509259259259256 (`F_DIST.xlsx` F3), statrs' form 28 ulp (T-0970).
    if direct.is_finite() {
        let scaled = df1 * x;
        let y = scaled / (scaled + df2);
        return beta_density(y, df1 / 2.0, df2 / 2.0)
            .map_or(direct, |density| density * y * (1.0 - y) / x);
    }
    let total = df1 + df2;
    let small_powers =
        df1.powf(df1).is_finite() && df2.powf(df2).is_finite() && total.powf(total).is_finite();
    if x <= 0.0 || !small_powers {
        return direct;
    }
    let scaled = df1 * x;
    let ln_beta = ln_gamma(df1 / 2.0) + ln_gamma(df2 / 2.0) - ln_gamma((df1 + df2) / 2.0);
    let ln_density = 0.5 * (df1 * scaled.ln() + df2 * df2.ln() - (df1 + df2) * (scaled + df2).ln())
        - x.ln()
        - ln_beta;
    // Only the evidence-backed underflow: ln of the smallest positive double is about -744.44.
    let density = ln_density.exp();
    if density == 0.0 {
        0.0
    } else {
        direct
    }
}

impl Model {
    /// `FISHER(x) = 0.5 * ln((1 + x) / (1 - x))`: lifts over an array and a range (SPEC-0011;
    /// probes/stat-argument-modes H252-H254).
    pub(crate) fn fn_fisher(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Fisher, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x <= -1.0 || x >= 1.0 {
                    return Err(num_error(cell));
                }
                let result = 0.5 * ((1.0 + x) / (1.0 - x)).ln();
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }

    /// `FISHERINV(y) = (e^(2y) - 1) / (e^(2y) + 1)`, evaluated as written (not tanh): lifts over
    /// an array and a range (probes/stat-argument-modes H255-H257).
    pub(crate) fn fn_fisher_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::FisherInv, args, cell, &mut |model, values| {
            let [y] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(y, cell).and_then(|y| {
                // Excel evaluates (exp(2y)-1)/(exp(2y)+1) in doubles, not a correctly rounded
                // tanh, and is 1 once exp(2y) overflows (docs/excel-semantics/statistics.md,
                // "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1)").
                let e = (2.0 * y).exp();
                let result = if e.is_infinite() {
                    1.0
                } else {
                    (e - 1.0) / (e + 1.0)
                };
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }

    /// `F.DIST(x, deg_freedom1, deg_freedom2, cumulative)`: every argument lifts over an array
    /// and a range (probes/stat-argument-modes H215-H220, probes/stat-later-argument-lift H69).
    /// The cdf is [`f_cdf`], an incomplete beta of at most 300 continued-fraction steps.
    pub(crate) fn fn_f_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::FDist, args, cell, &mut |model, values| {
            number_or_error(model.f_dist_of(values, cell))
        })
    }

    fn f_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df1, df2, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Excel reads a TRUE as 1 in x and the degrees of freedom
        // (probes/stat-later-argument-lift L13, probes-numeric/t1049-fdistrt A19, A26).
        let x = self.cast_number_ref(x, cell)?;
        let df1 = self.cast_number_ref(df1, cell)?.trunc();
        let df2 = self.cast_number_ref(df2, cell)?.trunc();
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        // Excel domain checks
        if x < 0.0 || !f_df_ok(df1) || !f_df_ok(df2) {
            return Err(num_error(cell));
        }
        // A finite x whose scaled ratio overflows (df1*x past f64::MAX) is #DIV/0! for the cdf and
        // #NUM! for the pdf in Excel (`F.DIST(1E308,2,3,TRUE)` and `F.DIST(1E308,2,3,FALSE)`,
        // while df1 = 1 gives 1 and 0; probes/stat-overflow-arguments A1, A2, B12, E11). statrs'
        // incomplete beta would unwrap the NaN ratio and panic.
        if !(df1 * x).is_finite() {
            return Err(if cumulative {
                div_error(cell)
            } else {
                num_error(cell)
            });
        }
        let dist = FisherSnedecor::new(df1, df2).map_err(|_| num_error(cell))?;
        let result = if cumulative {
            f_cdf(df1, df2, x)
        } else {
            f_pdf(&dist, df1, df2, x)
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `F.DIST.RT(x, deg_freedom1, deg_freedom2)`, and FDIST: every argument lifts over an array
    /// and a range (probes/stat-argument-modes H221-H230, probes/stat-later-argument-lift H71,
    /// H73).
    pub(crate) fn fn_f_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::FDistRT, args, cell, &mut |model, values| {
            number_or_error(model.f_dist_rt_of(values, cell))
        })
    }

    fn f_dist_rt_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df1, df2] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // A TRUE reads as 1, as in F.DIST (probes-numeric/m9-legacy-fdist A90).
        let x = self.cast_number_ref(x, cell)?;
        let df1 = self.cast_number_ref(df1, cell)?.trunc();
        let df2 = self.cast_number_ref(df2, cell)?.trunc();
        if x < 0.0 || !f_df_ok(df1) || !f_df_ok(df2) {
            return Err(num_error(cell));
        }
        // df1*x past f64::MAX is #DIV/0!, as for F.DIST's cdf (`F.DIST.RT(1E308,2,3)` and
        // `FDIST(1E308,2,3)`; probes/stat-overflow-arguments A3, A4).
        if !(df1 * x).is_finite() {
            return Err(div_error(cell));
        }
        let result = f_sf(df1, df2, x);
        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `F.INV(probability, deg_freedom1, deg_freedom2)`: every argument lifts over an array and a
    /// range (SPEC-0011; probes/stat-argument-modes H231-H235, probes/stat-quantile-lift). The
    /// quantile is the capped port of statrs' inversion refined to the cdf's crossing, at most 129
    /// cdfs (`f_quantile`), which the lifting table charges (lift/kinds/statistical.rs
    /// `QUANTILE_WEIGHT`).
    pub(crate) fn fn_f_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::FInv, args, cell, &mut |model, values| {
            number_or_error(model.f_inv_of(values, false, cell))
        })
    }

    /// `F.INV.RT(probability, deg_freedom1, deg_freedom2)`, and FINV: as F.INV, with `p` the
    /// right tail.
    pub(crate) fn fn_f_inv_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::FInvRT, args, cell, &mut |model, values| {
            number_or_error(model.f_inv_of(values, true, cell))
        })
    }

    /// The F.INV kernel; `right_tail` for F.INV.RT and FINV, whose `p` is `P(F > x)`.
    fn f_inv_of(
        &self,
        values: &[CalcResult],
        right_tail: bool,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, df1, df2] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // A boolean reads as 1 or 0 (probes/stat-quantile-lift L2, L19).
        let p = self.cast_number_ref(p, cell)?;
        let df1 = self.cast_number_ref(df1, cell)?.trunc();
        let df2 = self.cast_number_ref(df2, cell)?.trunc();
        // F.INV takes p in [0, 1], the right tail p in (0, 1]; outside is #NUM!.
        let p_out = if right_tail {
            p <= 0.0 || p > 1.0
        } else {
            !(0.0..=1.0).contains(&p)
        };
        if p_out || !f_df_ok(df1) || !f_df_ok(df2) {
            return Err(num_error(cell));
        }
        let x = f_quantile(df1, df2, p, right_tail);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    // F.TEST(array1, array2)
    pub(crate) fn fn_f_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let values1_opts = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Range { left, right } => match self.values_from_range(left, right) {
                Ok(v) => v,
                Err(error) => return error,
            },
            CalcResult::Array(a) => match self.values_from_array(a) {
                Ok(v) => v,
                Err(error) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        format!("Error in first array: {error:?}"),
                    );
                }
            },
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "First argument must be a range or array".to_string(),
                );
            }
        };

        // Get second sample as Vec<Option<f64>>
        let values2_opts = match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Range { left, right } => match self.values_from_range(left, right) {
                Ok(v) => v,
                Err(error) => return error,
            },
            CalcResult::Array(a) => match self.values_from_array(a) {
                Ok(v) => v,
                Err(error) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        format!("Error in second array: {error:?}"),
                    );
                }
            },
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Second argument must be a range or array".to_string(),
                );
            }
        };

        let values1: Vec<f64> = values1_opts.into_iter().flatten().collect();
        let values2: Vec<f64> = values2_opts.into_iter().flatten().collect();

        let n1 = values1.len();
        let n2 = values2.len();

        // If fewer than 2 numeric values in either sample -> #DIV/0!
        if n1 < 2 || n2 < 2 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "F.TEST requires at least two numeric values in each sample".to_string(),
            );
        }

        let v1 = sample_var(&values1);
        let v2 = sample_var(&values2);

        if v1 <= 0.0 || v2 <= 0.0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Variance of one sample is zero in F.TEST".to_string(),
            );
        }

        let f = v1 / v2;
        let df1 = (n1 - 1) as f64;
        let df2 = (n2 - 1) as f64;

        if FisherSnedecor::new(df1, df2).is_err() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid parameters for F distribution in F.TEST".to_string(),
            );
        }

        // Two-tailed: twice the smaller tail. The smaller tail is not always the one a variance
        // ratio above 1 picks (F(df1, df2) has its median away from 1), so take the minimum
        // (F.TEST(C1:C6,B1:B8) of probes/stat-argument-modes is 0.9196, not 2 - 0.9196; T-1292).
        let lower = f_cdf(df1, df2, f);
        let mut p = 2.0 * lower.min(f_sf(df1, df2, f));

        // Clamp tiny FP noise
        if p < 0.0 && p > -1e-15 {
            p = 0.0;
        }
        if p > 1.0 && p < 1.0 + 1e-15 {
            p = 1.0;
        }

        CalcResult::Number(p)
    }
}

#[cfg(test)]
mod tests {
    use super::f_quantile;

    // Excel's F quantile at a huge df is its BETA.INV mapped to F (probes-numeric, excel-graph).
    #[test]
    fn huge_df_quantiles_go_through_excels_beta_inv() {
        // t1538-f-inv-huge-df A3: y = BETA.INV(0.5,5E9,2.5) near 1, bit for bit.
        assert_eq!(
            f_quantile(1e10, 5.0, 0.5, false).to_bits(),
            1.149_039_552_113_033_7_f64.to_bits()
        );
        // A48: F.INV.RT(0.975,1E10,5) = 1/F.INV(0.975,5,1E10), through the flipped root.
        let rt = f_quantile(1e10, 5.0, 0.975, true);
        assert!(
            (rt - 0.389_635_637_664_716_87).abs() <= 4.0 * f64::EPSILON * rt,
            "{rt}"
        );
        // t1538-f-quantile-switchover A9: the flipped root mapped as it is (2E-15 relative of Excel's;
        // through 1 - (1 - s) it would be 9E-8 off).
        let upper = f_quantile(100_000.0, 5.0, 1.0 - 1e-12, false);
        assert!(
            (upper - 97_570.089_441_084_84).abs() <= 2e-15 * upper,
            "{upper}"
        );
    }
}
