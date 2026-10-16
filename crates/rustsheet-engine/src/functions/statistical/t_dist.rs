use statrs::distribution::{Continuous, ContinuousCDF, Normal, StudentsT};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::special::{bisect_increasing, ln_gamma};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

fn mean(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n == 0 {
        return 0.0;
    }
    let mut s = 0.0;
    for &x in xs {
        s += x;
    }
    s / (n as f64)
}

pub(crate) fn sample_var(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n < 2 {
        return 0.0;
    }
    let m = mean(xs);
    let mut s = 0.0;
    for &x in xs {
        let d = x - m;
        s += d * d;
    }
    s / ((n - 1) as f64)
}

enum TTestType {
    Paired,
    TwoSampleEqualVar,
    TwoSampleUnequalVar,
}

enum TTestTails {
    OneTailed,
    TwoTailed,
}

/// Largest (truncated) `deg_freedom` for which Excel evaluates the cumulative and inverse Student-t
/// functions; above it they are `#NUM!` while the pdf stays finite. Without the guard `statrs`'
/// quantile search does not finish for a huge df (fuzz hang on `T.INV(0.9,5E10)`).
/// Excel: see docs/excel-semantics/statistics.md#student-t-sweep-df-truncation-df-limits-tinv2t-above-1-t-1043
pub(crate) const T_DF_MAX: f64 = 1e10;

/// Above this df `statrs`' `inverse_cdf` is both inaccurate and unbounded in time (seconds at 1E7,
/// no finish at 1E8), and its cdf is only good to ~1E-6 relative, so the quantile comes from the
/// normal quantile and the A&S 26.7.5 series instead.
const T_QUANTILE_SERIES_DF: f64 = 1e5;

/// Probabilities below this take the tail path of [`t_tail_quantile`] instead of `statrs`.
const T_TAIL_P: f64 = 1e-5;

/// Below this `x = df / (df + t^2)` the incomplete beta `I_x(df/2, 1/2)` equals its leading term and
/// the first correction to better than 1E-16, so the tail quantile has a closed form.
const T_TAIL_X: f64 = 1e-8;

/// Below this probability the quantile comes from bisection on the cdf, not `statrs`' `inverse_cdf`,
/// which hangs at 1E-100 for df = 1000 and panics from about 1E-200 (the sweep test covers df 2 to
/// 9E4). Above it `statrs` is bit-exact with Excel where x is not tiny (`T_DIST.xlsx` `T.INV(0.9999999,10)`).
const T_STATRS_MIN_P: f64 = 1e-8;

/// Magnitude `t > 0` with `P(T < -t) = q` for a small `q`, `df < T_QUANTILE_SERIES_DF`; bounded time.
/// Excel gives finite values down to the smallest p it accepts (probes-numeric `t1531-t-tiny-p`:
/// `T.INV(1E-300,1)` = -3.183098861837907E299). Where `x` is not tiny `statrs` is bit-exact with
/// Excel (`T_DIST.xlsx` `T.INV(0.9999999,10)`), so it is kept there.
fn t_tail_quantile(dist: &StudentsT, df: f64, q: f64) -> f64 {
    let a = 0.5 * df;
    // P(T < -t) = I_x(a, 1/2) / 2 with I_x ~ x^a / (a B(a, 1/2)) * (1 + a / (2 (a + 1)) x).
    let ln_b = ln_gamma(a) + ln_gamma(0.5) - ln_gamma(a + 0.5);
    let mut ln_x = ((2.0 * q).ln() + a.ln() + ln_b) / a;
    if ln_x < T_TAIL_X.ln() {
        let x = ln_x.exp();
        ln_x -= (a / (2.0 * (a + 1.0)) * x).ln_1p() / a;
        let x = ln_x.exp();
        return df.sqrt() * (-0.5 * ln_x).exp() * (1.0 - x).sqrt();
    }
    if q >= T_STATRS_MIN_P {
        return -dist.inverse_cdf(q);
    }
    // x is not small and q is tiny: the quantile is moderate, so bisect on the cdf; -cdf(-t) is
    // increasing in t and -cdf(-t) >= -q exactly when P(T < -t) <= q.
    bisect_increasing(|t| -dist.cdf(-t), -q, 64)
}

/// Quantile `t` with `P(T > t) = q`. For a tiny `q`, `1 - q` rounds to 1, so the tail is asked for
/// directly (T-1531); otherwise this is `t_quantile(1 - q)` as before.
pub(crate) fn t_upper_quantile(dist: &StudentsT, df: f64, q: f64) -> f64 {
    if q < T_TAIL_P {
        -t_quantile(dist, df, q)
    } else {
        t_quantile(dist, df, 1.0 - q)
    }
}

/// Quantile of Student's t with `df` degrees of freedom for `p` in (0,1); the time is bounded for
/// every df up to [`T_DF_MAX`].
pub(crate) fn t_quantile(dist: &StudentsT, df: f64, p: f64) -> f64 {
    if df < T_QUANTILE_SERIES_DF {
        // Below the tail threshold `statrs` is fine; above it its inverse saturates (T.INV(1E-10,1)
        // gives -8.9E7, Excel -3.18E9) and panics from p about 1E-200 (T-1531).
        let (q, sign) = if p < 0.5 { (p, -1.0) } else { (1.0 - p, 1.0) };
        if q < T_TAIL_P {
            return sign * t_tail_quantile(dist, df, q);
        }
        return dist.inverse_cdf(p);
    }
    let Ok(normal) = Normal::new(0.0, 1.0) else {
        return f64::NAN;
    };
    let z = normal.inverse_cdf(p);
    // Abramowitz & Stegun 26.7.5; the neglected term is O(z^11 / df^5).
    let (z2, z3) = (z * z, z * z * z);
    let z5 = z3 * z2;
    let z7 = z5 * z2;
    let z9 = z7 * z2;
    let g1 = (z3 + z) / 4.0;
    let g2 = (5.0 * z5 + 16.0 * z3 + 3.0 * z) / 96.0;
    let g3 = (3.0 * z7 + 19.0 * z5 + 17.0 * z3 - 15.0 * z) / 384.0;
    let g4 = (79.0 * z9 + 776.0 * z7 + 1482.0 * z5 - 1920.0 * z3 - 945.0 * z) / 92160.0;
    z + g1 / df + g2 / df.powi(2) + g3 / df.powi(3) + g4 / df.powi(4)
}

impl Model {
    // T.DIST(x, deg_freedom, cumulative)
    pub(crate) fn fn_t_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        if df < 1.0 || (cumulative && df > T_DF_MAX) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom out of range in T.DIST".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST".to_string(),
                }
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // T.DIST.2T(x, deg_freedom)
    pub(crate) fn fn_t_dist_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
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

        if x < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "x must be >= 0 in T.DIST.2T".to_string(),
            };
        }

        if !(1.0..=T_DF_MAX).contains(&df) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom must be >= 1 in T.DIST.2T".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST.2T".to_string(),
                }
            }
        };

        let upper_tail = 1.0 - dist.cdf(x);
        let mut result = 2.0 * upper_tail;

        result = result.clamp(0.0, 1.0);

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST.2T".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // T.DIST.RT(x, deg_freedom)
    pub(crate) fn fn_t_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
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

        if !(1.0..=T_DF_MAX).contains(&df) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom must be >= 1 in T.DIST.RT".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST.RT".to_string(),
                }
            }
        };

        let result = 1.0 - dist.cdf(x);

        if !result.is_finite() || result < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST.RT".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // T.INV(probability, deg_freedom)
    pub(crate) fn fn_t_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        if p <= 0.0 || p >= 1.0 || !(1.0..=T_DF_MAX).contains(&df) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for T.INV".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.INV".to_string(),
                }
            }
        };

        let x = t_quantile(&dist, df, p);

        if !x.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.INV".to_string(),
            };
        }

        CalcResult::Number(x)
    }

    // T.INV.2T(probability, deg_freedom)
    pub(crate) fn fn_t_inv_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        if p <= 0.0 || p > 1.0 || !(1.0..=T_DF_MAX).contains(&df) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for T.INV.2T".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.INV.2T".to_string(),
                }
            }
        };

        // Two-sided: F(x) = 1 - p/2
        let x = t_upper_quantile(&dist, df, p / 2.0);

        if !x.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.INV.2T".to_string(),
            };
        }

        CalcResult::Number(x.abs())
    }

    // T.TEST(array1, array2, tails, type)
    pub(crate) fn fn_t_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let samples = match self.paired_samples(args, cell) {
            Ok(samples) => samples,
            Err(error) => return error,
        };

        let tails = match self.get_number(&args[2], cell) {
            Ok(f) => {
                let tf = f.trunc();
                if tf == 1.0 {
                    TTestTails::OneTailed
                } else if tf == 2.0 {
                    TTestTails::TwoTailed
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "tails must be 1 or 2".to_string(),
                    );
                }
            }
            Err(e) => return e,
        };
        let test_type = match self.get_number(&args[3], cell) {
            Ok(f) => {
                let tf = f.trunc();
                match tf {
                    1.0 => TTestType::Paired,
                    2.0 => TTestType::TwoSampleEqualVar,
                    3.0 => TTestType::TwoSampleUnequalVar,
                    _ => {
                        return CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "type must be 1, 2, or 3".to_string(),
                        );
                    }
                }
            }
            Err(e) => return e,
        };

        let (values1, values2): (Vec<f64>, Vec<f64>) = if matches!(test_type, TTestType::Paired) {
            // The pairs zipping the samples makes; a pair with a non-number side is skipped.
            let pairs = samples
                .first
                .into_iter()
                .zip(samples.second)
                .take(samples.paired);
            pairs
                .filter_map(|pair| match pair {
                    (Some(v1), Some(v2)) => Some((v1, v2)),
                    _ => None,
                })
                .unzip()
        } else {
            // keep only numeric entries, ignore non-numeric (Option::None)
            let v1: Vec<f64> = samples.first.into_iter().flatten().collect();
            let v2: Vec<f64> = samples.second.into_iter().flatten().collect();
            (v1, v2)
        };

        let n1 = values1.len();
        let n2 = values2.len();

        if n1 == 0 || n2 == 0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "T.TEST requires non-empty samples".to_string(),
            );
        }

        let (t_stat, df) = match test_type {
            TTestType::Paired => {
                if n1 != n2 {
                    return CalcResult::new_error(
                        Error::NA,
                        cell,
                        "For paired T.TEST, both samples must have the same length".to_string(),
                    );
                }
                if n1 < 2 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Paired T.TEST requires at least two pairs".to_string(),
                    );
                }

                let mut diffs = Vec::with_capacity(n1);
                for i in 0..n1 {
                    diffs.push(values1[i] - values2[i]);
                }

                let nd = diffs.len();
                let md = mean(&diffs);
                let vd = sample_var(&diffs);
                if vd <= 0.0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Zero variance in paired T.TEST".to_string(),
                    );
                }
                let sd = vd.sqrt();
                let t_stat = md / (sd / (nd as f64).sqrt());
                let df = (nd - 1) as f64;
                (t_stat, df)
            }

            // 2: two-sample, equal variance (homoscedastic)
            TTestType::TwoSampleEqualVar => {
                if n1 < 2 || n2 < 2 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Two-sample T.TEST type 2 requires at least two values in each sample"
                            .to_string(),
                    );
                }

                let m1 = mean(&values1);
                let m2 = mean(&values2);
                let v1 = sample_var(&values1);
                let v2 = sample_var(&values2);

                let df_i = (n1 + n2 - 2) as i32;
                if df_i <= 0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Degrees of freedom must be positive in T.TEST type 2".to_string(),
                    );
                }
                let df = df_i as f64;

                let sp2 = (((n1 - 1) as f64) * v1 + ((n2 - 1) as f64) * v2) / df; // pooled variance

                if sp2 <= 0.0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Zero pooled variance in T.TEST type 2".to_string(),
                    );
                }

                let denom = (sp2 * (1.0 / (n1 as f64) + 1.0 / (n2 as f64))).sqrt();
                if denom == 0.0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Zero denominator in T.TEST type 2".to_string(),
                    );
                }

                let t_stat = (m1 - m2) / denom;
                (t_stat, df)
            }

            // two-sample, unequal variance (Welch)
            TTestType::TwoSampleUnequalVar => {
                if n1 < 2 || n2 < 2 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Two-sample T.TEST type 3 requires at least two values in each sample"
                            .to_string(),
                    );
                }

                let m1 = mean(&values1);
                let m2 = mean(&values2);
                let v1 = sample_var(&values1);
                let v2 = sample_var(&values2);

                let s1n = v1 / (n1 as f64);
                let s2n = v2 / (n2 as f64);
                let denom = (s1n + s2n).sqrt();
                if denom == 0.0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Zero denominator in T.TEST type 3".to_string(),
                    );
                }

                let t_stat = (m1 - m2) / denom;

                let num_df = (s1n + s2n).powi(2);
                let den_df = (s1n * s1n) / ((n1 - 1) as f64) + (s2n * s2n) / ((n2 - 1) as f64);
                if den_df == 0.0 {
                    return CalcResult::new_error(
                        Error::DIV,
                        cell,
                        "Invalid degrees of freedom in T.TEST type 3".to_string(),
                    );
                }
                let df = num_df / den_df;
                (t_stat, df)
            }
        };

        if df <= 0.0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Degrees of freedom must be positive in T.TEST".to_string(),
            );
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Student's t distribution".to_string(),
                );
            }
        };

        let t_abs = t_stat.abs();
        let cdf = dist.cdf(t_abs);

        let mut p = match tails {
            TTestTails::OneTailed => 1.0 - cdf,
            TTestTails::TwoTailed => 2.0 * (1.0 - cdf),
        };

        // clamp tiny fp noise
        if p < 0.0 && p > -1e-15 {
            p = 0.0;
        }
        if p > 1.0 && p < 1.0 + 1e-15 {
            p = 1.0;
        }

        CalcResult::Number(p)
    }

    // TDIST(x, deg_freedom, tails) — x≥0; tails=1→T.DIST.RT, tails=2→T.DIST.2T
    pub(crate) fn fn_tdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if x < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "TDIST: x must be >= 0".to_string());
        }
        let tails = match self.get_number(&args[2], cell) {
            Ok(f) => f.trunc() as i64,
            Err(e) => return e,
        };
        match tails {
            1 => self.fn_t_dist_rt(&args[0..2], cell),
            2 => self.fn_t_dist_2t(&args[0..2], cell),
            _ => CalcResult::new_error(Error::NUM, cell, "TDIST: tails must be 1 or 2".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{t_quantile, t_upper_quantile};
    use statrs::distribution::StudentsT;

    // Excel (probes-numeric t1531-t-tiny-p): the Student-t quantile is finite, never an error, for
    // a tiny p, and statrs' own inverse_cdf panics there. Expected values are Excel's cached values.

    fn dist(df: f64) -> StudentsT {
        StudentsT::new(0.0, 1.0, df).expect("valid df")
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual / expected - 1.0).abs() < 1e-11,
            "actual {actual:e}, expected {expected:e}"
        );
    }

    #[test]
    fn tail_asymptotic_matches_excel() {
        // x = df / (df + t^2) is tiny: the closed-form branch.
        assert_close(
            t_quantile(&dist(1.0), 1.0, 1e-300),
            -3.183_098_861_837_907e299,
        );
        assert_close(t_quantile(&dist(1.0), 1.0, 1e-10), -3_183_098_861.837_906_4);
        assert_close(
            t_quantile(&dist(5.0), 5.0, 1e-300),
            -1.568_392_559_099_332e60,
        );
        assert_close(t_quantile(&dist(5.0), 5.0, 1e-50), -15_683_925_590.993_399);
        // The upper tail is the mirror image (T.INV.2T(1E-300,1) = 2 * 3.18E299).
        assert_close(
            t_upper_quantile(&dist(1.0), 1.0, 5e-301),
            6.366_197_723_675_814e299,
        );
        // CONFIDENCE.T(1E-300,1,2) = t / sqrt(2).
        assert_close(
            t_upper_quantile(&dist(1.0), 1.0, 5e-301) / 2.0_f64.sqrt(),
            4.501_581_580_785_53e299,
        );
    }

    #[test]
    fn statrs_branch_is_kept_where_it_matches_excel() {
        // T_DIST.xlsx `T.INV(0.9999999,10)`: bit-exact with statrs.
        assert_eq!(
            t_quantile(&dist(10.0), 10.0, 0.999_999_9),
            12.492_209_688_254_745
        );
    }

    #[test]
    fn bisection_branch_is_bounded() {
        // x is not tiny and q < 1E-8: bisection on the cdf (T.INV(1E-10,5), 1 ulp from Excel).
        assert_close(t_quantile(&dist(5.0), 5.0, 1e-10), -156.825_592_708_894_3);
        // x is not tiny and q < 1E-100: statrs panics here. Excel gives -79.06351062982456; ours is
        // wrong (the cdf of statrs saturates), so only the bound is pinned (TODO(T-1542)).
        let t = t_quantile(&dist(99999.0), 99999.0, 1e-300);
        assert!(t.is_finite() && t < -30.0 && t > -80.0, "{t}");
    }

    #[test]
    fn no_panic_across_df_and_p() {
        for df in [2.0, 10.0, 100.0, 1e3, 1e4, 9e4] {
            for q in [1e-100, 1e-50, 1e-10, 1e-9, 1e-8, 1e-7, 1e-6, 1e-150, 1e-300] {
                let t = t_quantile(&dist(df), df, q);
                assert!(t.is_finite() && t < 0.0, "df {df} q {q}: {t}");
            }
        }
    }
}
