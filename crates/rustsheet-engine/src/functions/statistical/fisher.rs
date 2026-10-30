use statrs::distribution::FisherSnedecor;

use crate::arithmetic::extended_quotient;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::statistical::bratio::{beta_dist_lower_tail, closed_or_ported_tails};
use crate::functions::statistical::f_density::excel_f_density;
use crate::functions::statistical::f_test_tail::f_test_bratio_p;
use crate::functions::statistical::half_integer_beta::half_integer_remainder;
use crate::functions::statistical::incomplete_beta::{
    beta_reg_tails, search_tails, A1_EXPM1_MAX_X,
};
use crate::functions::statistical::inverse_beta::{beta_lower_crossing, f_inverse_cdf};
use crate::functions::statistical::paired::marked_number_error;
use crate::functions::statistical::special::refine_increasing;
use crate::functions::statistical::t_dist::sample_var;
use crate::functions::Function;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;
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

/// The incomplete beta's argument and its complement for the F distribution at `x` as Excel's F
/// functions pass them to the kernel (T-2464 w5-triangulation, T-2502): u = 1/(1 + df2/(df1 x)) and
/// its complement 1/(1 + df1 x/df2), each formed directly. On the b0 = 1 closed-form cells, which
/// depend on the argument alone, this is bit-exact on 197 of 197, against 180 for
/// df1 x/(df1 x + df2) (docs/excel-semantics/statistics.md "Incomplete beta is TOMS 708 `bratio`").
fn f_beta_arguments(df1: f64, df2: f64, x: f64) -> (f64, f64) {
    let scaled = df1 * x;
    (1.0 / (1.0 + df2 / scaled), 1.0 / (1.0 + scaled / df2))
}

/// The incomplete beta's argument df1 x/(df1 x + df2) and its complement df2/(df1 x + df2), the
/// forms today's Cephes kernel takes for the F tails ([`f_tail`]'s fallback, [`f_quantile_tail`]).
fn f_kernel_arguments(df1: f64, df2: f64, x: f64) -> (f64, f64) {
    let scaled = df1 * x;
    let total = scaled + df2;
    (scaled / total, df2 / total)
}

/// Whether the F cdf at `x` is `#NUM!` because the incomplete beta's argument cannot be formed:
/// [`f_beta_arguments`] divides df2 by df1 x, and once that quotient leaves the double range Excel
/// reports `#NUM!` for either tail, where the density there is still 0. `FDIST(1E-300,6,1E+10)`,
/// `F.DIST(1E-300,6,1E+10,TRUE)`, `F.DIST.RT(1E-300,6,1E+10)` and `FDIST(1E-300,1,1E+10)` are
/// `#NUM!`, while `FDIST(1E-300,6,1E+9)` is 1 (quotient 1.67E+308, the largest the cut allows),
/// `FDIST(1E-290,6,1E+10)` is 1, one power of ten the other way is `#NUM!` again
/// (`FDIST(1E-301,6,1E+10)`) and `F.DIST(1E-300,6,1E+10,FALSE)` is 0. The `x > 0.0` guard is what
/// keeps `FDIST(0,6,1E+10)` and a subnormal x out of the quotient: number text below the smallest
/// normal double reads as 0, so `FDIST(1E-308,6,1E+8)` is 1 (numbers.md "Subnormal results become
/// 0"). (probes/t2435-dist-extremes A39, A85:A91, A130, A131, A141, A142; gen-m9
/// c6cfda6bd10cae93 A363.)
/// Excel: see docs/excel-semantics/statistics.md, "The F cdf is #NUM! where df2/(df1 x) overflows (T-2435)"
fn f_beta_ratio_overflows(df1: f64, df2: f64, x: f64) -> bool {
    x > 0.0 && !(df2 / (df1 * x)).is_finite()
}

/// One F tail (`upper`: P(F > x)): Excel's closed forms (a df of 2) and the bratio port's branches
/// on [`f_beta_arguments`] ([`closed_or_ported_tails`], T-2847), else today's Cephes kernel on
/// u = df1 x/(df1 x + df2) and its complement df2/(df1 x + df2). w5's forms there move gated cells
/// off Excel (T-2977: `FDIST(Data!$D5,1,9)`, gen-m9/c18dbf21f11093f3 A151, by 9 ulp, and
/// `2*F.DIST.RT(1E+12,22,5)`, probes/t2829-ftest-tail-over-b S!A17, visibly).
fn f_tail(df1: f64, df2: f64, x: f64, upper: bool) -> f64 {
    let (u, v) = f_beta_arguments(df1, df2, x);
    let (a, b) = (df1 / 2.0, df2 / 2.0);
    let (lower, upper_tail) = closed_or_ported_tails(u, v, a, b).unwrap_or_else(|| {
        let (u, v) = f_kernel_arguments(df1, df2, x);
        beta_reg_tails(u, v, a, b)
    });
    if upper {
        upper_tail
    } else {
        lower
    }
}

/// P(F <= x) for the F distribution: `I_u(df1/2, df2/2)` ([`f_tail`]).
fn f_cdf(df1: f64, df2: f64, x: f64) -> f64 {
    f_tail(df1, df2, x, false)
}

/// P(F > x): the upper tail of [`f_cdf`]'s incomplete beta, taken from its evaluation (the
/// unreflected side directly, not as 1 - a reflected one) rather than as 1 - [`f_cdf`]
/// (T-0970: `F.DIST.RT(0.7,4,2.4)` is 0.6597222222222223 in `F_DIST.xlsx`).
fn f_sf(df1: f64, df2: f64, x: f64) -> f64 {
    f_tail(df1, df2, x, true)
}

/// The F tail F.INV, F.INV.RT and FINV search the crossing of, until they invert the ported kernel
/// (T-2511): today's kernel on u = df1 x/(df1 x + df2) and v = df2/(df1 x + df2) without Excel's
/// oriented closed forms, a shape of 1 taken by [`unoriented_shape_one_tail`]. With [`f_tail`]
/// instead the crossing moves 9 gated cells 2 to 8 ulp off Excel's quantile (T-2977:
/// `FINV(0.025,7,10)`, gen-m9/fec39efaba9284c3 A336; gen-m9/9cca17438a7e6f8a A114 by 8 ulp; `F.INV(0.05,2,5)`,
/// probes-numeric/m9-f-inv A10, at a shape of 1).
fn f_quantile_tail(df1: f64, df2: f64, x: f64, upper: bool) -> f64 {
    let (u, v) = f_kernel_arguments(df1, df2, x);
    let (a, b) = (df1 / 2.0, df2 / 2.0);
    if u > 0.0 && v > 0.0 {
        // The upper tail is I_v(b, a).
        let shape_one = if upper {
            unoriented_shape_one_tail(v, b, a)
        } else {
            unoriented_shape_one_tail(u, a, b)
        };
        if let Some(tail) = shape_one {
            return tail;
        }
    }
    let (lower, upper_tail) = search_tails(u, v, a, b);
    if upper {
        upper_tail
    } else {
        lower
    }
}

/// Above this a + b (half the two df summed) F.TEST's tail is
/// [`beta_dist_lower_tail`]'s; below it the remainder of [`f_test_upper_tail`] is summed
/// in double-double, at most about 30,000 terms (x <= 0.9975 there).
const F_TEST_SERIES_SHAPE_MAX: f64 = 200.0;

/// F.TEST's two-tailed p, `2 min(u, 1 - u)` for the upper tail u of [`f_test_upper_tail`]: with
/// both shapes above 1 and a + b up to [`F_TEST_SERIES_SHAPE_MAX`], TOMS 708 `bratio`'s branches
/// with Excel's pieces where [`f_test_bratio_p`] reaches (T-2681; bit-exact on 76 of the 113 such
/// cells on file against 31 for the remainder model, docs/excel-semantics/statistics.md).
fn f_test_p(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if a + b <= F_TEST_SERIES_SHAPE_MAX {
        if let Some(p) = f_test_bratio_p(x, y, a, b) {
            return p;
        }
    }
    let upper = f_test_upper_tail(x, y, a, b);
    2.0 * upper.min(1.0 - upper)
}

/// F.TEST's upper tail `I_x(a, b)` for `x = df_small/(df_big f + df_small) <= a/(a+b)`,
/// `y = 1 - x`, `a = df_small/2`, `b = df_big/2` (f the larger variance over the smaller):
/// `exp(a ln x) exp(b ln y) R` with the remainder `R = I_x(a,b)/(x^a y^b)` close to correctly
/// rounded.
/// Excel's F.TEST carries the rounding of the two powers (tens of ulps in a deep tail, where
/// a ln x is about -100) but not a continued fraction's: over the 188 distinct F.TEST inputs on
/// file this form is bit-exact on 90 against 53 for the correctly rounded p and visibly off on 6
/// against 40 (docs/excel-semantics/statistics.md "F.TEST / FTEST: larger variance on top").
/// A shape of 1 is F.TEST's own form ([`unoriented_shape_one_tail`]), large shapes are the kernel's
/// ([`beta_dist_lower_tail`]: the bratio port's branches, else today's kernel; T-2977), and
/// a = b = 1/2 is Excel's closed form 2 asin(sqrt x)/pi (T-2571; current scores in statistics.md).
fn f_test_upper_tail(x: f64, y: f64, a: f64, b: f64) -> f64 {
    if x > 0.0 && y > 0.0 {
        if let Some(tail) = unoriented_shape_one_tail(x, a, b) {
            return tail;
        }
    }
    let half_integers = (2.0 * a).fract() == 0.0 && (2.0 * b).fract() == 0.0;
    if !(x > 0.0 && y > 0.0 && half_integers && a + b <= F_TEST_SERIES_SHAPE_MAX) {
        return beta_dist_lower_tail(x, y, a, b);
    }
    // Bit-exact on 13 of the 16 such cells on file against 3 for the series (T-2571; the
    // committed fit is `uv run rustsheet-fitlab f-test-half`, T-2681).
    if a == 0.5 && b == 0.5 {
        return 2.0 * x.sqrt().asin() / std::f64::consts::PI;
    }
    (a * x.ln()).exp() * (b * y.ln()).exp() * half_integer_remainder((x, 0.0), a, b)
}

/// `I_x(a, b)` at a shape of 1 as F.TEST and the F quantile's search ([`f_quantile_tail`]) take it,
/// for 0 < x < 1, on the caller's x without bratio's swap:
/// x^a at b = 1, and 1 - (1 - x)^b at a = 1 (as -expm1(b ln1p(-x)) below
/// [`A1_EXPM1_MAX_X`]); `None` for other shapes. Excel's F.TEST does not take the oriented
/// closed forms of F.DIST and BETA.DIST (`incomplete_beta::closed_form_tails`): with them
/// `F.TEST` moves 2 to 10 ulp off Excel at gen-m9/68d8ffb69ff739d8 A143 (a = 1, b = 1/2, x = 0.571),
/// a790a4d4a6a82756 A192 (a = 4, b = 1, x = 0.618) and bbdfc06cae0e0d71 A356 (a = 5.5, b = 1,
/// x = 0.579; visibly off), each bit-exact on this form (T-2977). The kernel's own tails at a
/// shape of 1 outside the closed forms are `incomplete_beta::shape_one_tails`, oriented.
fn unoriented_shape_one_tail(x: f64, a: f64, b: f64) -> Option<f64> {
    if b == 1.0 {
        return Some((a * x.ln()).exp());
    }
    if a != 1.0 {
        return None;
    }
    Some(if x < A1_EXPM1_MAX_X {
        -(b * (-x).ln_1p()).exp_m1()
    } else {
        1.0 - (b * (1.0 - x).ln()).exp()
    })
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
    // is solved on that tail of the kernel, as for BETA.INV (T-0974), so a tiny right-tail p
    // keeps the digits 1 - p loses. statrs' estimate is kept unrefined only for a deep lower tail,
    // the tail it was given: for a deep upper tail it solved the rounded 1 - p, which
    // `F.INV.RT(1E-16,9,3)` misses by 7% (Excel 60647458731.09812, gen-m9/0a9226016e8f5f33 A45).
    if estimate.is_finite() && lower < F_QUANTILE_REFINE_P_MIN {
        return estimate;
    }
    if lower > 0.5 {
        let upper = if right_tail { p } else { 1.0 - p };
        refine_increasing(
            |x| -f_quantile_tail(df1, df2, x, true),
            -upper,
            estimate,
            0.0,
            f64::MAX,
        )
    } else {
        refine_increasing(
            |x| f_quantile_tail(df1, df2, x, false),
            lower,
            estimate,
            0.0,
            f64::MAX,
        )
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

/// The F density at x >= 0 once the caller's argument rules have passed: Excel's own chain
/// ([`excel_f_density`], T-2586's model M-final) for x > 0, and 0 at x = 0 (df1 >= 3; df1 = 1 and
/// 2 are the caller's). Against the beta density times the Jacobian it replaces, it moves 21,404
/// probes-numeric F.DIST density cells to within 15 digits of Excel and 2,591 out, and no
/// (side, r, df, |c ln x|) region of r5-f-final's map loses cells (T-3034 Outcome).
fn f_pdf(df1: f64, df2: f64, x: f64) -> f64 {
    if x > 0.0 {
        excel_f_density(x, df1, df2)
    } else {
        0.0
    }
}

/// FISHER's kernel for -1 < x < 1: `0.5 * LN((1 + x) / (1 - x))` with Excel's LN and the quotient
/// rounded twice like the operator `/` (docs/excel-semantics/statistics.md "FISHER, FISHERINV,
/// EXPON.DIST and HARMEAN are Excel operators around LN and EXP"). The twice-rounded quotient is
/// what makes `FISHER(1E-15/3.14159)` 2.2204460492503126E-16 (gen-m9/4ab2ba98d26c8a5e A395,
/// excel-graph), where a plain quotient gives 3.3306690738754686E-16; fit lab `fisher`, T-2437.
/// `1 + x` and `1 - x` stay plain: twice-rounded sums score the same 152 of 155 there, so the data
/// do not ask for them.
fn fisher(x: f64) -> f64 {
    0.5 * excel_ln(extended_quotient(1.0 + x, 1.0 - x))
}

/// FISHERINV's kernel: `(EXP(2y) - 1) / (EXP(2y) + 1)` with Excel's EXP and the quotient rounded
/// twice, not a correctly rounded tanh, and 1 once EXP(2y) overflows (docs/excel-semantics/
/// statistics.md "FISHERINV is (EXP(2y)-1)/(EXP(2y)+1)"). The twice-rounded quotient makes
/// `FISHERINV(TAN(PI()))` -2^-53 (gen-m9/ee2497a5e683c5d9 A30, excel-graph); fit lab `fisherinv`, T-2437.
/// `e - 1` and `e + 1` stay plain: twice-rounded sums score the same 133 of 143 there.
fn fisher_inv(y: f64) -> f64 {
    let e = excel_exp(2.0 * y);
    if e.is_infinite() {
        1.0
    } else {
        extended_quotient(e - 1.0, e + 1.0)
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
                let result = fisher(x);
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
                let result = fisher_inv(y);
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
        // A tiny x whose df2/(df1 x) overflows is #NUM! for the cdf, 0 for the density
        // ([`f_beta_ratio_overflows`]).
        if cumulative && f_beta_ratio_overflows(df1, df2, x) {
            return Err(num_error(cell));
        }
        // The density at 0 is infinite for df1 = 1 and exactly 1 for df1 = 2; higher df1 give
        // 0 (`F.DIST(0,1,7,FALSE)` = #NUM!, `F.DIST(0,2,7,FALSE)` = 1;
        // probes-numeric/t1049-fdist A27, A28).
        // A subnormal x is #DIV/0! for the density (round 5 of statistics.md's T-2513 F density entry:
        // probes-numeric t2586-r5-f-edges-a B931, B932: `F.DIST(EXP(-720),3,5,FALSE)`).
        if !cumulative && x > 0.0 && x < f64::MIN_POSITIVE {
            return Err(div_error(cell));
        }
        if x == 0.0 && !cumulative {
            if df1 == 1.0 {
                return Err(num_error(cell));
            }
            if df1 == 2.0 {
                return Ok(1.0);
            }
        }
        FisherSnedecor::new(df1, df2).map_err(|_| num_error(cell))?;
        let result = if cumulative {
            f_cdf(df1, df2, x)
        } else {
            f_pdf(df1, df2, x)
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
        // A tiny x whose df2/(df1 x) overflows is #NUM!, as for F.DIST's cdf
        // ([`f_beta_ratio_overflows`]).
        if f_beta_ratio_overflows(df1, df2, x) {
            return Err(num_error(cell));
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

    /// One F.TEST sample: a range or an array as `values_from_range` / `values_from_array` read
    /// them, and a single value as a sample of at most one, so F.TEST is `#DIV/0!` whatever its
    /// type (`FTEST(5,{1;2;3})`, `FTEST("abc",..)`, `FTEST(TRUE,..)`, `FTEST({1;2;3},SUM(4))`;
    /// probes/t2455-transpose-single-value A8:A40, docs/excel-semantics/statistics.md "F.TEST of a
    /// single value"). A one-cell reference to a blank cell, or a computed blank
    /// (`FTEST(IF(TRUE,B10),..)`, A79), is `#VALUE!` instead.
    fn f_test_sample(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        which: &str,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let (sample, marked) = self.evaluate_sample_arg(node, cell);
        // `FTEST({1;2;3},ERROR.TYPE(#REF!))` and `F.TEST({-1,0,1},INDIRECT("B1",{1}))` are `#NUM!`,
        // either sample (T-2576, T-2577).
        if marked {
            return Err(marked_number_error(cell));
        }
        if self.is_blank_one_cell(&sample) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("F.TEST: the {which} sample is a blank cell"),
            ));
        }
        match sample {
            CalcResult::Range { left, right } => self.values_from_range(left, right),
            // An error element is that error, before the one-point check: `F.TEST({-1,0,1},{1,#N/A,3})`
            // is #N/A (statistics.md "An error element of an array argument is that error";
            // gen-m9/165db58d1540f014 A337, 9cca17438a7e6f8a A43; T-2692). A LAMBDA or nested element
            // stays #VALUE!.
            CalcResult::Array(a) => self.values_from_array(a).map_err(|error| {
                CalcResult::new_error(error, cell, format!("Error in {which} array"))
            }),
            error @ CalcResult::Error { .. } => Err(error),
            _ => Ok(Vec::new()),
        }
    }

    /// Whether `sample` is a one-cell reference to a blank cell: `B9:B9` arrives as a range, `B10`
    /// as the cell's blank value.
    pub(crate) fn is_blank_one_cell(&mut self, sample: &CalcResult) -> bool {
        match sample {
            CalcResult::EmptyCell => true,
            CalcResult::Range { left, right } if left == right => {
                matches!(self.evaluate_cell(*left), CalcResult::EmptyCell)
            }
            _ => false,
        }
    }

    // F.TEST(array1, array2)
    pub(crate) fn fn_f_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // The samples are read in order, each one's error or blank one-cell reference (`#VALUE!`)
        // before the next: `FTEST(5,B10)` and `F.TEST(Data!C8:C8,Data!Z1:Z3)` are `#VALUE!`
        // (probes/t2455-transpose-single-value A35; gen-m9 b4173fe25109a238 A348), while
        // `FTEST(1/0,B10)` is `#DIV/0!` and `FTEST(NA(),B10:B10)` `#N/A` (A77, A78); a blank
        // two-cell range is `#DIV/0!` (A33). statistics.md, T-2255.
        let values1_opts = match self.f_test_sample(&args[0], cell, "first") {
            Ok(v) => v,
            Err(error) => return error,
        };
        let values2_opts = match self.f_test_sample(&args[1], cell, "second") {
            Ok(v) => v,
            Err(error) => return error,
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

        let df1 = (n1 - 1) as f64;
        let df2 = (n2 - 1) as f64;

        if FisherSnedecor::new(df1, df2).is_err() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid parameters for F distribution in F.TEST".to_string(),
            );
        }

        // Excel divides the larger variance by the smaller (F.TEST is exactly symmetric in its
        // samples, probes/ftest-decomposition A7 = E7) and doubles the smaller of the upper tail
        // and its complement: the upper tail of an f above 1 is not always the smaller, as F's
        // median is away from 1 (F.TEST(C1:C6,B1:B8) of probes/stat-argument-modes is 0.9196,
        // T-1292). statistics.md "F.TEST / FTEST: larger variance on top", T-2361.
        let (v_big, v_small, df_big, df_small) = if v1 >= v2 {
            (v1, v2, df1, df2)
        } else {
            (v2, v1, df2, df1)
        };
        let f = v_big / v_small;
        let total = df_big * f + df_small;
        // A variance or ratio that overflows is `#NUM!`: `F.TEST({1E+200,1E+200,-1E+200},..)` is
        // (gen-m9/7674c89c824aca43 A39, f2bd5f380ec002bf A210; numbers.md "Overflow").
        if !f.is_finite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "The variance ratio overflows in F.TEST".to_string(),
            );
        }
        let mut p = f_test_p(
            df_small / total,
            df_big * f / total,
            df_small / 2.0,
            df_big / 2.0,
        );

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
    use super::{f_cdf, f_quantile, f_tail, fisher, fisher_inv};

    // FISHER and FISHERINV round their quotient twice, like Excel's `/` (excel-graph; T-2437).
    #[test]
    fn fisher_rounds_its_quotient_twice() {
        // gen-m9/4ab2ba98d26c8a5e Calc!A395 `FISHER(Data!E4/Data!$D2)`, x = 1E-15/3.14159.
        assert_eq!(
            fisher(3.183_101_550_488_765_5e-16),
            2.220_446_049_250_312_6e-16
        );
        // probes/t2437-fit-separators A1, A4, A8: Excel's cancellation, not atanh.
        assert_eq!(
            fisher(1.491_723_674_723_48e-11),
            1.491_717_860_324_600_7e-11
        );
        assert_eq!(fisher(1.027_594_709_980_55e-16), 0.0);
        assert_eq!(
            fisher(-6.605_891_778_770_15e-16),
            -6.661_338_147_750_943e-16
        );
    }

    // A shape of 1 outside the closed forms forms w1 as (0.5 - w) + 0.5 (T-3007): y0^b0 is 3 ulp
    // low at probes-numeric/t2220-f-dist-sweep-cdf A718 and 1 ulp low at A318.
    #[test]
    fn shape_one_other_tail_is_the_complement() {
        assert_eq!(f_cdf(20.0, 2.0, 5.0), 0.820_348_299_875_155_2);
        assert_eq!(f_cdf(4.0, 2.0, 5.0), 0.826_446_280_991_735_6);
    }

    // There w is -expm1(b0 ln1p(-x0)) with Excel's own expm1 and log1p (T-3170): libm's put
    // gen-m9/146b46e74f2dac3b A329 `F.DIST.RT(10,6,2)` and abc2939ebd1e2365 A318 `FDIST(40,8,2)`
    // 1 ulp low.
    #[test]
    fn shape_one_tail_takes_excels_expm1_and_log1p() {
        assert_eq!(f_tail(6.0, 2.0, 10.0, true), 0.093_686_012_554_127_09);
        assert_eq!(f_tail(8.0, 2.0, 40.0, true), 0.024_614_204_935_833_432);
    }

    #[test]
    fn fisher_inv_rounds_its_quotient_twice() {
        // gen-m9/ee2497a5e683c5d9 Calc!A30 `FISHERINV(TAN(PI()))`, TAN(PI()) = -1.22514845490862E-16.
        assert_eq!(
            fisher_inv(-1.225_148_454_908_62e-16),
            -1.110_223_024_625_156_5e-16
        );
        // probes/t2437-fit-separators A23, A26.
        assert_eq!(
            fisher_inv(-8.011_504_958_898_52e-15),
            -7.993_605_777_301_19e-15
        );
        assert_eq!(
            fisher_inv(-6.552_011_261_371_12e-16),
            -6.661_338_147_750_943e-16
        );
        // FISHER_FISHERINV.xlsx C38: EXP(2y) overflows to 1.
        assert_eq!(fisher_inv(1000.0), 1.0);
    }

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
