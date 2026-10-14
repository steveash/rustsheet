use statrs::distribution::{Continuous, ContinuousCDF, FisherSnedecor};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::t_dist::sample_var;
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
    let mut hi = 1.0_f64;
    let mut doublings = 0;
    while dist.cdf(hi) < p {
        hi *= 2.0;
        doublings += 1;
        if doublings > 1100 {
            return f64::INFINITY;
        }
    }
    let mut lo = 0.0_f64;
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if dist.cdf(mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

impl Model {
    // FISHER(x) = 0.5 * ln((1 + x) / (1 - x))
    pub(crate) fn fn_fisher(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        if x <= -1.0 || x >= 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "x must be between -1 and 1 (exclusive) in FISHER".to_string(),
            };
        }

        let ratio = (1.0 + x) / (1.0 - x);
        let result = 0.5 * ratio.ln();

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for FISHER".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // FISHERINV(y) = (e^(2y) - 1) / (e^(2y) + 1), evaluated as written (not tanh)
    pub(crate) fn fn_fisher_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let y = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel evaluates (exp(2y)-1)/(exp(2y)+1) in doubles, not a correctly rounded tanh, and
        // is 1 once exp(2y) overflows (docs/excel-semantics/statistics.md, "FISHERINV is
        // (EXP(2y)-1)/(EXP(2y)+1)").
        let e = (2.0 * y).exp();
        let result = if e.is_infinite() {
            1.0
        } else {
            (e - 1.0) / (e + 1.0)
        };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for FISHERINV".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // F.DIST(x, deg_freedom1, deg_freedom2, cumulative)
    pub(crate) fn fn_f_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
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

        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        // Excel domain checks
        if x < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "x must be >= 0 in F.DIST".to_string());
        }
        if !f_df_ok(df1) || !f_df_ok(df2) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.DIST".to_string(),
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

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_f_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // F.DIST.RT(x, deg_freedom1, deg_freedom2)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
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

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in F.DIST.RT".to_string(),
            );
        }
        if !f_df_ok(df1) || !f_df_ok(df2) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.DIST.RT".to_string(),
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

        // Right-tail probability: P(F > x) = 1 - CDF(x)
        let result = 1.0 - dist.cdf(x);

        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.DIST.RT".to_string(),
            );
        }

        CalcResult::Number(result)
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
