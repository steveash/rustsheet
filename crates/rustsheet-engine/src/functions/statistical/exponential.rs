use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::{excel_exp, excel_expm1};
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// Below this `t = lambda * x` the cdf is Excel's expm1, from it on `1 - EXP(-t)`. Excel's own cut
/// lies between 0.999999999999999 (Kahan's digits) and 1.35079141386477 (1 - EXP's), and the two
/// forms give the same double wherever a search tried in between (probes/t2437-fit-separators
/// A67:A117, excel-graph).
/// FITTED(T-2437, 51 cells, t 0.198..8.07)
const EXPON_EXPM1_CUT: f64 = 1.0;

/// EXPON.DIST's cdf `1 - e^-t` at `t = lambda * x >= 0`: `-expm1(-t)` with Excel's Kahan expm1
/// below [`EXPON_EXPM1_CUT`], `1 - EXP(-t)` with Excel's EXP from it on (docs/excel-semantics/
/// statistics.md "FISHER, FISHERINV, EXPON.DIST and HARMEAN are Excel operators around LN and
/// EXP"; fit lab `expon-cdf`, T-2437). `EXPONDIST(TAN(100*PI()),1E+8,TRUE)` is
/// 1.959369974159244E-07 (gen-m9/06b4e8f5de409d5e A140), libm's expm1 one ulp below.
fn expon_cdf(t: f64) -> f64 {
    if t < EXPON_EXPM1_CUT {
        -excel_expm1(-t)
    } else {
        1.0 - excel_exp(-t)
    }
}

impl Model {
    /// `EXPON.DIST(x, lambda, cumulative)`, and EXPONDIST: every argument lifts over an array
    /// and a range (SPEC-0011; probes/stat-argument-modes H205-H214,
    /// probes/stat-later-argument-lift H41, H43).
    pub(crate) fn fn_expon_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ExponDist, args, cell, &mut |model, values| {
            number_or_error(model.expon_dist_of(values, cell))
        })
    }

    fn expon_dist_of(
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
        if x < 0.0 || lambda <= 0.0 {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            // A subnormal cdf is 0, as in Excel (`(x)=0` TRUE: probes/t2706-subnormal-density-paths
            // B29), though WEIBULL.DIST keeps the same 1E-310 (B34).
            flush_subnormal(expon_cdf(lambda * x))
        } else {
            // A subnormal density is 0, as in Excel (`(x)=0` TRUE: probes/t2693-subnormal-expon-dist
            // A2; docs/excel-semantics/numbers.md "Distribution densities").
            flush_subnormal(lambda * (-lambda * x).exp())
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::expon_cdf;

    // Excel's values (excel-graph; T-2437).
    #[test]
    fn expon_cdf_is_kahan_expm1_below_one_and_one_minus_exp_above() {
        // gen-m9/06b4e8f5de409d5e A140: t = TAN(100*PI()) * 1E+8.
        assert_eq!(
            expon_cdf(1.959_370_166_115_803_6e-15 * 1e8),
            1.959_369_974_159_244e-7
        );
        // gen-m9/623663cacea53bbc A318: EXPON.DIST(PI(),0.01,TRUE).
        assert_eq!(
            expon_cdf(std::f64::consts::PI * 0.01),
            0.030_927_573_695_189_364
        );
        // probes/t2437-fit-separators A114, A117 (Kahan) and A88 (1 - EXP).
        assert_eq!(expon_cdf(0.998_288_783_366_65), 0.631_490_498_478_630_8);
        assert_eq!(expon_cdf(0.999_999_999_999_999), 0.632_120_558_828_557_2);
        assert_eq!(expon_cdf(1.363_658_509_455_04), 0.744_276_503_355_796);
        // A43: t = 9.04503180245175 * 0.915101692530016.
        assert_eq!(
            expon_cdf(0.915_101_692_530_016 * 9.045_031_802_451_75),
            0.999_745_732_554_896_1
        );
    }
}
