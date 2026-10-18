//! LOGNORM.DIST, LOGNORM.INV and LOGNORMDIST (LOGINV is LOGNORM.INV): every argument lifts over
//! an array and a range (SPEC-0011; probes/stat-argument-modes H353-H373,
//! probes/stat-later-argument-lift H49-H55). The kernels are closed form.

use statrs::distribution::{ContinuousCDF, LogNormal};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::statistical::special::{normal_cdf, SQRT_2PI};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `LOGNORM.DIST(x, mean, standard_dev, cumulative)`.
    pub(crate) fn fn_log_norm_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::LogNormDist, args, cell, &mut |model, values| {
            let [x, mean, std_dev, cumulative] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.log_norm_dist_of(x, mean, std_dev, Some(cumulative), cell))
        })
    }

    /// The LOGNORM.DIST kernel; `cumulative` is `None` for LOGNORMDIST, always cumulative.
    fn log_norm_dist_of(
        &self,
        x: &CalcResult,
        mean: &CalcResult,
        std_dev: &CalcResult,
        cumulative: Option<&CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let x = self.cast_number_ref(x, cell)?;
        let mean = self.cast_number_ref(mean, cell)?;
        let std_dev = self.cast_number_ref(std_dev, cell)?;
        let cumulative = match cumulative {
            Some(value) => Self::cast_bool_ref(value, cell)?,
            None => true,
        };
        // Excel domain checks
        if x <= 0.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            normal_cdf(x.ln(), mean, std_dev)
        } else {
            // Same kernel as `normal_pdf` with the extra `x` factor; this order reproduces
            // the cached last digits in LOGNORM.xlsx.
            let d = (x.ln() - mean) / std_dev;
            (-0.5 * d * d).exp() / (x * SQRT_2PI * std_dev)
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `LOGNORM.INV(probability, mean, standard_dev)`, and LOGINV.
    pub(crate) fn fn_log_norm_inv(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::LogNormInv, args, cell, &mut |model, values| {
            number_or_error(model.log_norm_inv_of(values, cell))
        })
    }

    fn log_norm_inv_of(
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
        // Excel domain checks
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return Err(num_error(cell));
        }
        let dist = LogNormal::new(mean, std_dev).map_err(|_| num_error(cell))?;
        let result = dist.inverse_cdf(p);
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `LOGNORMDIST(x, mean, standard_dev)`: LOGNORM.DIST with `cumulative` TRUE.
    pub(crate) fn fn_lognormdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::LogNormDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [x, mean, std_dev] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.log_norm_dist_of(x, mean, std_dev, None, cell))
            },
        )
    }
}
