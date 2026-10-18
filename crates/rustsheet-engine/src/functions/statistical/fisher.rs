use statrs::distribution::{Continuous, ContinuousCDF, FisherSnedecor};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::statistical::special::{bisect_increasing, ln_gamma};
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

/// Above this df `statrs`' `inverse_cdf` for the F distribution does not finish (found at 1E10, the
/// same failure as Student-t), so the quantile is found by bisection on the cdf instead.
const F_QUANTILE_BISECT_DF: f64 = 1e5;

/// Quantile of the F distribution for `p` in [0,1]; the time is bounded for every df up to
/// [`F_DF_MAX`] (at most ~1200 cdf evaluations).
fn f_quantile(dist: &FisherSnedecor, df1: f64, df2: f64, p: f64) -> f64 {
    if df1 < F_QUANTILE_BISECT_DF && df2 < F_QUANTILE_BISECT_DF {
        return dist.inverse_cdf(p);
    }
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // 2^1100 is past the f64 range, so a quantile not bracketed by then is infinite.
    bisect_increasing(|x| dist.cdf(x), p, 1100)
}

/// The F density. statrs' direct form raises `df1*x` and `df1*x + df2` to the power `df1` and
/// `df1 + df2`, which overflows to inf/inf (NaN) for a huge x although the density is tiny:
/// Excel's `F.DIST(1E308,1,3,FALSE)` is 0 (probes/stat-overflow-arguments E11, -b A6). Only in
/// that case, where the overflow comes from x alone (`df1^df1`, `df2^df2` and
/// `(df1+df2)^(df1+df2)` are finite, so the direct form does not overflow at x near 1), is the
/// density evaluated in logarithms. Degrees of freedom large enough to overflow those powers
/// (`F.DIST(3,50,100,FALSE)`, `F.DIST(2,5,1E10,FALSE)`) overflow the direct form too; they stay
/// `#NUM!` as before, because the logarithmic form is not accurate enough there (Excel's
/// `F.DIST(2,5,1E10,FALSE)` is 0.14167277676179968, probes-numeric/t1530-f-df-cut; TODO(T-1864)).
fn f_pdf(dist: &FisherSnedecor, df1: f64, df2: f64, x: f64) -> f64 {
    let direct = dist.pdf(x);
    let total = df1 + df2;
    let small_powers =
        df1.powf(df1).is_finite() && df2.powf(df2).is_finite() && total.powf(total).is_finite();
    if direct.is_finite() || x <= 0.0 || !small_powers {
        return direct;
    }
    let scaled = df1 * x;
    let ln_beta = ln_gamma(df1 / 2.0) + ln_gamma(df2 / 2.0) - ln_gamma((df1 + df2) / 2.0);
    let ln_density = 0.5 * (df1 * scaled.ln() + df2 * df2.ln() - (df1 + df2) * (scaled + df2).ln())
        - x.ln()
        - ln_beta;
    ln_density.exp()
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
    /// The cdf is statrs' incomplete beta, a continued fraction of at most 140 steps.
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
        // TODO(T-1849): Excel reads a TRUE as 1 in x and the degrees of freedom
        // (probes/stat-later-argument-lift L13, probes-numeric/t1049-fdistrt A19, A26).
        let x = self.cast_number_no_bools_ref(x, cell)?;
        let df1 = self.cast_number_no_bools_ref(df1, cell)?.trunc();
        let df2 = self.cast_number_no_bools_ref(df2, cell)?.trunc();
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
            dist.cdf(x)
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
        // TODO(T-1849): a TRUE reads as 1 in Excel, as in F.DIST.
        let x = self.cast_number_no_bools_ref(x, cell)?;
        let df1 = self.cast_number_no_bools_ref(df1, cell)?.trunc();
        let df2 = self.cast_number_no_bools_ref(df2, cell)?.trunc();
        if x < 0.0 || !f_df_ok(df1) || !f_df_ok(df2) {
            return Err(num_error(cell));
        }
        // df1*x past f64::MAX is #DIV/0!, as for F.DIST's cdf (`F.DIST.RT(1E308,2,3)` and
        // `FDIST(1E308,2,3)`; probes/stat-overflow-arguments A3, A4).
        if !(df1 * x).is_finite() {
            return Err(div_error(cell));
        }
        let dist = FisherSnedecor::new(df1, df2).map_err(|_| num_error(cell))?;
        // Right-tail probability: P(F > x) = 1 - CDF(x)
        let result = 1.0 - dist.cdf(x);
        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    // F.INV(probability, deg_freedom1, deg_freedom2)
    pub(crate) fn fn_f_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df1 = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };
        let df2 = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in F.INV".to_string(),
            );
        }
        if !f_df_ok(df1) || !f_df_ok(df2) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.INV".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        let x = f_quantile(&dist, df1, df2, p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid result for F.INV".to_string());
        }

        CalcResult::Number(x)
    }

    // F.INV.RT(probability, deg_freedom1, deg_freedom2)
    pub(crate) fn fn_f_inv_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df1 = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };
        let df2 = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        if p <= 0.0 || p > 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in (0,1] in F.INV.RT".to_string(),
            );
        }
        if !f_df_ok(df1) || !f_df_ok(df2) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.INV.RT".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        // p is right-tail: p = P(F > x) = 1 - CDF(x)
        let x = f_quantile(&dist, df1, df2, 1.0 - p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.INV.RT".to_string(),
            );
        }

        CalcResult::Number(x)
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

        // F ratio: larger variance / smaller variance
        let mut f = v1 / v2;
        let mut df1 = (n1 - 1) as f64;
        let mut df2 = (n2 - 1) as f64;

        if f < 1.0 {
            f = 1.0 / f;
            std::mem::swap(&mut df1, &mut df2);
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution in F.TEST".to_string(),
                );
            }
        };

        // One-tailed right-tail probability
        let tail = 1.0 - dist.cdf(f);
        // F.TEST is two-tailed: p = 2 * tail (with F >= 1)
        let mut p = 2.0 * tail;

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
