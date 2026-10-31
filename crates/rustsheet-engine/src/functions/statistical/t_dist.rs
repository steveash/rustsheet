use statrs::distribution::{Continuous, ContinuousCDF, Normal, StudentsT};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error, LiftCost};
use crate::functions::statistical::inverse_beta::t_inverse_cdf;
use crate::functions::statistical::paired::{MarkedSample, PairedSamplesError, PairedValues};
use crate::functions::statistical::special::{bisect_increasing, ln_gamma, refine_increasing};
use crate::functions::statistical::student_t::{
    student_t_centre, student_t_density, student_t_tail_at, student_t_tails,
};
use crate::functions::Function;
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

/// One T.TEST element over the samples read once: `tails`, then `type`, then the samples' error,
/// then the test.
fn t_test_element(
    model: &Model,
    samples: &Result<PairedValues, PairedSamplesError>,
    values: &[CalcResult],
    cell: CellReferenceIndex,
) -> CalcResult {
    let [_, _, tails, test_type] = values else {
        return CalcResult::new_args_number_error(cell);
    };
    // A sample that is itself an error value, or a blank one-cell reference, comes before tails
    // and type (`T.TEST(<#VALUE!>,42,1,-1)` is `#VALUE!`, `TTEST(<#VALUE!>,<#REF!>,2,<#REF!>)`
    // too, probes/t2475-stat-subexpressions P!A58, P!A222; `T.TEST(E11,A6:C12,2,123456789.123)`
    // with E11 blank is `#VALUE!`, probes/tbill-error-order-inner-calls; the second sample's
    // `#VALUE!` before tails' `#N/A`, gen-m9 9e2225ead8a6a07b A59, and its `#DIV/0!` before
    // tails' `#VALUE!`, 3b7990bd4e3307ba A100), while an error inside a range comes after.
    if let Err(PairedSamplesError {
        error,
        argument_is_error_value: true,
        ..
    }) = samples
    {
        return error.clone();
    }
    // Both are read before either is checked: type's `#VALUE!` beats tails 7
    // (`T.TEST(..,{..},7,<#VALUE!>)`, gen-m9 607e85cfee83a689 A194).
    let tails = match model.kernel_number(tails, cell) {
        Ok(tails) => tails.trunc(),
        Err(error) => return error,
    };
    let test_type = match model.kernel_number(test_type, cell) {
        Ok(test_type) => test_type.trunc(),
        Err(error) => return error,
    };
    let tails = match tails {
        1.0 => TTestTails::OneTailed,
        2.0 => TTestTails::TwoTailed,
        _ => return num_error(cell),
    };
    let test_type = match test_type {
        1.0 => TTestType::Paired,
        2.0 => TTestType::TwoSampleEqualVar,
        3.0 => TTestType::TwoSampleUnequalVar,
        _ => return num_error(cell),
    };
    match samples {
        Ok(samples) => number_or_error(t_test_p(samples, &tails, &test_type, cell)),
        // A paired test of samples of different sizes is `#N/A` before an error cell is
        // (probes/ftest-decomposition G12); the unpaired tests return the error (G13).
        Err(PairedSamplesError {
            sizes_differ: true, ..
        }) if matches!(test_type, TTestType::Paired) => {
            CalcResult::new_error(Error::NA, cell, String::new())
        }
        Err(PairedSamplesError { error, .. }) => error.clone(),
    }
}

/// T.TEST's p-value for valid `tails` and `test_type`.
fn t_test_p(
    samples: &PairedValues,
    tails: &TTestTails,
    test_type: &TTestType,
    cell: CellReferenceIndex,
) -> Result<f64, CalcResult> {
    let error = |error| Err(CalcResult::new_error(error, cell, String::new()));
    let paired = matches!(test_type, TTestType::Paired);
    // For a paired test both samples must have the same size.
    if paired && !samples.same_size {
        return error(Error::NA);
    }
    let (values1, values2): (Vec<f64>, Vec<f64>) = if paired {
        // The pairs zipping the samples makes; a pair with a non-number side is skipped.
        samples
            .first
            .iter()
            .zip(&samples.second)
            .take(samples.paired)
            .filter_map(|pair| match pair {
                (Some(v1), Some(v2)) => Some((*v1, *v2)),
                _ => None,
            })
            .unzip()
    } else {
        // keep only numeric entries, ignore non-numeric (Option::None)
        (
            samples.first.iter().flatten().copied().collect(),
            samples.second.iter().flatten().copied().collect(),
        )
    };

    let n1 = values1.len();
    let n2 = values2.len();
    if n1 == 0 || n2 == 0 {
        return error(Error::DIV);
    }

    let (t_square, df) = match test_type {
        TTestType::Paired => {
            if n1 != n2 {
                return error(Error::NA);
            }
            // At least two pairs.
            if n1 < 2 {
                return error(Error::DIV);
            }
            let diffs: Vec<f64> = values1
                .iter()
                .zip(&values2)
                .map(|(v1, v2)| v1 - v2)
                .collect();
            let nd = diffs.len();
            let md = mean(&diffs);
            let vd = sample_var(&diffs);
            // A sample variance that overflows (data whose squares do) is #NUM!, not a p from an
            // infinite deviation, in every test type (`T.TEST(B6:D12,{1E+200,1E+200,-1E+200},1,2)`,
            // gen-m9 a612e89321a6df9c A375; statistics.md "Overflowing squares (T-2432)").
            if vd.is_infinite() {
                return error(Error::NUM);
            }
            // Zero variance of the differences.
            if vd <= 0.0 {
                return error(Error::DIV);
            }
            // Excel's squared statistic is md^2 / (vd / n): bit-exact on 7 of the 11 paired
            // cells (probes-numeric t1048/t1049-ttest, `T_TEST.xlsx` F2 and F5) and within
            // 4 ulp on the rest; md^2 n / vd misses F2 by 2 ulp.
            let t_square = md * md / (vd / (nd as f64));
            let df = (nd - 1) as f64;
            (t_square, df)
        }

        // 2: two-sample, equal variance (homoscedastic)
        TTestType::TwoSampleEqualVar => {
            // At least two values in each sample.
            if n1 < 2 || n2 < 2 {
                return error(Error::DIV);
            }

            let m1 = mean(&values1);
            let m2 = mean(&values2);
            let v1 = sample_var(&values1);
            let v2 = sample_var(&values2);
            if v1.is_infinite() || v2.is_infinite() {
                return error(Error::NUM);
            }

            let df_i = (n1 + n2 - 2) as i32;
            if df_i <= 0 {
                return error(Error::DIV);
            }
            let df = df_i as f64;

            let sp2 = (((n1 - 1) as f64) * v1 + ((n2 - 1) as f64) * v2) / df; // pooled variance

            if sp2 <= 0.0 {
                return error(Error::DIV);
            }

            let denom = (sp2 * (1.0 / (n1 as f64) + 1.0 / (n2 as f64))).sqrt();
            if denom == 0.0 {
                return error(Error::DIV);
            }

            let t_stat = (m1 - m2) / denom;
            (t_stat * t_stat, df)
        }

        // two-sample, unequal variance (Welch)
        TTestType::TwoSampleUnequalVar => {
            // At least two values in each sample.
            if n1 < 2 || n2 < 2 {
                return error(Error::DIV);
            }

            let m1 = mean(&values1);
            let m2 = mean(&values2);
            let v1 = sample_var(&values1);
            let v2 = sample_var(&values2);
            if v1.is_infinite() || v2.is_infinite() {
                return error(Error::NUM);
            }

            let s1n = v1 / (n1 as f64);
            let s2n = v2 / (n2 as f64);
            let denom = (s1n + s2n).sqrt();
            if denom == 0.0 {
                return error(Error::DIV);
            }

            let t_stat = (m1 - m2) / denom;

            let num_df = (s1n + s2n).powi(2);
            let den_df = (s1n * s1n) / ((n1 - 1) as f64) + (s2n * s2n) / ((n2 - 1) as f64);
            if den_df == 0.0 {
                return error(Error::DIV);
            }
            let df = num_df / den_df;
            (t_stat * t_stat, df)
        }
    };

    if df <= 0.0 {
        return error(Error::DIV);
    }

    // Excel rounds y = df/(df + t^2) to a double before the tail (`student_t_tail_at`).
    let tail = student_t_tail_at(df / (df + t_square), df);
    let mut p = match tails {
        TTestTails::OneTailed => tail,
        TTestTails::TwoTailed => 2.0 * tail,
    };

    // clamp tiny fp noise
    if p < 0.0 && p > -1e-15 {
        p = 0.0;
    }
    if p > 1.0 && p < 1.0 + 1e-15 {
        p = 1.0;
    }

    Ok(p)
}

/// Largest (truncated) `deg_freedom` for which Excel evaluates the cumulative and inverse Student-t
/// functions; above it they are `#NUM!` while the pdf stays finite. Without the guard `statrs`'
/// quantile search does not finish for a huge df (fuzz hang on `T.INV(0.9,5E10)`).
/// Excel: see docs/excel-semantics/statistics.md#student-t-sweep-df-truncation-df-limits-tinv2t-above-1-t-1043
pub(crate) const T_DF_MAX: f64 = 1e10;

/// Whether the cumulative Student-t functions refuse `x`: Excel's T.DIST (cumulative), T.DIST.RT,
/// T.DIST.2T and TDIST of 1E+300 and 1E+307 are `#NUM!`, not 0, while `T.DIST.2T(1E+15,1)` is a number
/// (gen-m9 T-2256 cells; statistics.md "Scalar distributions at extreme arguments, part 2"). The cut,
/// where `x²` overflows (|x| about 1.34E+154), is UNVERIFIED (T-2474).
fn t_x_overflows(x: f64) -> bool {
    (x * x).is_infinite()
}

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
        return -t_inverse_cdf(df, q);
    }
    // x is not small and q is tiny: the quantile is moderate, so bisect on the cdf; -cdf(-t) is
    // increasing in t and -cdf(-t) >= -q exactly when P(T < -t) <= q.
    bisect_increasing(|t| -dist.cdf(-t), -q, 64)
}

/// Central probabilities `c = |2p - 1|` below this take [`t_centre_quantile`]. statrs' inversion
/// solves for `y = df / (df + t^2)` near 1 and back-transforms with `1 - y`, which keeps only the
/// digits of `y` below 1 (`TINV(1-1E-10,5)` 2.36E-8, Excel 1.32E-10; T-2724).
const T_CENTRE_C: f64 = 0.1;

/// Magnitude `t >= 0` with `P(|T| < t) = c` for a central probability `c` below [`T_CENTRE_C`]
/// (so `t` < 0.16 at every df): the crossing of the centre probability itself
/// (`student_t_centre`), which never forms `1 - y`, so a tiny quantile keeps its digits. Excel's
/// centre quantiles are accurate (statistics.md "Student t sweep", T-2724). The estimate is
/// `c / (2 f(0))`, `f` the density; at most 129 evaluations (`refine_increasing`).
fn t_centre_quantile(df: f64, c: f64) -> f64 {
    let estimate = student_t_density(0.0, df).map_or(f64::NAN, |density| 0.5 * c / density);
    refine_increasing(|t| student_t_centre(t, df), c, estimate, 0.0, 1.0)
}

/// Quantile `t` with `P(T > t) = q`. For a tiny `q`, `1 - q` rounds to 1, so the tail is asked for
/// directly (T-1531); near 1/2 the centre is (T-2724); otherwise this is `t_quantile(1 - q)`.
pub(crate) fn t_upper_quantile(dist: &StudentsT, df: f64, q: f64) -> f64 {
    // 1 - 2q is exact for q in [1/4, 1] (Sterbenz) where 1 - q is not for q below 1/2.
    let c = 1.0 - 2.0 * q;
    if c.abs() < T_CENTRE_C {
        return t_centre_quantile(df, c.abs()).copysign(c);
    }
    if q < T_TAIL_P {
        -t_quantile(dist, df, q)
    } else {
        t_quantile(dist, df, 1.0 - q)
    }
}

/// Quantile of Student's t with `df` degrees of freedom for `p` in (0,1); the time is bounded for
/// every df up to [`T_DF_MAX`]. statrs' search is the capped port of `inverse_beta.rs` (T-1848),
/// bit for bit statrs' value where statrs finishes.
pub(crate) fn t_quantile(dist: &StudentsT, df: f64, p: f64) -> f64 {
    // 2p - 1 is exact for p in [1/4, 1] (Sterbenz).
    let c = 2.0 * p - 1.0;
    if c.abs() < T_CENTRE_C {
        return t_centre_quantile(df, c.abs()).copysign(c);
    }
    if df < T_QUANTILE_SERIES_DF {
        // Below the tail threshold `statrs` is fine; above it its inverse saturates (T.INV(1E-10,1)
        // gives -8.9E7, Excel -3.18E9) and panics from p about 1E-200 (T-1531).
        let (q, sign) = if p < 0.5 { (p, -1.0) } else { (1.0 - p, 1.0) };
        if q < T_TAIL_P {
            return sign * t_tail_quantile(dist, df, q);
        }
        return t_inverse_cdf(df, p);
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
    /// `T.DIST(x, deg_freedom, cumulative)`: every argument lifts over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes H505-H509, probes/stat-later-argument-lift H61).
    /// The cdf and the integer-df density come from `student_t` (T-0969); other densities from statrs.
    pub(crate) fn fn_t_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDist, args, cell, &mut |model, values| {
            number_or_error(model.t_dist_of(values, cell))
        })
    }

    fn t_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, df, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let df = self.cast_number_ref(df, cell)?.trunc();
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        // The density divides by the truncated df: df in (-1, 1) is `#DIV/0!`, not `#NUM!`
        // (`T.DIST(2,0.5,FALSE)`, `T.DIST(2,-0.5,FALSE)`; probes-numeric m9-t-edges A27-A41,
        // statistics.md "Student t sweep").
        if !cumulative && df == 0.0 {
            return Err(div_error(cell));
        }
        if df < 1.0 || (cumulative && (df > T_DF_MAX || t_x_overflows(x))) {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            student_t_tails(x, df).0
        } else if let Some(density) = student_t_density(x, df) {
            density
        } else {
            StudentsT::new(0.0, 1.0, df)
                .map_err(|_| num_error(cell))?
                .pdf(x)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.DIST.2T(x, deg_freedom)`: both arguments lift over an array and a range
    /// (probes/stat-argument-modes H510-H513, probes/stat-later-argument-lift H63).
    pub(crate) fn fn_t_dist_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDist2T, args, cell, &mut |model, values| {
            let [x, df] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(
                model
                    .cast_number_ref(x, cell)
                    .and_then(|x| model.t_dist_2t_of(x, df, cell)),
            )
        })
    }

    /// T.DIST.2T once `x` is read. The degrees of freedom are read before `x` is checked
    /// (`T.DIST.2T(-1,NA())` is #N/A; probes/stat-overflow-arguments F1, F5).
    fn t_dist_2t_of(
        &self,
        x: f64,
        df: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let df = self.cast_number_ref(df, cell)?.trunc();
        if x < 0.0 || !(1.0..=T_DF_MAX).contains(&df) || t_x_overflows(x) {
            return Err(num_error(cell));
        }
        let result = 2.0 * student_t_tails(x, df).1;
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.DIST.RT(x, deg_freedom)`: both arguments lift over an array and a range
    /// (probes/stat-argument-modes H514-H517, probes/stat-later-argument-lift H65).
    pub(crate) fn fn_t_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TDistRT, args, cell, &mut |model, values| {
            let [x, df] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(
                model
                    .cast_number_ref(x, cell)
                    .and_then(|x| model.t_dist_rt_of(x, df, cell)),
            )
        })
    }

    /// T.DIST.RT once `x` is read.
    fn t_dist_rt_of(
        &self,
        x: f64,
        df: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let df = self.cast_number_ref(df, cell)?.trunc();
        if !(1.0..=T_DF_MAX).contains(&df) || t_x_overflows(x) {
            return Err(num_error(cell));
        }
        let result = student_t_tails(x, df).1;
        if !result.is_finite() || result < 0.0 {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `T.INV(probability, deg_freedom)`: both arguments lift over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes, probes/stat-quantile-lift). The quantile is the
    /// capped port of statrs' inversion, a closed form, or a bisection of at most 264 cdfs
    /// (`t_quantile`), which the lifting table charges (lift/kinds/statistical.rs
    /// `T_QUANTILE_WEIGHT`).
    pub(crate) fn fn_t_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TInv, args, cell, &mut |model, values| {
            number_or_error(model.t_inv_of(values, false, cell))
        })
    }

    /// `T.INV.2T(probability, deg_freedom)`, and TINV: as T.INV, two-tailed.
    pub(crate) fn fn_t_inv_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::TInv2T, args, cell, &mut |model, values| {
            number_or_error(model.t_inv_of(values, true, cell))
        })
    }

    /// The T.INV kernel; `two_tailed` for T.INV.2T and TINV, which take p in (0, 2) and return
    /// the magnitude with `P(|T| > t) = p` for p up to 1, and the negative quantile of 2 - p
    /// above it.
    fn t_inv_of(
        &self,
        values: &[CalcResult],
        two_tailed: bool,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, df] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        let df = self.cast_number_ref(df, cell)?.trunc();
        // p = 0 with a df that truncates to 1 is `#DIV/0!`, from df 2 up `#NUM!` (`TINV(0,1)`,
        // `T.INV(0,1.999)`, `T.INV.2T(FALSE,1.005)` against `T.INV(0,2)`; statistics.md "Student t
        // quantile at p = 0 with df below 2 is #DIV/0!", probes/t2475-stat-subexpressions P!A124).
        if p == 0.0 && df == 1.0 {
            return Err(div_error(cell));
        }
        let p_out = if two_tailed {
            p <= 0.0 || p >= 2.0
        } else {
            p <= 0.0 || p >= 1.0
        };
        if p_out || !(1.0..=T_DF_MAX).contains(&df) {
            return Err(num_error(cell));
        }
        let dist = StudentsT::new(0.0, 1.0, df).map_err(|_| num_error(cell))?;
        let x = if two_tailed && p > 1.0 {
            // T.INV.2T(p) = T.INV(1 - p/2), the negative quantile, for p in (1, 2) (m9-t-edges A2
            // `T.INV.2T(1.5,5)`, A43 `T.INV.2T(1.5,10)`). 2 - p is exact on [1, 2] (Sterbenz), and
            // the upper tail of (2 - p) / 2 reaches the tiny-p path for p near 2.
            -t_upper_quantile(&dist, df, (2.0 - p) / 2.0).abs()
        } else if two_tailed {
            // Two-sided: F(x) = 1 - p/2
            t_upper_quantile(&dist, df, p / 2.0).abs()
        } else {
            t_quantile(&dist, df, p)
        };
        if !x.is_finite() {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    /// `T.TEST(array1, array2, tails, type)` and TTEST: `tails` and `type` lift over an array
    /// and a multi-cell range, the samples are read once (SPEC-0011, T-1982). An element
    /// truncates and checks `tails`, then `type`, then takes the samples' error:
    /// `T.TEST(D1:D3,A1:A3,3,2)` with D2 = `#N/A` is `#NUM!`, while a lifted element with good
    /// arguments is the samples' `#N/A`; `tails` 3 or `type` 4 is `#NUM!` per element, `1.7` and
    /// `2.9` are 1 and 2, TRUE is 1 (probes/stat-test-lift P60-P67, P88). Each element walks the
    /// samples, so the call is charged their cells per element.
    pub(crate) fn fn_t_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // TTEST shares T.TEST's table entry (`TWO_DATA_THEN_2`).
        let kind = Function::TTest;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // A text sample is an array of one cell with no number, not `#VALUE!`
        // (`TTEST(IFERROR(#VALUE!,""),Data!B11:E11,1,3)` is `#DIV/0!`, gen-m9
        // 46ac3ef5e61583ca A344; statistics.md "T.TEST: an error sample first ...").
        let samples = self
            .paired_samples(args, cell, MarkedSample::NumError)
            .map_err(|error| PairedSamplesError {
                error: error.error.without_message(),
                ..error
            });
        let cells = samples.as_ref().map_or(0, |samples| {
            let cells = samples.first.len().saturating_add(samples.second.len());
            u64::try_from(cells).unwrap_or(u64::MAX)
        });
        self.lift_call_costed(
            kind,
            args,
            // At least one unit an element, so a samples error copied per element is charged too.
            LiftCost::Cells(cells.max(1)),
            cell,
            &mut |model, values| t_test_element(model, &samples, values, cell),
        )
    }

    /// `TDIST(x, deg_freedom, tails)`: x >= 0; tails 1 is T.DIST.RT, tails 2 T.DIST.2T. Every
    /// argument lifts over an array and a range (probes/stat-argument-modes H530-H534,
    /// probes/stat-later-argument-lift H67, L8-L10). It reads all three before it checks any.
    pub(crate) fn fn_tdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::TDistCompat, args, cell, &mut |model, values| {
            number_or_error(model.tdist_of(values, cell))
        })
    }

    fn tdist_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [x, df, tails] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Every argument is read before any is checked: an error in deg_freedom wins over x < 0
        // and over a bad tails (`TDIST(-1,NA(),1)` and `TDIST(1,NA(),3)` are #N/A;
        // probes/stat-overflow-arguments F2, F3, F6).
        let x = self.cast_number_ref(x, cell)?;
        self.cast_number_ref(df, cell)?;
        let tails = self.cast_number_ref(tails, cell)?.trunc() as i64;
        if x < 0.0 {
            return Err(num_error(cell));
        }
        match tails {
            1 => self.t_dist_rt_of(x, df, cell),
            2 => self.t_dist_2t_of(x, df, cell),
            _ => Err(num_error(cell)),
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

    /// Whether `actual` is within half a unit of the 15th significant digit of `expected`.
    fn within_15_digits(actual: f64, expected: f64) -> bool {
        let unit = 10f64.powf(expected.abs().log10().floor() - 14.0);
        (actual - expected).abs() <= 0.5 * unit
    }

    #[test]
    fn centre_quantiles_keep_their_digits() {
        // T-2724: two-tailed quantiles with p near 1, against the true quantile at the double p
        // (mpmath, 40 digits, `I_w(1/2, df/2) = 1 - p`), within 1E-15 relative; statrs' 1 - y lost
        // most digits (`TINV(1-1E-10,5)` was 2.36E-8) and gave 0 at 1-1E-16.
        for (df, p, exact) in [
            (1.0, 0.999_999, 1.570_796_326_841_358e-6),
            (1.0, 1.0 - 1e-10, 1.570_796_456_763_167_3e-10),
            (1.0, 1.0 - 1e-16, 1.743_934_249_004_316e-16),
            (2.0, 0.999_999, 1.414_213_562_414_469e-6),
            (2.0, 1.0 - 1e-10, 1.414_213_679_385_65e-10),
            (2.0, 1.0 - 1e-16, 1.570_092_458_683_775_2e-16),
            (5.0, 0.999_999, 1.317_152_762_108_468_7e-6),
            (5.0, 1.0 - 1e-10, 1.317_152_871_051_844_3e-10),
            (5.0, 1.0 - 1e-16, 1.462_333_323_398_885_8e-16),
            (1000.0, 0.999_999, 1.253_627_505_003_303e-6),
            (1000.0, 1.0 - 1e-10, 1.253_627_608_692_530_5e-10),
            (1000.0, 1.0 - 1e-16, 1.391_806_120_317_668_7e-16),
            (1e6, 0.999_999, 1.253_314_450_680_441_7e-6),
            (1e6, 1.0 - 1e-10, 1.253_314_554_343_776_3e-10),
            (1e6, 1.0 - 1e-16, 1.391_458_560_200_48e-16),
        ] {
            let t = t_upper_quantile(&dist(df), df, p / 2.0);
            assert!(
                (t / exact - 1.0).abs() < 1e-15,
                "T.INV.2T({p},{df}) = {t:e}, exact {exact:e}"
            );
        }
        // Excel's values (gen-m9 b9a32ce9a1fd0d71 A58, 334f3dc9ca937f10 A128, probes-numeric
        // m9-t-inv-2t A117 and A130) to half a unit of the 15th digit.
        for (df, p, excel) in [
            (5.0, 1.0 - 1e-10, 1.317_152_871_051_845_8e-10),
            (7.0, 0.999_999, 1.298_730_137_860_588_3e-6),
            (1000.0, 0.999_999_999_9, 1.253_627_608_692_531_2e-10),
            (1e6, 0.999_999_999_9, 1.253_314_554_343_774_5e-10),
        ] {
            let t = t_upper_quantile(&dist(df), df, p / 2.0);
            assert!(
                within_15_digits(t, excel),
                "T.INV.2T({p},{df}) = {t:e}, Excel {excel:e}"
            );
        }
        // T.INV at p = 1/2 +- 1E-10: the same centre at c = |2p - 1|, odd in p.
        let exact = 2.634_305_742_103_688_5e-10;
        let above = t_quantile(&dist(5.0), 5.0, 0.5 + 1e-10);
        assert!((above / exact - 1.0).abs() < 1e-15, "{above:e}");
        assert_eq!(t_quantile(&dist(5.0), 5.0, 0.5 - 1e-10), -above);
        assert_eq!(t_quantile(&dist(5.0), 5.0, 0.5), 0.0);
    }

    #[test]
    fn centre_quantile_rises_with_c_at_every_df() {
        // The search is bounded (`refine_increasing`); its result is finite and rises with c up to
        // the switch back to statrs.
        for df in [1.0, 2.0, 3.0, 30.0, 1e3, 99_999.0, 1e5, 1e8, 1e10] {
            let mut last = 0.0;
            for c in [1.2e-16, 1e-12, 1e-8, 1e-4, 1e-2, 0.05, 0.099] {
                let t = t_upper_quantile(&dist(df), df, (1.0 - c) / 2.0);
                assert!(t.is_finite() && t > last, "df {df} c {c}: {t}");
                last = t;
            }
        }
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
