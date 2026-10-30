use super::f_density::binet;
use super::gammaln::excel_gammaln;
use super::incomplete_gamma::gamma_ur;
use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::{excel_exp, excel_exp_extended};
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// Up to this relative gap |k - mean| <= cut (k + mean), Loader's bd0 is its series
/// in v = (k - mean)/(k + mean); above it, the log form. Excel's cut is 0.2, not R's 0.1, as in its
/// beta density (docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's
/// saddle point", T-2513), and the gap at the cut itself takes the series: `POISSON(2,3,FALSE)` =
/// 0.22404180765538775, where the log form is 2 ulp off (probes-numeric/t1048-poisson A4).
const BD0_SERIES_CUT: f64 = 0.2;

/// The largest mean, compared exactly, at which a pmf with k >= 2 is still the closed form: at
/// 1.1 `POISSON.DIST(7,1.1,FALSE)` is the closed form's double and 10 ulp from the saddle point's
/// (k 5, 8, 10, 20: within an ulp of the closed form, 5 to 21 from the saddle point), and from
/// 1.100001 on it is the saddle point's to the bit at all five (probes-numeric/
/// t2055-poisson-pmf-switch).
const CLOSED_FORM_MAX_MEAN: f64 = 1.1;

/// The series terms bd0 sums at most: below the cut |v| < 0.2, so each term is under 1/25 of the
/// one before and the sum stops changing within about 12 terms; the bound only caps the loop.
const BD0_SERIES_TERMS: u32 = 64;

/// Excel's stirlerr(k) for k = 2, ..., 6, as the saddle point uses it: 13, 12, 29, 34 and 39 ulp
/// below the exact value (`f_density.rs` `binet`), about 4E-17 to 1.2E-16 low, the offsets that
/// reproduce the most Excel pmfs at each k: 32 of 33 Excel cells at k = 2 (21 with the exact
/// value), 22 of 23 at 3 (18), 24 of 25 at 4 (8), 46 of 48 at 5 (33), 23 of 26 at 6 (20); from
/// k = 7 on the exact value is as good (probes-numeric/t2055-poisson-pmf-sweep, -switch,
/// t1048-poisson; gen-m9). No closed form of Excel's GAMMALN or LN reproduces them.
const EXCEL_STIRLERR_2_TO_6: [f64; 5] = [
    f64::from_bits(0x3FA5_2A9B_923E_A63C), // 0.04134069595540921
    f64::from_bits(0x3F9C_579A_268D_80A7), // 0.027677925684998297
    f64::from_bits(0x3F95_4A26_62FD_788C), // 0.020790672103764993
    f64::from_bits(0x3F91_0B4E_513F_CBCB), // 0.016644691189821075
    f64::from_bits(0x3F8C_6B16_7BEB_DF0F), // 0.01387612882307068
];

/// Excel's Poisson pmf P(X = k) at a positive finite mean (docs/excel-semantics/statistics.md
/// "POISSON.DIST pmf: two closed forms", T-2055). Two forms, by the evidence:
/// - mean <= 1.1, or k <= 1: `EXP(fl(k LN(mean)) - GAMMALN(k + 1)) * EXP(-mean)`, the exponent held
///   in a register, the mean's own factor apart (k = 0 is EXP(-mean));
/// - otherwise Loader's saddle point `EXP(-stirlerr(k) - bd0(k, mean)) / SQRT(2 pi k)`, the
///   exponent stored to a double, Excel's x87 bd0 and stirlerr ([`EXCEL_STIRLERR_2_TO_6`], then
///   the exact value).
///
/// Over the 121 Excel-computed pmf cells with a resolvable argument in gen-m9, probes-numeric
/// t1048-poisson and ironcalc-calc POISSON.xlsx it is Excel's double at 104 and within 4 ulp at
/// all 121, against 47 and 80 for statrs' `exp(k ln mean - mean - ln k!)`; on the 204 cells of
/// probes-numeric/t2055-poisson-pmf-sweep 154 and 197, against 29 and 121.
fn excel_poisson_pmf(count: u64, mean: f64) -> f64 {
    // The conversion rounds only above 2^53, where the saddle point is all there is anyway.
    let k = count as f64;
    if mean <= CLOSED_FORM_MAX_MEAN || k <= 1.0 {
        // `POISSON.DIST(9,1E-10,FALSE)` = 2.7557319221230322E-96 here and 67 ulp off with the
        // saddle point; `POISSON(1,100,FALSE)` 3.7200759760208376E-42, 65 off with it (gen-m9
        // a4090567758520df A246, afc00c8b70e78baf A193).
        let exponent = Register::load(extended_product(k, excel_ln(mean)))
            .add(Register::load(-excel_gammaln(k + 1.0)));
        return extended_product(excel_exp_extended(exponent), excel_exp(-mean));
    }
    // `POISSON(1000,1000,FALSE)` = 0.012614611348721503, Excel's double here (probes-numeric/
    // t1048-poisson A29), where statrs was 1,060 ulp off.
    let stirlerr = (2..=6)
        .zip(EXCEL_STIRLERR_2_TO_6)
        .find_map(|(n, value)| (n == count).then_some(value))
        .unwrap_or_else(|| binet(k).0);
    let exponent = extended_sum(-stirlerr, -bd0(k, mean));
    extended_quotient(
        excel_exp(exponent),
        extended_sqrt(extended_product(std::f64::consts::TAU, k)),
    )
}

/// Loader's deviance term bd0(k, mean) = k ln(k / mean) + mean - k for k >= 2, mean > 1.1, each
/// step an x87 op stored to a double: the log form `fl(fl(k LN(fl(k / mean))) + fl(mean - k))`,
/// or the series 2k sum v^(2j+1) / (2j+1) near k = mean, where the log form cancels.
fn bd0(k: f64, mean: f64) -> f64 {
    let gap = k - mean;
    if gap.abs() > BD0_SERIES_CUT * (k + mean) {
        let t = extended_product(k, excel_ln(extended_quotient(k, mean)));
        return extended_sum(t, extended_sum(mean, -k));
    }
    let v = extended_quotient(gap, extended_sum(k, mean));
    let v2 = extended_product(v, v);
    let mut sum = extended_product(gap, v);
    let mut term = extended_product(2.0 * k, v);
    for j in 1..=BD0_SERIES_TERMS {
        term = extended_product(term, v2);
        let next = extended_sum(sum, extended_quotient(term, f64::from(2 * j + 1)));
        if next == sum {
            break;
        }
        sum = next;
    }
    sum
}

impl Model {
    /// `POISSON.DIST(x, mean, cumulative)`, and POISSON: every argument lifts over an array and a
    /// range (SPEC-0011; probes/stat-argument-modes H476-H485, probes/stat-quantile-lift
    /// H47-H50). The cdf is statrs' Q(k + 1, mean), the capped port of `incomplete_gamma.rs`,
    /// which the lifting table charges (lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT`).
    pub(crate) fn fn_poisson_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::PoissonDist, args, cell, &mut |model, values| {
            number_or_error(model.poisson_dist_of(values, cell))
        })
    }

    fn poisson_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, lambda, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let lambda = self.cast_number_ref(lambda, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;

        if x < 0.0 {
            return Err(num_error(cell));
        }
        // An x past u64 (the count truncates to one) is a cdf of 1 for a mean of 0 or more
        // (`POISSON.DIST(2E+29,7,-1)` is 1, probes/t2475-stat-subexpressions P!A216); the
        // pmf there, and a negative mean, stay #NUM! (unprobed).
        if x > (u64::MAX as f64) {
            return if cumulative && lambda >= 0.0 {
                Ok(1.0)
            } else {
                Err(num_error(cell))
            };
        }

        let k = x.trunc() as u64;

        // A negative mean is #NUM! unless x truncates to 0: then both the pmf and the cdf are
        // EXP(-mean), which only P(X = 0) = e^-mean gives (`POISSON(0,-1,0)` = 2.718281828459045,
        // `POISSON(0.3,-0.5,TRUE)` = 1.6487212707001282; gen-m9 607e85cfee83a689 A189, b9a32ce9a1fd0d71
        // A221; statistics.md "Scalar distributions at extreme arguments, part 2").
        if lambda < 0.0 {
            let prob = (-lambda).exp();
            return if k == 0 && prob.is_finite() {
                Ok(prob)
            } else {
                Err(num_error(cell))
            };
        }

        // A mean of 0 is the distribution at 0: P(X <= x) = 1, P(X = 0) = 1, P(X = k > 0) = 0.
        if lambda == 0.0 {
            return Ok(if cumulative || k == 0 { 1.0 } else { 0.0 });
        }

        if !lambda.is_finite() {
            return Err(num_error(cell));
        }
        let prob = if cumulative {
            // statrs' `Poisson::cdf`.
            gamma_ur(k as f64 + 1.0, lambda)
        } else {
            excel_poisson_pmf(k, lambda)
        };
        if !prob.is_finite() {
            return Err(num_error(cell));
        }
        Ok(prob)
    }
}
