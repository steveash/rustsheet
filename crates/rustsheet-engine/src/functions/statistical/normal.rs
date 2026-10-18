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
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The standard normal quantile of `p` in (0, 1): statrs' closed form (`erfc_inv`), O(1).
fn standard_normal_quantile(p: f64, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    let dist = Normal::new(0.0, 1.0)
        .map_err(|_| CalcResult::new_error(Error::ERROR, cell, String::new()))?;
    Ok(dist.inverse_cdf(p))
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
        let dist = Normal::new(mean, std_dev).map_err(|_| num_error(cell))?;
        let x = dist.inverse_cdf(p);
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
    /// `T_QUANTILE`).
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
