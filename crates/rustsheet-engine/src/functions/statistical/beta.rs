use statrs::distribution::{Beta, Continuous, ContinuousCDF};

use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::special::{bisect_increasing, ln_gamma};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// Below this probability `statrs`' `Beta::inverse_cdf` panics (checked from 1E-200 for every shape
/// tried), so the quantile comes from the tail asymptotic or a bisection on the cdf (T-1543).
const BETA_STATRS_MIN_P: f64 = 1e-100;

/// Below this `x` the incomplete beta `I_x(a, b)` equals its leading term and first correction to
/// better than 1E-16.
const BETA_TAIL_X: f64 = 1e-8;

/// Quantile of the standard beta distribution for `p` in (0,1); the time is bounded. Excel gives
/// finite values down to 1E-300 (probes-numeric `t1543-beta-inv-tiny-p`), `NaN` when the result
/// underflows (`BETA.INV(1E-300,0.001,0.001)` is `#NUM!`).
fn beta_quantile(dist: &Beta, a: f64, b: f64, p: f64) -> f64 {
    if p >= BETA_STATRS_MIN_P {
        return dist.inverse_cdf(p);
    }
    // I_x(a, b) ~ x^a / (a B(a, b)) * (1 + a (1 - b) / (a + 1) x) for a small x.
    let ln_b = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let mut ln_x = (p.ln() + a.ln() + ln_b) / a;
    if ln_x < BETA_TAIL_X.ln() {
        let x = ln_x.exp();
        ln_x -= (a * (1.0 - b) / (a + 1.0) * x).ln_1p() / a;
        let x = ln_x.exp();
        return if x > 0.0 { x } else { f64::NAN };
    }
    // x is not small: bisect on the cdf over [0, 1].
    bisect_increasing(|x| dist.cdf(x), p, 0)
}

impl Model {
    // BETA.DIST(x, alpha, beta, cumulative, [A], [B])
    pub(crate) fn fn_beta_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_param = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let cumulative = match self.evaluate_node_in_context(&args[3], cell) {
            CalcResult::Boolean(b) => b,
            CalcResult::Number(n) => n != 0.0,
            CalcResult::EmptyArg => false,
            CalcResult::EmptyCell => false,
            CalcResult::String(s) => {
                let up = s.to_uppercase();
                if up == "TRUE" {
                    true
                } else if up == "FALSE" {
                    false
                } else {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "cumulative must be TRUE/FALSE or numeric".to_string(),
                    };
                }
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid cumulative argument".to_string(),
                }
            }
        };

        // Optional A, B
        let a = if arg_count >= 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        } else {
            0.0
        };

        let b = if arg_count >= 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        } else {
            1.0
        };

        // Excel: alpha <= 0 or beta <= 0 → #NUM!
        if alpha <= 0.0 || beta_param <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in BETA.DIST".to_string(),
            );
        }

        // Excel: if x < A, x > B, or A = B → #NUM!
        if b == a || x < a || x > b {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be between A and B and A < B in BETA.DIST".to_string(),
            );
        }

        // Transform to standard Beta(0,1)
        let width = b - a;
        let t = (x - a) / width;

        let dist = match Beta::new(alpha, beta_param) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Beta distribution".to_string(),
                )
            }
        };

        let result = if cumulative {
            dist.cdf(t)
        } else {
            // general-interval beta pdf: f_X(x) = f_T(t) / (B - A), t=(x-A)/(B-A)
            dist.pdf(t) / width
        };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for BETA.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_beta_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_param = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let a = if arg_count >= 4 {
            match self.get_number_no_bools(&args[3], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        } else {
            0.0
        };

        let b = if arg_count >= 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        } else {
            1.0
        };

        if alpha <= 0.0 || beta_param <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in BETA.INV".to_string(),
            );
        }

        // probability <= 0 or probability > 1 → #NUM!
        // NB: p==0 or p==1 are actually valid inputs.
        if p <= 0.0 || p >= 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in (0,1) in BETA.INV".to_string(),
            );
        }

        if b <= a {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "A must be < B in BETA.INV".to_string(),
            );
        }

        let dist = match Beta::new(alpha, beta_param) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Beta distribution".to_string(),
                )
            }
        };

        let t = beta_quantile(&dist, alpha, beta_param, p);
        if t.is_nan() || t.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for BETA.INV".to_string(),
            );
        }

        // Map back from [0,1] to [A,B]
        let x = a + t * (b - a);
        CalcResult::Number(x)
    }

    // BETADIST(x, alpha, beta, [A], [B]) — always cumulative=TRUE, 3-5 args
    pub(crate) fn fn_betadist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let mut new_args = args.to_vec();
        new_args.insert(3, Node::BooleanKind(true));
        self.fn_beta_dist(&new_args, cell)
    }
}
