//! The normal distribution functions NORM.DIST, NORM.INV, NORM.S.DIST, NORM.S.INV and
//! CONFIDENCE.NORM (and their legacy names), and CONFIDENCE.T, which lift every argument over an
//! array and a range through `lift_call` (SPEC-0011; probes/stat-argument-modes,
//! probes/stat-later-argument-lift, probes/stat-quantile-lift, docs/excel-semantics/statistics.md
//! "Statistical function argument modes").

use statrs::distribution::{ContinuousCDF, Normal, StudentsT};

use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::statistical::special::{normal_cdf, normal_pdf, standard_normal_pdf};
use crate::functions::statistical::t_dist::{t_upper_quantile, T_DF_MAX};
use crate::functions::util::poly;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// AS 241 PPND16's tail split: |p - 0.5| above it takes the tail rationals (Wichura, Applied
/// Statistics 37, 1988).
const PPND16_SPLIT: f64 = 0.425;

/// PPND16's tail rationals, ascending coefficients: C/D for r = sqrt(-ln(min(p, 1-p))) <= 5, in
/// r - 1.6, and E/F beyond, in r - 5 (`tools/src/rustsheet_tools/fitlab/kernels/norm_inv.py` `PPND16`).
const PPND16_C: [f64; 8] = [
    1.423_437_110_749_683_5,
    4.630_337_846_156_546,
    5.769_497_221_460_691,
    3.647_848_324_763_204_5,
    1.270_458_252_452_368_4,
    0.241_780_725_177_450_6,
    0.022_723_844_989_269_184,
    0.000_774_545_014_278_341_4,
];
const PPND16_D: [f64; 8] = [
    1.0,
    2.053_191_626_637_759,
    1.676_384_830_183_803_8,
    0.689_767_334_985_1,
    0.148_103_976_427_480_08,
    0.015_198_666_563_616_457,
    0.000_547_593_808_499_534_5,
    1.050_750_071_644_416_9e-09,
];
const PPND16_E: [f64; 8] = [
    6.657_904_643_501_103,
    5.463_784_911_164_114,
    1.784_826_539_917_291_3,
    0.296_560_571_828_504_87,
    0.026_532_189_526_576_124,
    0.001_242_660_947_388_078_4,
    2.711_555_568_743_487_6e-05,
    2.010_334_399_292_288_1e-07,
];
const PPND16_F: [f64; 8] = [
    1.0,
    0.599_832_206_555_888,
    0.136_929_880_922_735_8,
    0.014_875_361_290_850_615,
    0.000_786_869_131_145_613_3,
    1.846_318_317_510_054_8e-05,
    1.421_511_758_316_446e-07,
    2.044_263_103_389_939_7e-15,
];

/// Wichura's AS 241 PPND16 tail, the standard normal quantile of `p` for |p - 0.5| > 0.425 in
/// plain doubles, with ln(1 - p) on the upper side; `None` centrally. It is bit-exact on 611 of 616
/// tail cells (docs/excel-semantics/statistics.md "NORM.S.INV is AS 241 PPND16 in the tails",
/// T-1746). Centrally nothing is fitted yet and callers keep statrs' closed form (T-0925).
fn ppnd16_tail(p: f64) -> Option<f64> {
    let q = p - 0.5;
    if q.abs() <= PPND16_SPLIT {
        return None;
    }
    let r = (-(if q < 0.0 { p } else { 1.0 - p }).ln()).sqrt();
    let value = if r <= 5.0 {
        poly(&PPND16_C, r - 1.6) / poly(&PPND16_D, r - 1.6)
    } else {
        poly(&PPND16_E, r - 5.0) / poly(&PPND16_F, r - 5.0)
    };
    Some(if q < 0.0 { -value } else { value })
}

/// The quantile of `p` in (0, 1) of the normal distribution with `mean` and `std_dev`, O(1): the
/// PPND16 tail as `mean + z*std_dev`, else statrs' closed form (`erfc_inv`).
fn normal_quantile(
    p: f64,
    mean: f64,
    std_dev: f64,
    cell: CellReferenceIndex,
) -> Result<f64, CalcResult> {
    if let Some(z) = ppnd16_tail(p) {
        return Ok(mean + z * std_dev);
    }
    let dist = Normal::new(mean, std_dev).map_err(|_| num_error(cell))?;
    Ok(dist.inverse_cdf(p))
}

/// The standard normal quantile of `p` in (0, 1) ([`normal_quantile`]).
fn standard_normal_quantile(p: f64, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    normal_quantile(p, 0.0, 1.0, cell)
}

impl Model {
    /// `NORM.DIST(x, mean, standard_dev, cumulative)`, and NORMDIST.
    pub(crate) fn fn_norm_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::NormDist, args, cell, &mut |model, values| {
            number_or_error(model.norm_dist_of(values, cell))
        })
    }

    fn norm_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, mean, std_dev, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let x = self.cast_number_ref(x, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        // Excel: standard_dev must be > 0
        if std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            normal_cdf(x, mean, std_dev)
        } else {
            normal_pdf(x, mean, std_dev)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(flush_subnormal(result))
    }

    /// `NORM.INV(probability, mean, standard_dev)`, and NORMINV.
    pub(crate) fn fn_norm_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::NormInv, args, cell, &mut |model, values| {
            number_or_error(model.norm_inv_of(values, cell))
        })
    }

    fn norm_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, mean, std_dev] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        let x = normal_quantile(p, mean, std_dev, cell)?;
        if !x.is_finite() {
            return Err(num_error(cell));
        }
        Ok(x)
    }

    /// `NORM.S.DIST(z, cumulative)`.
    pub(crate) fn fn_norm_s_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::NormSdist, args, cell, &mut |model, values| {
            number_or_error(model.norm_s_dist_of(values, cell))
        })
    }

    fn norm_s_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [z, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let z = self.cast_number_ref(z, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        Self::norm_s_dist_number(z, cumulative, cell)
    }

    fn norm_s_dist_number(
        z: f64,
        cumulative: bool,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = if cumulative {
            normal_cdf(z, 0.0, 1.0)
        } else {
            standard_normal_pdf(z)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(flush_subnormal(result))
    }

    /// `NORM.S.INV(probability)`, and NORMSINV.
    pub(crate) fn fn_norm_s_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::NormSInv, args, cell, &mut |model, values| {
            number_or_error(model.norm_s_inv_of(values, cell))
        })
    }

    fn norm_s_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let p = self.cast_number_ref(p, cell)?;
        if p <= 0.0 || p >= 1.0 {
            return Err(num_error(cell));
        }
        let z = standard_normal_quantile(p, cell)?;
        if !z.is_finite() {
            return Err(num_error(cell));
        }
        Ok(z)
    }

    /// `CONFIDENCE.NORM(alpha, standard_dev, size)`, and CONFIDENCE.
    pub(crate) fn fn_confidence_norm(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::ConfidenceNorm,
            args,
            cell,
            &mut |model, values| number_or_error(model.confidence_norm_of(values, cell)),
        )
    }

    fn confidence_norm_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [alpha, std_dev, size] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let alpha = self.cast_number_ref(alpha, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        let size = self.cast_number_ref(size, cell)?.floor();
        if alpha <= 0.0 || alpha >= 1.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        if size < 1.0 {
            return Err(num_error(cell));
        }
        let quantile = standard_normal_quantile(1.0 - alpha / 2.0, cell)?;
        if !quantile.is_finite() {
            return Err(num_error(cell));
        }
        Ok(quantile * std_dev / size.sqrt())
    }

    /// `CONFIDENCE.T(alpha, standard_dev, size)`: every argument lifts over an array and a range
    /// (SPEC-0011; probes/stat-argument-modes, probes/stat-quantile-lift). The Student-t quantile
    /// is T.INV.2T's (`t_upper_quantile`), which the lifting table charges (kinds.rs
    /// `T_QUANTILE_WEIGHT`).
    pub(crate) fn fn_confidence_t(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::ConfidenceT, args, cell, &mut |model, values| {
            number_or_error(model.confidence_t_of(values, cell))
        })
    }

    fn confidence_t_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [alpha, std_dev, size] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let alpha = self.cast_number_ref(alpha, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        let size = self.cast_number_ref(size, cell)?.trunc();
        if alpha <= 0.0 || alpha >= 1.0 || std_dev <= 0.0 || size <= 0.0 {
            return Err(num_error(cell));
        }
        // Need at least 2 observations so df = n - 1 > 0
        if size < 2.0 {
            return Err(div_error(cell));
        }
        let df = size - 1.0;
        if df > T_DF_MAX {
            return Err(num_error(cell));
        }
        let t_dist = StudentsT::new(0.0, 1.0, df)
            .map_err(|_| CalcResult::new_error(Error::ERROR, cell, String::new()))?;
        // Two-sided CI => use 1 - alpha/2
        let t_crit = t_upper_quantile(&t_dist, df, alpha / 2.0);
        if !t_crit.is_finite() {
            return Err(num_error(cell));
        }
        Ok(t_crit * std_dev / size.sqrt())
    }

    /// `NORMSDIST(z)`: NORM.S.DIST with `cumulative` TRUE.
    pub(crate) fn fn_normsdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::NormSDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [z] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(
                    model
                        .cast_number_ref(z, cell)
                        .and_then(|z| Self::norm_s_dist_number(z, true, cell)),
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppnd16_tails_only() {
        assert_eq!(ppnd16_tail(0.5), None);
        assert_eq!(ppnd16_tail(0.9), None);
        // NORM_DIST.xlsx `NORM.S.DIST NORM.S.INV`!D13: Excel's NORM.S.INV(NORM.S.DIST(-3)) is -3.
        assert_eq!(ppnd16_tail(0.001_349_898_031_630_093_3), Some(-3.0));
        // The upper side uses ln(1 - p): the tail is odd up to that rounding.
        let upper = ppnd16_tail(0.99).unwrap_or(f64::NAN);
        let lower = ppnd16_tail(0.01).unwrap_or(f64::NAN);
        assert!((upper + lower).abs() < 1e-14, "{upper} {lower}");
        // r > 5 takes the E/F rational.
        assert!(ppnd16_tail(1e-300).is_some_and(|z| (z + 37.047_096_299_361_2).abs() < 1e-9));
    }
}
