use statrs::distribution::{Continuous, ContinuousCDF, Gamma};
use statrs::function::gamma::gamma;

use super::special::ln_gamma;

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, number_or_error};
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// `GAMMA(x)`: lifts over an array and a range (SPEC-0011; probes/stat-argument-modes
    /// H284-H286). The Lanczos sum is closed form.
    pub(crate) fn fn_gamma(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Gamma, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x < 0.0 && x.floor() == x {
                    return Err(num_error(cell));
                }
                let result = gamma(x);
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }

    pub(crate) fn fn_gamma_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // GAMMA.DIST(x, alpha, beta, cumulative)
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_scale = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in GAMMA.DIST".to_string(),
            );
        }
        if alpha <= 0.0 || beta_scale <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in GAMMA.DIST".to_string(),
            );
        }

        let rate = 1.0 / beta_scale;

        let dist = match Gamma::new(alpha, rate) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Gamma distribution".to_string(),
                )
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for GAMMA.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_gamma_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // GAMMA.INV(probability, alpha, beta)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_scale = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in GAMMA.INV".to_string(),
            );
        }

        if alpha <= 0.0 || beta_scale <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in GAMMA.INV".to_string(),
            );
        }

        let rate = 1.0 / beta_scale;

        let dist = match Gamma::new(alpha, rate) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Gamma distribution".to_string(),
                )
            }
        };

        let x = dist.inverse_cdf(p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for GAMMA.INV".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    /// `GAMMALN(x)` and GAMMALN.PRECISE: lift over an array and a range (SPEC-0011;
    /// probes/stat-argument-modes H287-H292).
    pub(crate) fn fn_gamma_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::GammaLn, args, cell, &mut |model, values| {
            let [x] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.cast_number_ref(x, cell).and_then(|x| {
                if x < 0.0 {
                    return Err(num_error(cell));
                }
                let result = ln_gamma(x);
                if result.is_nan() || result.is_infinite() {
                    return Err(num_error(cell));
                }
                Ok(result)
            }))
        })
    }
}
