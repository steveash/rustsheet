use super::gammaln::excel_gammaln;
use super::incomplete_gamma::gamma_ur;
use super::loader::{excel_bd0, stirlerr_exact};
use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::{excel_exp, excel_exp_extended};
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The largest mean, compared exactly, at which a pmf with k >= 2 is still the closed form: at
/// 1.1 `POISSON.DIST(7,1.1,FALSE)` is the closed form's double and 10 ulp from the saddle point's
/// (k 5, 8, 10, 20: within an ulp of the closed form, 5 to 21 from the saddle point), and at every
/// probed mean from 1.100001 to 1.2 it is the saddle point's to the bit at all five, so the switch
/// lies in [1.1, 1.100001); no mean in between is probed yet (T-3247 lists the probe;
/// probes-numeric/t2055-poisson-pmf-switch).
/// FITTED(T-2055, 133 cells, mean 1.1..1.2, k 5..20)
const CLOSED_FORM_MAX_MEAN: f64 = 1.1;

/// A POISSON-only fit for k = 2 to 5 of the saddle point's stirlerr: 13, 12, 29 and 34 ulp below
/// the exact value (`loader.rs` `stirlerr_exact`), 4E-17 to 1.2E-16. It is not Excel's stirlerr
/// and is not valid for the beta density (which takes the exact value): it is below the ulp of the
/// stored exponent E, so it acts only by flipping E's rounding, and it stands in for a cause not
/// yet identified, `excel_bd0`'s series or E's storage (statistics.md "POISSON.DIST pmf: two closed
/// forms"). Each offset matches the most Excel pmfs at its k with the shipped `excel_bd0`, against
/// the exact value: k = 2 24 of 25 (13), 3 22 of 23 (17), 4 24 of 25 (8), 5 44 of 46 (33); fitted
/// on half of probes-numeric/t2055-poisson-pmf-sweep the other half scores 8/9 (4), 7/10 (8), 8/10
/// (4), 9/10 (3). The gated cells that pin it, which the exact value misses: gen-m9
/// b9a32ce9a1fd0d71 A183, 5cebac1f02bf30f2 A362, c7664e494e418395 A370 (k = 2); 91e167cd8d7eb652
/// A260 (k = 3, the one cell keeping it); 7fd0a98183c6babf A107, a5342a7e24fd803e A47,
/// 5919660b7bb4d195 A50, 1b31092a0d79e5f1 A358 (k = 4); probes-numeric/t1048-poisson A6, B6 and
/// ironcalc-calc POISSON.xlsx D9 (k = 5).
/// At k = 6 no offset beats the exact value (21 of 25), so it takes it.
// TODO(T-3247): replace this proxy with the identified cause.
/// FITTED(T-2055, 119 cells, k 2..5, half of t2055-poisson-pmf-sweep and gated cells)
const POISSON_STIRLERR_2_TO_5: [f64; 4] = [
    f64::from_bits(0x3FA5_2A9B_923E_A63C), // 0.04134069595540921
    f64::from_bits(0x3F9C_579A_268D_80A7), // 0.027677925684998297
    f64::from_bits(0x3F95_4A26_62FD_788C), // 0.020790672103764993
    f64::from_bits(0x3F91_0B4E_513F_CBCB), // 0.016644691189821075
];

/// Excel's Poisson pmf P(X = k) at a positive finite mean (docs/excel-semantics/statistics.md
/// "POISSON.DIST pmf: two closed forms", T-2055). Two forms, by the evidence:
/// - mean <= 1.1, or k <= 1: `EXP(fl(k LN(mean)) - GAMMALN(k + 1)) * EXP(-mean)`, the exponent held
///   in a register, the mean's own factor apart (k = 0 is EXP(-mean));
/// - otherwise Loader's saddle point `EXP(-stirlerr(k) - bd0(k, mean)) / SQRT(2 pi k)`, the
///   exponent stored to a double, the beta density's x87 bd0 (`loader.rs` `excel_bd0`) and the
///   exact stirlerr ([`POISSON_STIRLERR_2_TO_5`] for k = 2 to 5).
///
/// Over the 121 Excel-computed pmf cells with a resolvable argument in gen-m9, probes-numeric
/// t1048-poisson and ironcalc-calc POISSON.xlsx it is Excel's double at 106 and within 4 ulp at
/// all 121, against 47 and 80 for statrs' `exp(k ln mean - mean - ln k!)`; on the 204 cells of
/// probes-numeric/t2055-poisson-pmf-sweep 154 and 198, against 29 and 121.
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
    let stirlerr = match count {
        2 => POISSON_STIRLERR_2_TO_5[0],
        3 => POISSON_STIRLERR_2_TO_5[1],
        4 => POISSON_STIRLERR_2_TO_5[2],
        5 => POISSON_STIRLERR_2_TO_5[3],
        _ => stirlerr_exact(k).0,
    };
    // The beta density's bd0, its 0.2 cut included: the gap at the cut takes the series
    // (`POISSON(2,3,FALSE)` = 0.22404180765538775, 2 ulp from the log form; probes-numeric/
    // t1048-poisson A4). A huge k over a tiny mean overflows k / mean: NaN, the caller's #NUM!,
    // as `excel_dbinom_exponent` stops there.
    let bd0 = excel_bd0(k, mean);
    if !bd0.is_finite() {
        return f64::NAN;
    }
    let exponent = extended_sum(-stirlerr, -bd0);
    extended_quotient(
        excel_exp(exponent),
        extended_sqrt(extended_product(std::f64::consts::TAU, k)),
    )
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
